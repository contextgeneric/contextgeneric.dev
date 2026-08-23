---
sidebar_label: 'UseDelegate'
sidebar_position: 7
---

# `UseDelegate`

Dispatch a generic-parameter component to a different inner provider per type, through a lookup table.
Superseded by the `open` statement.

:::info

### Legacy — read, don't write

`UseDelegate`, the [`#[derive_delegate]`](../attributes/derive_delegate.md) attribute that generates its
provider impl, and its nested-table wiring are the older way to choose a provider per type. The `open`
statement of [`delegate_components!`](../macros/delegate_components.md) does the same job through the
[`RedirectLookup`](redirect_lookup.md) impl every [`#[cgp_component]`](../macros/cgp_component.md)
already has, with no separate table type and no wrapper.

**Prefer `open` for anything new.** This page is here because you will meet `UseDelegate` in existing
code and in CGP's own error and handler components, which are still defined with `#[derive_delegate]`.
It is expected to be deprecated once `open` is shown to cover every dispatch case.

:::

## Overview

`UseDelegate` chooses a provider based on a type argument rather than on the component alone. An
ordinary component picks its provider by looking the component name up in the **context**'s delegation
table, where the context is the type the capability runs against. But when a provider trait carries an
extra generic parameter, such as a `SourceError` to convert or a `Shape` to measure, the right provider
often depends on which concrete type that parameter is. `UseDelegate` performs a second lookup: it
treats one generic parameter as a key and reads the matching inner provider out of a table, so a single
wiring entry can fan out to many type-specific providers.

This lets context-generic providers stay generic even when their implementations would otherwise overlap
on a parameter. Rather than one provider matching every possible `Shape`, you write a small provider per
shape and let `UseDelegate` route each concrete shape to its own. The dispatch is type-directed: the key
is a real type appearing in the trait, and the table maps that type to the provider responsible for it.
Like every CGP provider, `UseDelegate` holds no runtime value; the `Components` table is carried in
`PhantomData`.

## Usage

`UseDelegate` is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the
lookup table, and is most often written with an inline nested table using the `new` keyword:

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

The `new` keyword declares the inner table `AreaCalculatorComponents` in place and maps each concrete
type to its provider. The component must be defined with a
[`#[derive_delegate(UseDelegate<Param>)]`](../attributes/derive_delegate.md) attribute naming the
parameter to dispatch on, which is what generates the `UseDelegate` provider impl that reads the table.

## Examples

`UseDelegate` is wired through a nested table that builds both the outer entry and the inner lookup in
one place. Given an area component that dispatches on a `Shape` parameter:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
#[derive_delegate(UseDelegate<Shape>)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle;
pub struct Circle;

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

The wiring reads in two layers. `MyApp` delegates `AreaCalculatorComponent` to
`UseDelegate<AreaCalculatorComponents>`, so its area calculation routes through `UseDelegate` using that
table. The inner table maps the `Rectangle` type to `RectangleArea` and the `Circle` type to
`CircleArea`. The result is that `MyApp` implements `CanCalculateArea<Rectangle>` through
`RectangleArea` and `CanCalculateArea<Circle>` through `CircleArea`, with `UseDelegate` selecting between
them by the `Shape` argument. Adding a shape is one more entry in the inner table.

The same dispatch, written with `open` for new code, folds the entries into the context's own table:

```rust
delegate_components! {
    MyApp {
        open AreaCalculatorComponent;

        @AreaCalculatorComponent.Rectangle: RectangleArea,
        @AreaCalculatorComponent.Circle: CircleArea,
    }
}
```

## When to reach for it, and when not

**Prefer the [`open` statement](../macros/delegate_components.md) for new per-type dispatch.** It needs
no separate table type, no `UseDelegate` wrapper, and no `#[derive_delegate]` on the component, because
it rides the [`RedirectLookup`](redirect_lookup.md) impl every component already generates.

Read `UseDelegate` when you meet it in existing code, and keep using it where a component is still
defined with [`#[derive_delegate]`](../attributes/derive_delegate.md), including CGP's own error and
handler families. The dispatching-per-type choice is worked through in
[`#[derive_delegate]`](../attributes/derive_delegate.md).

For dispatching on the *input* type of a handler rather than a `Code` parameter, the sibling provider is
[`UseInputDelegate`](handler/use_input_delegate.md).

## Under the hood

The [`#[derive_delegate(UseDelegate<Param>)]`](../attributes/derive_delegate.md) attribute on a
component generates the `UseDelegate` provider impl, which uses `Components` as the table and the named
parameter as the key. For a component such as `CanRaiseError<SourceError>` (provider `ErrorRaiser`) with
a `#[derive_delegate(UseDelegate<SourceError>)]` attribute, the generated impl is:

```rust
impl<Context, SourceError, Components, Delegate> ErrorRaiser<Context, SourceError>
    for UseDelegate<Components>
where
    Context: HasErrorType,
    Components: DelegateComponent<SourceError, Delegate = Delegate>,
    Delegate: ErrorRaiser<Context, SourceError>,
{
    fn raise_error(error: SourceError) -> Context::Error {
        Delegate::raise_error(error)
    }
}
```

The mechanism is a single [`DelegateComponent`](../traits/delegate_component.md) lookup keyed on
`SourceError`. `UseDelegate<Components>` implements `ErrorRaiser` for a given `SourceError` exactly when
`Components` maps that `SourceError` to a delegate that itself implements `ErrorRaiser`, and the method
forwards to it. Only the parameter named inside `UseDelegate<...>` is the key; the rest pass through
unchanged. Each impl is paired with an [`IsProviderFor`](../traits/is_provider_for.md) impl so
dependencies propagate to a check. A component may derive more than one dispatcher when different
parameters should be routed differently.

## Related constructs

- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates this provider's impl; the page
  that works through the legacy-versus-`open` choice.
- [`delegate_components!`](../macros/delegate_components.md) — wires it, usually through a nested table,
  and carries the `open` statement that replaces it.
- [`RedirectLookup`](redirect_lookup.md) — the mechanism `open` rides instead.
- [`UseInputDelegate`](handler/use_input_delegate.md) — the sibling that keys on a handler's `Input` type.
- [`DelegateComponent`](../traits/delegate_component.md) — the table the lookup reads.
- [`IsProviderFor`](../traits/is_provider_for.md) — propagates the dispatched provider's dependencies.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the delegation table
  this reuses, keyed on a type parameter rather than a component name.

## Source

- Struct:
  [`use_delegate.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/use_delegate.rs)
- The generated impl comes from the `#[derive_delegate]` directive parsed in
  [`types/attributes/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/attributes/)
  and emitted by the
  [`cgp_component`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_component/)
  pipeline.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
