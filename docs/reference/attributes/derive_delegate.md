---
sidebar_label: '#[derive_delegate]'
---

# `#[derive_delegate]`

Generate the `UseDelegate` dispatcher impl for a component. Superseded by the `open` statement.

:::info

### Legacy — read, don't write

`#[derive_delegate]` and the [`UseDelegate`](../providers/use_delegate.md) provider it generates are the
older way to choose an implementation per type. A component no longer needs this attribute to be
dispatched on a type parameter: the `open` statement of
[`delegate_components!`](../macros/delegate_components.md) does the same job through machinery every
component already has, with no separate table and no wrapper.

**Prefer `open` for anything new.** This page is here because you will meet the older form — including in
CGP's own error and handler components, which are still defined with this attribute — and because a
component that keeps it stays compatible with existing wiring. It is expected to be deprecated once
`open` is shown to cover every dispatch case.

:::

## What it's for

A component generic over a type parameter usually wants a different implementation per value of it:
`Rectangle` handled one way, `Circle` another. Something has to look at the type and pick, and written by
hand that dispatcher is an implementation of the provider trait that reads a lookup table, finds the entry
for the type, and forwards every method to it — mechanical code, identical in shape for every component,
differing only in which parameter is the key.

`#[derive_delegate]` generates it:

```rust
#[cgp_component(AreaCalculator)]
#[derive_delegate(UseDelegate<Shape>)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}
```

A **context** — the type the capability runs against, which owns the wiring — then points the component at
[`UseDelegate`](../providers/use_delegate.md) over a table naming one implementation per shape, and the
generated dispatcher does the lookup.

The reason the `open` statement replaced this is that the indirection turned out to be unnecessary. Every
[`#[cgp_component]`](../macros/cgp_component.md) already generates a
[`RedirectLookup`](../providers/redirect_lookup.md) impl, and `open` routes through that instead — so the
per-type entries live on the context itself, there is no second table type to name, and the component
needs no attribute at all.

## Using it

`#[derive_delegate]` takes a wrapper type parameterized by the trait generic to dispatch on:

```rust
#[derive_delegate(UseDelegate<Shape>)]
```

`UseDelegate` is the type that will carry the lookup table, and `Shape` is the trait parameter used as the
key. The key may also be a parenthesized tuple when the table should be keyed on more than one parameter at
once:

```rust
#[derive_delegate(UseDelegate<(Code, Input)>)]
```

**To dispatch on several parameters independently, repeat the attribute**, one per dispatcher. Each names
its own wrapper type, and only the parameter in that wrapper's brackets is used as its key — the rest flow
through unchanged:

```rust
#[cgp_component(Computer)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
pub trait CanCompute<Code, Input> {
    type Output;

    fn compute(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
}
```

`UseDelegate` is the wrapper CGP provides, but the machinery is not tied to it. Any struct of the same
shape works, which is how a second, independent dispatcher is added — a user-defined
`pub struct UseInputDelegate<Components>(pub PhantomData<Components>);` is all the second line above
needs.

The attribute belongs on a [`#[cgp_component]`](../macros/cgp_component.md) trait and only makes sense for
one that has generic parameters.

## Examples

A dispatching component, two implementations, and a context that routes each shape to its own:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
#[derive_delegate(UseDelegate<Shape>)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle { pub width: f64, pub height: f64 }
pub struct Circle { pub radius: f64 }

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<Rectangle> {
    fn area(&self, shape: &Rectangle) -> f64 {
        shape.width * shape.height
    }
}

#[cgp_impl(new CircleArea)]
impl AreaCalculator<Circle> {
    fn area(&self, shape: &Circle) -> f64 {
        core::f64::consts::PI * shape.radius * shape.radius
    }
}
```

The wiring names the wrapper and defines its table in place:

```rust
pub struct MyApp;

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

`MyApp` now implements `CanCalculateArea<Rectangle>` through `RectangleArea` and
`CanCalculateArea<Circle>` through `CircleArea`.

**The same result without the attribute** is what to write instead. Drop `#[derive_delegate]` from the
component and let the context open it:

```rust
delegate_components! {
    MyApp {
        open AreaCalculatorComponent;

        @AreaCalculatorComponent.Rectangle: RectangleArea,
        @AreaCalculatorComponent.Circle: CircleArea,
    }
}
```

No wrapper, no second table type, and nothing on the component. The two forms dispatch on the same
parameter and resolve to the same implementations.

## When to reach for it, and when not

**Do not add `#[derive_delegate]` to a new component.** Use the `open` statement of
[`delegate_components!`](../macros/delegate_components.md), which needs no attribute and no table type.
That is the recommendation without qualification for new code.

Three situations still involve it, and only the first is a reason to write it.

- **Keeping compatibility with existing wiring.** The attribute is what makes
  `UseDelegate<new Table { … }>` wiring possible, so removing it from a published component is a breaking
  change for any downstream user who wires that way. Removing it is still the right direction, but it is a
  breaking change to schedule rather than a cleanup to slip in.
- **A dispatcher `open` cannot express.** `open` resolves through the `RedirectLookup` impl a component
  generates, which keys on the parameter that impl dispatches on. A *second*, independently keyed
  dispatcher — the `UseInputDelegate<Input>` case above — has no `open` equivalent, so a component wanting
  one still declares it here.
- **Reading existing code.** CGP's own error and handler components are defined with this attribute, so
  `UseDelegate` tables appear in wiring you did not write. Understanding the form matters more than being
  able to author it.

## Under the hood

:::note

### Advanced

This section shows the dispatcher the attribute generates. You do not need it to use `#[derive_delegate]`,
but it is what a failed per-type lookup names in an error, and seeing it explains why `open` was able to
replace it. `cargo cgp expand` prints the same thing for your own code.

:::

Each `#[derive_delegate]` adds one provider impl to everything
[`#[cgp_component]`](../macros/cgp_component.md) already emits. It is the ordinary forwarding shape, except
that the lookup key is the dispatch parameter rather than the component name. From the `AreaCalculator`
example:

```rust
impl<__Context__, Shape, __Components__, __Delegate__>
    AreaCalculator<__Context__, Shape> for UseDelegate<__Components__>
where
    __Components__: DelegateComponent<(Shape), Delegate = __Delegate__>,
    __Delegate__: AreaCalculator<__Context__, Shape>,
{
    fn area(__context__: &__Context__, shape: &Shape) -> f64 {
        __Delegate__::area(__context__, shape)
    }
}
```

Reading it back: `UseDelegate<Components>` is a provider for any `Shape` whose entry in `Components`
names something that is itself a provider for that `Shape`, and the method simply forwards. The
`Components` type is the inner table, and [`DelegateComponent`](../traits/delegate_component.md) is the
same trait ordinary wiring is made of — which is why the table is written with
[`delegate_components!`](../macros/delegate_components.md) like any other.

Two details of the real output are worth recognizing. **The key is wrapped in a tuple**, as
`DelegateComponent<(Shape), …>`, so that a single-parameter key and a multi-parameter one compose
uniformly — a `UseDelegate<(Code, Input)>` declaration produces `DelegateComponent<(Code, Input), …>` with
no other change. And the generics carry reserved names: the table is `__Components__` and the looked-up
entry `__Delegate__`, alongside the provider trait's own `__Context__`.

**A component's supertraits ride along into the dispatcher.** The provider trait records each supertrait as
a `Context:` predicate, and because the dispatcher reuses the provider trait's generics that predicate
appears here too — so a component declaring an error type through
[`#[use_type]`](use_type.md) yields a dispatcher whose `where` clause also requires
`__Context__: HasErrorType`.

When several `#[derive_delegate]` attributes are present, one impl is generated per attribute, each keyed
on its own parameter and otherwise identical. The two are independent, so a context may dispatch on either
or nest one table inside the other.

## Gotchas

**The key must be a generic parameter the trait actually declares.** The attribute names it by identifier,
so a parameter that is not in scope on the trait is reported as an unresolved type rather than as anything
about dispatch:

```text
error[E0425]: cannot find type `Shape` in this scope
   |
   | #[derive_delegate(UseDelegate<Shape>)]
   |                               ^^^^^ not found in this scope
```

The usual cause is putting the attribute on a component that has no type parameter to dispatch on, where
there is nothing for it to do.

**One component gets one dispatch mechanism.** Wiring the same component both with `open` and with a
`UseDelegate` table is a coherence conflict, because each produces its own table entry for that key. A
wiring entry expands to two impls, and both of them collide, so the conflict is reported twice — once
for `IsProviderFor` and once for `DelegateComponent`. The second is the readable one, since its trait
argument names the component at issue:

```text
error[E0119]: conflicting implementations of trait `DelegateComponent<AreaCalculatorComponent>`
              for type `App`
   |
   |         open AreaCalculatorComponent;
   |              ----------------------- first implementation here
   |
   |         AreaCalculatorComponent:
   |         ^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `App`
```

Pick one per component. The related restriction is that `open` does not combine with a joined namespace
where the component carries a `#[prefix(...)]`: its lookups are already routed under that path, so the
per-type entries have to be written with the full prefixed path instead.

## Related constructs

- [`delegate_components!`](../macros/delegate_components.md) — its `open` statement is the replacement, and
  its nested-table form is what this attribute enables.
- [`UseDelegate`](../providers/use_delegate.md) — the provider the generated impl is written for.
- [`RedirectLookup`](../providers/redirect_lookup.md) — what `open` resolves through instead.
- [`#[cgp_component]`](../macros/cgp_component.md) — the host, and the source of the provider trait.
- [`DelegateComponent`](../traits/delegate_component.md) — the table trait the lookup reads.
- [`cgp_namespace!`](../macros/cgp_namespace.md) — the full form of the mechanism `open` is a special case of.

## Source

- Parsing and the generated impl: [`types/attributes/derive_delegate/attribute.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/derive_delegate/attribute.rs)
- Emission: [`types/cgp_component/evaluated/item.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_component/evaluated/item.rs)
- The provider: [`providers/use_delegate.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/use_delegate.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
