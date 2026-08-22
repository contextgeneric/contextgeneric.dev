---
sidebar_label: 'check_components!'
---

# `check_components!`

Assert at compile time that a context can actually use each component it wires.

## Overview

**CGP's wiring is lazy, and this macro is the answer to that.** The **context** is the type the
capability runs against, and it supplies the values it needs as its own fields. When
[`delegate_components!`](./delegate_components.md) records that a context delegates a component to some
provider, nothing checks that the provider can do the job. The entry is stored as a type-level fact and
believed. Whether the chosen provider's own requirements hold *for this context* is a separate question,
and it is not asked until something tries to use the component.

So a context can look completely wired, compile, and still be broken. Every entry is accepted, the struct
compiles, the module compiles. Then the first call to the capability fails, often in a file far from
the mistake.

`check_components!` forces the question at a line you choose:

```rust
check_components! {
    Person {
        GreeterComponent,
    }
}
```

That block has no runtime existence. It compiles if `Person` really can use the component and fails if it
cannot, at the wiring site rather than wherever the capability first gets called.

The second thing it buys is a *readable* failure, and that is the part worth understanding. Asking the
obvious question, "does `Person` implement `CanGreet`?", makes the compiler report only the last link in
the chain, typically that some provider does not implement its provider trait, with no word about why. The
macro instead routes the assertion through [`CanUseComponent`](../traits/can_use_component.md), which holds
only when the context both delegates the component *and* the chosen provider's real bounds are satisfied.
Because those bounds ride explicitly on a marker trait, the compiler evaluates them and names the one that
failed. A missing `name` field surfaces as a missing `name` field, not as an opaque "trait not
implemented".

## Usage

The macro takes one or more check tables. Each is a context type followed by a brace-delimited list of the
components to verify on it:

```rust
check_components! {
    Person {
        GreeterComponent,
    }
}
```

Several tables may appear in one invocation, each with its own context type and attributes.

### Components with generic parameters

A component taking type parameters needs them supplied, because a check must name something concrete to
verify. Parameters go after a colon: one bare, several as a tuple, mirroring how the provider trait groups
them:

```rust
check_components! {
    MyApp {
        AreaCalculatorComponent: Rectangle,                 // one parameter
        TransformCalculatorComponent: (Rectangle, f64),     // two, as a tuple
    }
}
```

Bracketing either side expands to the **cartesian product**, so a set of components can be checked against
a set of parameters in one line:

```rust
check_components! {
    MyApp {
        [AreaCalculatorComponent, RotatorComponent]: [Rectangle, Circle],
    }
}
```

That is four checks from one entry. A table may also carry a leading `<...>` generic list and a trailing
`where` clause, to introduce and constrain generics the checked parameters use.

### Naming the check trait

The macro derives a trait name of the form `__Check{Context}` (`__CheckPerson`) from the **final segment**
of the context's path, so `some_mod::Person` also yields `__CheckPerson`. Override it with
`#[check_trait(Name)]` on the table. You need this when two tables in one module would otherwise
collide:

```rust
check_components! {
    #[check_trait(CheckPersonGreeting)]
    Person {
        GreeterComponent,
    }
}
```

Note that [`delegate_and_check_components!`](./delegate_and_check_components.md) derives
`__CanUse{Context}` instead: deliberately different, so both macros can be used once each in one module
without a clash.

### Checking each provider instead of the context

A `#[check_providers(...)]` attribute changes *what* is asserted. Rather than asking whether the context can
use the component, it asserts that each listed provider is a provider for that context, one independent impl
per provider:

```rust
check_components! {
    #[check_trait(CheckScaledProviders)]
    #[check_providers(
        RectangleArea,
        ScaledArea<RectangleArea>,
    )]
    ScaledRectangle {
        AreaCalculatorComponent,
    }
}
```

**This localizes a broken layer of a nested provider stack**, and it is the main reason to keep
`check_components!` separate from the wiring. A dependency missing only from the outer wrapper fails the
wrapper's line alone, while one missing from the inner provider fails both. The pattern of failures
tells you which layer to look at. It must list at least one provider, and may appear at most once per table.

## Examples

A check that catches a real mistake. The provider needs a `name` field, and the context has the wrong field
name:

```rust
use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}

#[derive(HasField)]
pub struct Person {
    pub first_name: String, // GreetHello needs `name`
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    Person {
        GreeterComponent,
    }
}
```

The `delegate_components!` block compiles on its own, because wiring is lazy. The `check_components!` block
does not. It fails *here*, naming the missing field, rather than at some later `person.greet()` in
another file.

A generic component supplies its parameters, and the bracketed form checks several at once:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

check_components! {
    MyApp {
        AreaCalculatorComponent: [Rectangle, Circle],
    }
}
```

This verifies `MyApp: CanCalculateArea<Rectangle>` and `MyApp: CanCalculateArea<Circle>` in one table.

## When to reach for it, and when not

**Every context's wiring should be checked somehow.** That is the rule without exceptions; only the
choice of macro scales with the wiring's complexity.

- **Use a standalone `check_components!` for anything past simple wiring.** It is the form advanced
  codebases keep, because it is the only one with full control: concrete parameters for generic keys,
  `#[check_providers(...)]` for per-layer checks, and coverage of wiring the fused macro's derivation cannot
  read: the `open` statement, `@`-path keys, and [namespaces](./cgp_namespace.md).
- **Use [`delegate_and_check_components!`](./delegate_and_check_components.md) while getting started, or for
  plain `Component: Provider` tables.** It fuses wiring and checking so the check cannot be forgotten. A
  newcomer needs exactly that. Its derivation understands only the plain entry form.
- **Do not check an [aggregate provider](./delegate_components.md#defining-the-target-at-the-same-time) as
  though it were a context.** A `new`-keyword bundle is delegated *to*; it has no fields and never stands in
  the context position, so a context-side check on it asks the wrong question. Verify it through a real
  context that delegates to it, or with `#[check_providers(...)]` against a real context. The
  [Gotchas](#gotchas) show what happens if you try.

One limit is worth stating plainly: **not every unsatisfied bound is a component.** A provider may depend on
an ordinary Rust trait, and there is no component to name in a table for that. The check will still surface
the unsatisfied bound, but you cannot ask for it directly.

## Under the hood

:::note

### Advanced

This section shows what a check table expands to. You do not need it to write one, but knowing that a check
is an empty impl explains both why it costs nothing at runtime and why its errors read the way they do.
`cargo cgp expand` prints the same thing for your own code.

:::

A check table expands to one marker trait plus one empty impl per entry. The trait's supertrait *is* the
assertion; the impl has nothing of its own to prove, so it compiles exactly when the supertrait holds. From
this input:

```rust
check_components! {
    Person {
        GreeterComponent,
    }
}
```

the macro emits:

```rust
trait __CheckPerson<__Component__, __Params__: ?Sized>:
    CanUseComponent<__Component__, __Params__>
{}

impl __CheckPerson<GreeterComponent, ()> for Person {}
```

The impl holds only if `Person: CanUseComponent<GreeterComponent, ()>`, which in turn requires that `Person`
delegates the component and that its delegate satisfies [`IsProviderFor`](../traits/is_provider_for.md) for
`Person`. That indirection is the entire point: because the marker carries the provider's real bounds, an
unmet one is reported specifically rather than as a bare missing implementation. The generic parameters are
literally `__Component__` and `__Params__` in the emitted code.

`__Params__` carries a `?Sized` bound, and that is essential rather than defensive: a component's
parameter may be an unsized type, so a table checking a component declared over `str`
(`ReferenceGetterComponent: (Life<'a>, str)`) would be rejected before the assertion was even evaluated
without it.

Generic parameters land in the `__Params__` slot: one directly, several as a tuple:

```rust
impl __CheckMyApp<AreaCalculatorComponent, Rectangle> for MyApp {}
impl __CheckMyApp<TransformCalculatorComponent, (Rectangle, f64)> for MyApp {}
```

The bracketed forms expand to the product before any impl is emitted, so
`[AreaCalculatorComponent, RotatorComponent]: [Rectangle, Circle]` yields four impls. A table's own generics
and `where` clause are merged into every impl it produces, so
`<'a, I> Context where I: Clone { FooComponent: &'a I }` becomes
`impl<'a, I> __CheckContext<FooComponent, &'a I> for Context where I: Clone {}`.

The `#[check_providers(...)]` form changes both the supertrait and the implementing type. The assertion
becomes `IsProviderFor` with the context *fixed*, and the impls are written for each provider:

```rust
trait CheckScaledProviders<__Component__, __Params__: ?Sized>:
    IsProviderFor<__Component__, ScaledRectangle, __Params__>
{}

impl CheckScaledProviders<AreaCalculatorComponent, ()> for RectangleArea {}
impl CheckScaledProviders<AreaCalculatorComponent, ()> for ScaledArea<RectangleArea> {}
```

Because each provider is checked on its own line, the failures localize: a dependency the inner provider
lacks fails both impls, while one only the wrapper needs fails the wrapper's alone.

<details>
<summary>Formal grammar</summary>

The input is one or more check tables, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CheckComponents -> CheckTable+

CheckTable      -> TableAttr* Generics? ContextType WhereClause? `{` CheckEntries `}`

TableAttr       -> `#` `[` `check_trait` `(` IDENTIFIER `)` `]`
                 | `#` `[` `check_providers` `(` Type ( `,` Type )* `,`? `)` `]`

ContextType     -> Type

CheckEntries    -> ( CheckEntry ( `,` CheckEntry )* `,`? )?

CheckEntry      -> CheckKey ( `:` CheckValue )?

CheckKey        -> Type
                 | `[` ( Type ( `,` Type )* `,`? )? `]`

CheckValue      -> CheckParam
                 | `[` ( CheckParam ( `,` CheckParam )* `,`? )? `]`

CheckParam      -> Generics? Type
```

One invocation may carry several `CheckTable`s, written one after another with no separator. This is how
a module checks two contexts from one block. Both `TableAttr`s are optional, each may appear at most once,
and any other attribute is rejected by name rather than ignored. `#[check_trait(...)]` overrides the derived
`__Check{Context}` name, and `#[check_providers(...)]` switches the assertion to the listed providers. A
`CheckEntry`'s value is omitted for a component with no generic parameters; when present, a bracketed
`CheckKey` or `CheckValue` expands to the cartesian product. A `CheckParam` may carry its own generic list,
which is merged with the table's before the impl is emitted. `Generics`, `WhereClause`, and `Type` are Rust
grammar productions.

Both bracketed lists accept **zero** elements, and the two empty forms behave differently. An empty value
list, `FooComponent: []`, falls back to the no-parameter check, exactly as omitting the colon would. An
empty *key* list, `[]: Rectangle`, produces no entries at all, so the line silently checks nothing; there
is no diagnostic, and it is worth a second look if a table appears to pass without doing anything.

</details>

## Gotchas

**A check on an aggregate provider asks the wrong question, and how it fails depends on the provider.** A
`new`-keyword bundle is not a context, so a context-side check demands that the *bundle* satisfy the leaf
provider's dependencies. If that provider has none, the check passes vacuously and proves nothing. If it
needs anything from its context, such as a field or a type, the check fails, blaming the bundle:

```text
error[E0277]: the trait bound `GeometryComponents: CanUseComponent<AreaCalculatorComponent>`
              is not satisfied
note: required for `RectangleArea` to implement
      `IsProviderFor<AreaCalculatorComponent, GeometryComponents>`
```

Nothing there says the target was never meant to be a context. Wire the bundle with plain
[`delegate_components!`](./delegate_components.md) and check it through a real context instead.

**A component with generic parameters cannot be checked without them, and the error does not say so.**
Omitting the value emits a check with an empty parameter tuple, which a component expecting a `Shape` can
never satisfy. You get an ordinary unsatisfied-wiring complaint that never mentions the
parameters:

```text
error[E0277]: the trait bound `App: CanUseComponent<AreaCalculatorComponent>` is not satisfied
```

That looks identical to a genuinely broken wiring. If a check fails on a generic component and the wiring
looks right, the missing value is the first thing to suspect.

**`#[check_providers(...)]` must list at least one provider**, and is rejected rather than treated as a
no-op:

```text
error: `#[check_providers(...)]` requires at least one provider type.
```

**Two tables in one module can collide on the derived name.** The name comes from the context's *last* path
segment, so `a::Person` and `b::Person` both derive `__CheckPerson`, and so do two tables for the same
context:

```text
error[E0428]: the name `__CheckPerson` is defined multiple times
```

Use `#[check_trait(...)]` on one of them.

**A passing check is not a claim that the capability is correct**, only that it resolves. It proves the
provider was found and its dependencies are satisfiable, not that the provider does what you meant.

## Related constructs

- [`delegate_components!`](./delegate_components.md) — the wiring this verifies.
- [`delegate_and_check_components!`](./delegate_and_check_components.md) — wires and checks in one step, for
  simple tables.
- [`CanUseComponent`](../traits/can_use_component.md) — the assertion the default form makes.
- [`IsProviderFor`](../traits/is_provider_for.md) — what `#[check_providers]` asserts, and what carries a
  provider's real bounds.
- [`DelegateComponent`](../traits/delegate_component.md) — the lazily-accepted table entry that makes a check
  necessary.
- [`#[cgp_component]`](./cgp_component.md) — defines the components a table names.
- [`#[use_provider]`](../attributes/use_provider.md) — builds the nested stacks `#[check_providers]` exists
  to localize.

The ideas behind it:

- [Checking your wiring](/docs/concepts/check-traits) — why wiring is lazy, and one broken context
  followed through three stages of diagnosis.

## Source

- Entry point: [`check_components.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/check_components.rs)
- Implementation: [`types/check_components/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/check_components/)
- Runtime traits: [`cgp-component/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-component/src/traits/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
