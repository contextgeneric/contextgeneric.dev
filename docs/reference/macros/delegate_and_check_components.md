---
sidebar_label: 'delegate_and_check_components!'
sidebar_position: 6
---

# `delegate_and_check_components!`

Wire a context and check that wiring in one macro.

## Overview

CGP's wiring is [lazy](./delegate_components.md#common-mistakes): a
[`delegate_components!`](./delegate_components.md) entry is accepted without verifying that the provider it
names can actually satisfy the component, so a **context** (the type the capability runs against, which
supplies the values it needs as its own fields) can compile while being broken. The fix is a
[`check_components!`](./check_components.md) block, and the problem with the fix is that keeping it in step
with the wiring is manual. Add a delegation, remember to add its check.

`delegate_and_check_components!` removes that bookkeeping by deriving the checks from the delegations:

```rust
delegate_and_check_components! {
    MyContext {
        NameTypeProviderComponent: UseType<String>,
        NameGetterComponent: UseField<Symbol!("name")>,
    }
}
```

Every entry is wired *and* proven, in one place, with nothing to keep in sync. It emits exactly what
writing both macros by hand would emit.

**It is aimed at simple wiring and at getting started, not at being the default everywhere.** Its value is
that a newcomer cannot forget the check and then run into the confusing errors lazy wiring produces. The
derivation understands only a mapping keyed on a component *name*, though, so a codebase whose wiring grows
past that keeps the two macros separate. The reasons are in
[When to use it](#when-to-use-it).

## Usage

The macro takes the same table shape as [`delegate_components!`](./delegate_components.md): an optional
generic list and `new` keyword, a target type, and brace-delimited `Key: Value` entries, plus a few
attributes governing the checking half:

```rust
delegate_and_check_components! {
    ScaledRectangle {
        AreaCalculatorComponent: ScaledArea<RectangleArea>,
    }
}
```

Every delegated component is checked unless explicitly opted out, so the default is "wire it and prove it
works".

### Naming the check trait

The derived trait is named `__CanUse{Context}` (`__CanUseScaledRectangle`). That deliberately differs from
the `__Check{Context}` name [`check_components!`](./check_components.md) derives, so both macros can be used
once each in the same module without colliding. Override it with a table-level `#[check_trait(Name)]`:

```rust
delegate_and_check_components! {
    #[check_trait(TestScaledRectangle)]
    ScaledRectangle {
        AreaCalculatorComponent: ScaledArea<RectangleArea>,
    }
}
```

### Components with generic parameters

A component with type parameters needs `#[check_params(...)]` on its entry. The delegation half does not
need them, because its impl is generic over them, but the check half must name something concrete, and the
macro cannot infer what from the delegation alone:

```rust
delegate_and_check_components! {
    MyApp {
        #[check_params(
            Rectangle,
            Circle,
        )]
        AreaCalculatorComponent:
            UseDelegate<new AreaCalculatorComponents {
                Rectangle: RectangleArea,
                Circle: CircleArea,
            }>,
    }
}
```

The same single-versus-tuple convention as `check_components!` applies: one parameter bare, several as a
tuple.

### Skipping one entry's check

`#[skip_check]` wires an entry and generates no check for it, for a component verified elsewhere. It saves
splitting out a second plain `delegate_components!` block just to leave one component unchecked:

```rust
delegate_and_check_components! {
    ScaledRectangle {
        #[skip_check]
        AreaCalculatorComponent: ScaledArea<RectangleArea>,
    }
}
```

The two per-entry attributes are **mutually exclusive**, and at most one may appear on a given key. A
second one is rejected, as is any other attribute, and `#[skip_check]` takes no arguments.

### Attributes on a list key merge

A bracketed key may carry a check attribute of its own *and* have attributes on the names inside it, and
the two are combined per element rather than one winning. An absent attribute defers to the present one,
two `#[check_params]` lists concatenate, and two `#[skip_check]`s stay a skip:

```rust
delegate_and_check_components! {
    MyApp {
        #[check_params(Rectangle)]
        [
            #[check_params(Circle)]
            AreaCalculatorComponent,
            RotatorComponent,
        ]: ShapeProvider,
    }
}
```

`AreaCalculatorComponent` is checked against `Rectangle` and `Circle`; `RotatorComponent` against
`Rectangle` alone. The one combination refused is a `#[skip_check]` merged with a `#[check_params]`, since
they ask for opposite things:

```text
error: cannot combine #[skip_check] with #[check_params]
```

### Two forms that quietly check nothing

An **empty** `#[check_params()]` has no parameters to iterate over, so it skips the entry exactly as
`#[skip_check]` would, without saying so. Write `#[skip_check]` to say so directly.

A key carrying **only generics** is the opposite case and is easy to assume away: it *is* still checked,
with its generics bound on the check impl and unit parameters. `<I> FooKey<I>: FooProvider` derives
`impl<I> __CanUseContext<FooKey<I>, ()> for Context {}`, which keeps `I` from appearing unbound.

### What is wired, and what is checked

**The wiring half accepts every form [`delegate_components!`](./delegate_components.md) accepts**: all
three operators, all three key forms including grouped `@`-paths, per-key generics, nested table values,
and the `open`, `namespace`, and `for` statements. Read that page for the grammar; the delegation is
literally the same evaluation.

**The checking half reads only some of it, and the gap is silent.** Check entries come from the delegation
*keys*, and only a key that names a component can become one:

| Form | Wired | Checked |
|---|---|---|
| `Component: Provider` | yes | yes |
| `Component -> Table` | yes | yes |
| `[A, B]: Provider` | yes | yes, one per name |
| `@path.Key: Provider` | yes | **no** |
| `Component => @path` | yes | **no** |
| `open` / `namespace` / `for` | yes | **no** |

Nothing warns about the rows that are not checked: the block compiles, the wiring is correct, and those
components simply go unverified. That silence is the practical reason to split the macros once a table uses
more than plain entries. A standalone [`check_components!`](./check_components.md) block can name the
concrete parameters an opened component needs and cover what a namespace brought in.

Attributes are accepted only where they mean something. `#[check_params(...)]` and `#[skip_check]` attach
to the table's single and list keys; on an `@`-path key, or inside a `for` loop, they are rejected outright
rather than read and ignored.

## Examples

The intended use is a straightforward context, wired and proven together:

```rust
use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct MyContext {
    pub name: String,
}

delegate_and_check_components! {
    MyContext {
        NameGetterComponent: UseField<Symbol!("name")>,
    }
}
```

If `MyContext` were missing the `name` field, the derived check would fail here, naming the missing field,
rather than letting the gap survive to some later call to `name()`.

Mixing checked and skipped entries lets a nested delegation be verified more precisely elsewhere while the
rest is checked inline:

```rust
delegate_and_check_components! {
    ScaledRectangle {
        AreaCalculatorComponent:
            ScaledArea<RectangleArea>,

        #[skip_check]
        TransformCalculatorComponent:
            ComplexTransform<RectangleArea>,
    }
}

check_components! {
    #[check_providers(RectangleArea, ComplexTransform<RectangleArea>)]
    ScaledRectangle {
        TransformCalculatorComponent,
    }
}
```

That pairing is the usual reason to use `#[skip_check]`: the fused derivation can only check the
context, and a nested stack is better checked per layer.

## When to use it

**Use it while getting started, and for tables that are plain `Component: Provider` entries.** It is
the form that makes forgetting a check impossible, which is worth more than the control it gives up when the
wiring is simple.

**Keep the two macros separate once the wiring is not simple.** The derivation reads delegation keys, so
there are a few things it cannot do, and each is a reason a larger codebase writes
[`delegate_components!`](./delegate_components.md) and [`check_components!`](./check_components.md) apart:

- **Per-layer checks.** Only a standalone block can use `#[check_providers(...)]`, which localizes a
  broken layer of a nested provider stack.
- **Generic-parameter dispatch.** The `open` statement and `@`-path keys wire a provider per type; the
  derivation cannot tell which concrete types to check, so those entries need a standalone block naming them.
- **[Namespaces](./cgp_namespace.md).** A `namespace` header brings in components the table never names, so a
  derived check covers only the entries written directly and says nothing about what was inherited.

**One case makes this macro wrong rather than merely unnecessary: an
[aggregate provider](./delegate_components.md#defining-the-target-at-the-same-time).** A `new`-keyword bundle
is a provider other contexts delegate *to*, never a context itself. It has no fields and never stands in the
context position, so the derived context-side check asks a question that does not apply to it. Wire a bundle
with plain `delegate_components!`. The [Common Mistakes](#common-mistakes) show what the failure looks like, and why it is
easy to misread.

The invariant across all of this is that a context's wiring is checked *somehow*. This macro is the
beginner-proof way to guarantee that for simple contexts; the two separate macros are the way that scales.

## Under the hood

The macro emits the delegation impls exactly as [`delegate_components!`](./delegate_components.md) would,
then appends a check trait and one impl per non-skipped entry exactly as
[`check_components!`](./check_components.md) would. From this input:

```rust
delegate_and_check_components! {
    #[check_trait(CheckMyContext)]
    MyContext {
        NameTypeProviderComponent: UseType<String>,
        NameGetterComponent: UseField<Symbol!("name")>,
    }
}
```

the wiring half is a [`DelegateComponent`](../traits/wiring/delegate_component.md) impl plus an
[`IsProviderFor`](../traits/wiring/is_provider_for.md) forwarding impl per entry:

```rust
impl DelegateComponent<NameTypeProviderComponent> for MyContext {
    type Delegate = UseType<String>;
}
impl<__Context__, __Params__>
    IsProviderFor<NameTypeProviderComponent, __Context__, __Params__> for MyContext
where
    UseType<String>: IsProviderFor<NameTypeProviderComponent, __Context__, __Params__>,
{}
```

and the same pair again for `NameGetterComponent`. Then comes the checking half: a marker trait aliasing
[`CanUseComponent`](../traits/wiring/can_use_component.md), with one empty impl per delegated component:

```rust
trait CheckMyContext<__Component__, __Params__: ?Sized>:
    CanUseComponent<__Component__, __Params__>
{}

impl CheckMyContext<NameTypeProviderComponent, ()> for MyContext {}
impl CheckMyContext<NameGetterComponent, ()> for MyContext {}
```

Without the `#[check_trait(...)]` override the trait would be `__CanUseMyContext`. The whole output is
identical to a `delegate_components!` block followed by a `check_components!` block whose trait carries the
`__CanUse{Context}` name. This is why moving to separate blocks later changes nothing about what is
checked.

A `#[check_params(...)]` entry expands its parameters into the `__Params__` slot, one check impl per listed
parameter, while its single delegation impl stays generic over them:

```rust
impl __CanUseMyApp<AreaCalculatorComponent, Rectangle> for MyApp {}
impl __CanUseMyApp<AreaCalculatorComponent, Circle> for MyApp {}
```

A `#[skip_check]` entry contributes its delegation impls and no check impl: present in the first half,
absent from the second.

A generic table threads its generics through both halves, so `<T> MyContext<T> { … }` yields
`impl<T> DelegateComponent<…> for MyContext<T>` alongside
`impl<T> __CanUseMyContext<…, ()> for MyContext<T> {}`. A key carrying its own generics is bound on the
derived check the same way, so `<I> BarGetterAtComponent<I>: UseField<Symbol!("dummy")>` checks as
`impl<I> __CanUse…<BarGetterAtComponent<I>, ()> for MyContext {}`.

## Formal grammar

The body is [`delegate_components!`](./delegate_components.md)'s table shape plus the check attributes, in the
Rust Reference's [notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
DelegateAndCheck -> TableAttr* Generics? `new`? TargetType `{` TableBody `}`

TableAttr        -> `#` `[` `check_trait` `(` IDENTIFIER `)` `]`

TableBody        -> Statement* ( CheckedMapping ( `,` CheckedMapping )* `,`? )?

CheckedMapping   -> EntryAttr? Mapping    // Mapping, Key, ProviderValue, Statement
                                          // see delegate_components!

EntryAttr        -> `#` `[` `check_params` `(` Type ( `,` Type )* `,`? `)` `]`
                  | `#` `[` `skip_check` `]`
```

The `Mapping`, `Key`, `ProviderValue`, and `Statement` productions are exactly
[`delegate_components!`](./delegate_components.md)'s; only the attributes differ. The table-level
`#[check_trait(...)]` overrides the derived `__CanUse{Context}` name. Each mapping may carry at most one
`EntryAttr`, and the two are mutually exclusive: `#[check_params(...)]` supplies the parameters a generic
component's check needs, and `#[skip_check]` wires the entry with no check at all.

## Common Mistakes

**Used on an aggregate provider, this macro reports a failure that describes nothing real.** A `new`-keyword
bundle is not a context, so the derived check asks whether the *bundle* satisfies the leaf provider's
dependencies. When the bundled provider needs nothing, the check passes and proves nothing; when it needs
anything from its context, the check fails and blames the bundle:

```text
error[E0277]: the trait bound `GeometryComponents: CanUseComponent<AreaCalculatorComponent>`
              is not satisfied
note: required for `RectangleArea` to implement
      `IsProviderFor<AreaCalculatorComponent, GeometryComponents>`
```

Neither outcome is informative, and neither says the target was never meant to be a context. Use plain
[`delegate_components!`](./delegate_components.md) for a bundle.

**The two per-entry attributes cannot be combined**, and the macro says so rather than picking one:

```text
error: Expected at most one `#[check_params]` or `#[skip_check]` attribute
```

**A generic component without `#[check_params(...)]` cannot be checked, and the error does not say so.** The
delegation succeeds; the derived check is emitted with an empty parameter tuple, which a component expecting a
type parameter can never satisfy. An ordinary unsatisfied-marker complaint against the
provider surfaces instead:

```text
error[E0277]: the trait bound `RectArea: IsProviderFor<AreaCalculatorComponent, App2>` is not satisfied
```

Nothing in it mentions the missing parameters, so it reads like a broken provider rather than an incomplete
check. Add `#[check_params(...)]`, or move the entry to a standalone
[`check_components!`](./check_components.md).

**An inherited, opened, or redirected component is not covered.** A `namespace` header, an `open`
statement, an `@`-path key, and a `=>` redirect are all accepted in the table and all wired, but the
derivation only produces checks for entries keyed on a component name, so a table using any of them is
*partly* checked, with no warning about the rest. The
[coverage table above](#what-is-wired-and-what-is-checked) says which is which; add a standalone
[`check_components!`](./check_components.md) for the rest.

## Related constructs

- [`delegate_components!`](./delegate_components.md) — the wiring half, and what to use alone for a bundle.
- [`check_components!`](./check_components.md) — the checking half, and the form to use once wiring grows.
- [`CanUseComponent`](../traits/wiring/can_use_component.md) — the assertion the derived check makes.
- [`DelegateComponent`](../traits/wiring/delegate_component.md) and
  [`IsProviderFor`](../traits/wiring/is_provider_for.md) — the impls each entry expands into.
- [`#[cgp_component]`](./cgp_component.md) — defines the components a table wires.
- [`cgp_namespace!`](./cgp_namespace.md) — inherited wiring, which the derivation does not cover.
- [`UseField`](../providers/use_field.md) and [`UseType`](../providers/use_type.md) — the providers the
  examples wire.

The ideas behind it:

- [Checking your wiring](/docs/concepts/check-traits) — the lazy-wiring problem this macro exists to
  make unforgettable.

## Source

- Entry point: [`delegate_and_check_components.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/delegate_and_check_components.rs)
- Implementation: [`types/delegate_and_check_components/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_and_check_components/)
- The two reused tables: [`types/delegate_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component/)
  and [`types/check_components/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/check_components/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
