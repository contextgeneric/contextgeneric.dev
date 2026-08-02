---
sidebar_label: 'delegate_components!'
---

# `delegate_components!`

Build a context's wiring table, naming the provider that implements each of its components.

## What it's for

[`#[cgp_component]`](./cgp_component.md) separates the trait callers use from the trait
implementations target, which leaves one question open: for a given type, *which* implementation
supplies the behavior? `delegate_components!` answers it. You give it a type and a list of entries —
the component name as the key, the chosen provider as the value — and it records that choice on the
type.

```rust
delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}
```

The mental model is a **table**: a compact list saying which implementation supplies each capability.
It is worth being precise about one thing, because the analogy invites the wrong conclusion. This looks
like an object's method table, but it is resolved entirely at compile time — the keys and values are
types, the lookup happens during trait resolution, and the result monomorphizes to a direct call.
There is no table in the compiled program, no dynamic dispatch, and nothing in the binary for a
provider you did not choose.

The type this table is attached to is a **context**: the type that owns the wiring. Often it is a type
standing for your whole application, whose job is to carry choices rather than data — `struct App;`
with no fields is a perfectly good context. Sometimes, as above, it is the data type itself.

A macro is worth having here because each entry expands to two impls rather than one. Besides the impl
that stores the choice, the macro emits a second one that forwards the chosen provider's own
dependencies back through the table — which is what makes a missing transitive requirement produce a
usable error instead of a dead end. [Under the hood](#under-the-hood) shows both.

## Using it

The macro takes a target type and a brace-delimited list of `Key: Value` entries.

```rust
delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}
```

### Defining the target at the same time

A leading `new` keyword makes the macro declare the target struct as well.

```rust
delegate_components! {
    new GeometryComponents {
        AreaCalculatorComponent: RectangleArea,
        PerimeterCalculatorComponent: RectanglePerimeter,
    }
}
```

This is how you build an **aggregate provider**: a zero-sized provider whose only job is to hold a
table dispatching each component to a sub-provider, so that other contexts can delegate a whole group
of components to it as one unit. An aggregate provider is a *provider*, not a context — it forwards
each component's provider trait onward and never implements one with itself in the context position.
That is why it must be wired with plain `delegate_components!` and never with
[`delegate_and_check_components!`](./delegate_and_check_components.md).

### One provider for several components

A bracketed list on the key side expands to one entry per name:

```rust
delegate_components! {
    MyComponents {
        [
            FooComponent,
            BarComponent,
        ]: FooBarProvider,
        BazComponent: BazProvider,
    }
}
```

### One table for a family of types

A leading generic list makes the whole table generic, wiring every instantiation at once:

```rust
delegate_components! {
    <T> MyContext<T> {
        AreaCalculatorComponent: RectangleArea,
    }
}
```

### Choosing a provider per type: the `open` statement

When a component is generic over a type parameter, you usually want a different provider for each
value of it. The `open` statement folds those per-value entries straight into the context's own table.
A leading `open …;` header opens one or more components, and `@`-path entries then assign a provider
per key:

```rust
delegate_components! {
    MyApp {
        open AreaCalculatorComponent;

        @AreaCalculatorComponent.Rectangle: RectangleArea,
        @AreaCalculatorComponent.Circle: CircleArea,
    }
}
```

The `open` header **must come first**, before any plain `Component: Provider` entries, or the macro
fails to parse. Braces are optional when opening a single component, so `open AreaCalculatorComponent;`
and `open { AreaCalculatorComponent };` are the same; the braced list is only needed to open several at
once. A key may share one provider across several values with the array shorthand on its final segment,
`@AreaCalculatorComponent.[Rectangle, Circle]: SomeProvider`, and may carry generics,
`@SomeComponent.<'a, T> &'a T: SomeProvider`.

`open` needs no extra attribute on the component: it works through the
[`RedirectLookup`](../providers/redirect_lookup.md) impl that every `#[cgp_component]` already
generates. It is a lightweight special case of the full [namespace](./cgp_namespace.md) feature, suited
to a context wiring its own components directly, and it does not combine with a joined namespace where
the component carries a `#[prefix(...)]`.

:::info

### Legacy — read, don't write

Older code dispatches the same way by nesting a table inside a
[`UseDelegate`](../providers/use_delegate.md) value:

```rust
delegate_components! {
    MyApp {
        AreaCalculatorComponent:
            UseDelegate<new AreaCalculatorComponents {
                Rectangle: RectangleArea,
                Circle: CircleArea,
            }>,
    }
}
```

This still works, and you will meet it — including in CGP's own error and handler components, which
are defined with [`#[derive_delegate]`](../attributes/derive_delegate.md). The `open` statement above
achieves the same dispatch with no separate table type and no wrapper, and is preferred for new code.

:::

### Namespace statements

Beyond entries and `open`, the table body accepts the statement forms that opt a context into a
[`cgp_namespace!`](./cgp_namespace.md): a leading `namespace SomeNamespace;` header that forwards
unwired lookups through that namespace, `@`-path keys targeting a route rather than a bare component
name, and `for <T, Provider> in SomeTable { … }` loops that pull entries from another table. These are
described on the namespace page, where they are most often written.

The macro accepts **no attributes** on the table or on any key, and rejects any it finds rather than
ignoring them. Attribute-driven variants such as `#[check_params(...)]` and `#[skip_check]` belong to
[`delegate_and_check_components!`](./delegate_and_check_components.md).

## Examples

A component, a provider, and a context wired to use it:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn print_area(rect: &Rectangle) {
    println!("area = {}", rect.area());
}
```

The payoff is what happens when you change one line. Wiring a second context to a different provider
gives two types that answer `area()` differently, with no change to any code that calls it:

```rust
delegate_components! {
    Square {
        AreaCalculatorComponent: SquareArea,
    }
}
```

## When to reach for it, and when not

Use `delegate_components!` whenever a context needs to choose providers, which is any time you use
[`#[cgp_component]`](./cgp_component.md) at all. The real decision is not whether to wire but **how to
check the wiring**, because CGP's wiring is *lazy*: a table with a missing or wrong entry still
compiles, and the failure only surfaces later, where the capability is used.

- **Pair it with [`check_components!`](./check_components.md)** for anything beyond simple wiring. A
  separate check gives you full control over what is asserted — concrete parameters for generic keys,
  per-provider layers, and opened or namespaced wiring — which is why larger codebases keep the two
  macros apart.
- **Use [`delegate_and_check_components!`](./delegate_and_check_components.md)** when you are getting
  started or the wiring is plain `Component: Provider` entries. It fuses the two so the check cannot be
  forgotten.
- **Use plain `delegate_components!` with no check** for an **aggregate provider** — the
  [`new`-keyword form](#defining-the-target-at-the-same-time). That target is a provider other contexts
  delegate to rather than a context in its own right, so the fused macro's check would fail spuriously
  on it.

The one thing not to do is leave a context's wiring unchecked. Which macro you use to check it scales
with how complicated the wiring is; that it is checked somehow does not.

## Under the hood

:::note

### Advanced

This section shows the impls each entry expands to. You do not need them to wire a context, but they
are what a wiring error names, so reading one makes those errors much easier to follow.
`cargo cgp expand` prints the same thing for your own code.

:::

Each entry becomes a pair of impls. From the single-entry table above, first the
[`DelegateComponent`](../traits/delegate_component.md) impl that records the choice:

```rust
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = RectangleArea;
}
```

That impl alone is the whole lookup. The provider blanket impl generated by
[`#[cgp_component]`](./cgp_component.md) reads it, so `Rectangle` inherits `AreaCalculator<Rectangle>`
from `RectangleArea`, and the consumer blanket impl then gives `Rectangle` the `CanCalculateArea` trait.
The hand-written equivalent of one wiring entry is exactly that block.

Second, the [`IsProviderFor`](../traits/is_provider_for.md) impl that forwards the provider's
requirements:

```rust
impl<__Context__, __Params__>
    IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
```

This is why missing dependencies stay diagnosable. `RectangleArea`'s own `IsProviderFor` impl carries
the `where` bounds it needs, so an unsatisfied requirement flows back through this forwarding impl to
the point of use. Note that these parameters are literally named `__Context__` and `__Params__` in the
emitted code.

The array syntax simply repeats the pair per key. A table pairing
`[FooComponent, BarComponent]` with `FooBarProvider`, alongside a `BazComponent: BazProvider` entry,
yields three `DelegateComponent` impls and three matching `IsProviderFor` impls.

The `open` statement expands to a redirect plus per-value entries stored on the context itself. The
header wires the component to a `RedirectLookup` rooted at the component name inside `MyApp`'s own
table:

```rust
impl DelegateComponent<AreaCalculatorComponent> for MyApp {
    type Delegate = RedirectLookup<MyApp, Path!(@AreaCalculatorComponent)>;
}
```

Each `@AreaCalculatorComponent.Rectangle: RectangleArea` entry then stores its provider in that same
table under a path key:

```rust
impl<__Wildcard__>
    DelegateComponent<PathCons<AreaCalculatorComponent, PathCons<Rectangle, __Wildcard__>>>
    for MyApp
{
    type Delegate = RectangleArea;
}
```

The [`RedirectLookup`](../providers/redirect_lookup.md) impl appends the dispatch parameter onto the
path and reads the result back, so `MyApp: CanCalculateArea<Rectangle>` resolves to `RectangleArea`.

The tail of that key is a **generic `__Wildcard__` parameter rather than `Nil`**, and that is what makes
it work: the entry matches any path *beginning* with this component and this dispatch type, whatever the
lookup appends after it, so one entry answers the redirect without having to predict the exact length of
the path reaching it.

One presentational quirk is worth knowing before you compare these listings against your own expansion.
Both paths above are [`PathCons`](../types/type_level_spines.md) lists, but `cargo cgp expand` prints
them differently — it resugars the header's redirect target to [`Path!`](./path.md) form while leaving
the per-entry key as the raw spine — so the same kind of type appears in two spellings in one expansion.

The legacy nested-table form keys on the same parameter; the difference is only that its per-value
entries live in a separate table type.

<details>
<summary>Formal grammar</summary>

In the Rust Reference's [notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
DelegateComponents -> Generics? `new`? TargetType `{` TableBody `}`

TargetType    -> Type

TableBody     -> Statement* ( Mapping ( `,` Mapping )* `,`? )?

Statement     -> OpenStmt | NamespaceStmt | ForStmt

OpenStmt      -> `open` ( `{` Type ( `,` Type )* `,`? `}` | Type ) `;`

Mapping       -> Key `:`  ProviderValue
               | Key `->` ProviderValue
               | Key `=>` Path

Key           -> SingleKey | MultiKey | PathKey
SingleKey     -> Generics? Type
MultiKey      -> `[` SingleKey ( `,` SingleKey )* `,`? `]`
PathKey       -> Generics? Path

ProviderValue -> Type
               | IDENTIFIER `<` `new` TargetType `{` TableBody `}` `>`

Path          -> `@` PathSegment ( `.` PathSegment )*
```

A leading `Generics` list makes the table generic over the target; `new` additionally emits the target
struct. Each `Mapping` uses one of three operators: `:` maps a key to a provider, which is the common
form, while `->` delegates to the value's own entry for that key and `=>` redirects along an `@`-path;
the latter two belong to the namespace machinery. A `Key` may be a single type, a bracketed list
expanding to one entry per name, or an `@`-path. An `OpenStmt` may omit its braces when opening exactly
one component. `NamespaceStmt`, `ForStmt`, and the `Path` segment rules are defined under
[`cgp_namespace!`](./cgp_namespace.md) and [`Path!`](./path.md). The macro accepts no attributes
anywhere and rejects any it finds.

</details>

## Gotchas

**Wiring is lazy.** A table with a missing entry, or one naming a provider whose own dependencies are
unmet, still compiles. The failure appears later, at the place the capability is used, often as a long
error naming types you did not write. This is the single most common source of confusion with CGP, and
the answer is to check the table — see [`check_components!`](./check_components.md) — and to run
[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) in place of `cargo check`, which leads
with the root cause for the classes it recognizes.

**The `open` header must lead the block.** Placing it after a plain `Component: Provider` entry is a
parse error rather than a semantic one, so the message will not point at the real problem.

**Naming a provider struct that does not exist** is easy to do when a provider was written with
[`#[cgp_impl]`](./cgp_impl.md) *without* the `new` keyword and never declared separately. The error
names an unresolved type rather than anything about wiring.

## Related constructs

- [`#[cgp_component]`](./cgp_component.md) — defines the components this table wires.
- [`#[cgp_impl]`](./cgp_impl.md) and [`#[cgp_provider]`](./cgp_provider.md) — write the providers it
  names.
- [`check_components!`](./check_components.md) — asserts the table is complete and resolvable.
- [`delegate_and_check_components!`](./delegate_and_check_components.md) — wires and checks together.
- [`DelegateComponent`](../traits/delegate_component.md) and
  [`IsProviderFor`](../traits/is_provider_for.md) — the traits each entry expands into.
- [`RedirectLookup`](../providers/redirect_lookup.md) — what `open` resolves through.
- [`UseDelegate`](../providers/use_delegate.md) — the legacy dispatch form `open` supersedes.
- [`cgp_namespace!`](./cgp_namespace.md) — reusable tables, for wiring that outgrows one context.
- [`UseField`](../providers/use_field.md) — the usual value for a field-backed getter.

## Source

- Entry point: [`delegate_components.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/delegate_components.rs)
- Implementation: [`types/delegate_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
