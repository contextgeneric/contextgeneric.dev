---
sidebar_label: 'Higher-order providers'
sidebar_position: 7
---

# Higher-order providers

A **higher-order provider** takes another provider as a type parameter and uses it to perform part
of its work. This separates a reusable transformation, such as scaling a result, from the inner
implementation that produces the result. Wiring can then combine the same wrapper with different
inner providers, while both operate on the same context.

## Choosing is not the only thing you can do with a provider

Scaling an area illustrates what a wrapper can share. Whatever calculation produces the base area,
the scaling step multiplies it by the scale factor squared. Giving that step a provider parameter
lets it reuse the same body with different base calculations:

```rust
#[cgp_impl(new ScaledArea<Inner>)]
#[use_provider(Inner: AreaCalculator)]
impl<Inner> AreaCalculator {
    fn area(&self, #[implicit] scale: f64) -> f64 {
        let base = Inner::area(self);
        base * scale * scale
    }
}
```

Inside `ScaledArea<Inner>`, the call to `Inner::area(self)` calculates the base area using the
same context that the wrapper received. The wrapper then reads `scale` from that context and
applies the scaling. Its bound on `Inner` ensures that the selected provider can perform the base
calculation for this context.

Wiring chooses the combination for each context. A rectangle and a circle can reuse the wrapper
with different inner providers:

```rust
delegate_components! { ScaledRectangle { AreaCalculatorComponent: ScaledArea<RectangleArea> } }
delegate_components! { ScaledCircle    { AreaCalculatorComponent: ScaledArea<CircleArea>    } }
```

Both wired types carry their shape data and a scale factor. They must satisfy the scaling wrapper's
field requirement as well as the inner calculation's requirements: dimensions for `RectangleArea`,
or a radius for `CircleArea`. The wrapper does not need to know which of those calculations it uses.

All of these choices are expressed through types, so the program stores no inner provider value
and performs no runtime provider lookup. The calculations themselves still run as ordinary method
calls, with their final machine code depending on inlining and other compiler optimizations.

## The extra `<Self>`, and the attribute that fills it in

An inner-provider bound must identify the context on which the provider operates. The
[consumer/provider split](./consumer-and-provider-traits.md) gives `AreaCalculator` a leading
context parameter. Inside `#[cgp_impl]`, `Self` names that context, so the explicit bound is
`Inner: AreaCalculator<Self>`:

```rust
#[cgp_impl(new ScaledArea<Inner>)]
impl<Inner> AreaCalculator
where
    Inner: AreaCalculator<Self>,
{
    fn area(&self, #[implicit] scale: f64) -> f64 {
        Inner::area(self) * scale * scale
    }
}
```

The first example uses [`#[use_provider]`](/docs/reference/attributes/use_provider) to write the
same requirement more briefly. Given `Inner: AreaCalculator`, the attribute inserts the context
argument and adds the completed bound to the implementation's `where` clause. This is the preferred
form for declaring an inner-provider dependency.

With that bound in place, the method still calls `Inner::area(self)` explicitly. The attribute
supplies the requirement for that call without rewriting the call itself. Using `self.area()`
would ask the context's consumer trait for an implementation, which may lead back to the wrapper
rather than to `Inner`.

## Composition without a context to name

A type alias can name a provider composition before any context uses it:

```rust
pub type ScaledScaledRectangleArea = ScaledArea<ScaledArea<RectangleArea>>;
```

This alias can exist without bounds because the requirements belong to the providers' trait
implementations, rather than their struct declarations. A library can therefore give the composition
a name, publish it, and wrap it again before any application chooses a context for it.

Naming the composition does not prove that a context can use it, and neither does a wiring entry
alone. That proof is required at a use or an explicit check. Here it requires the dimensions read
by `RectangleArea` and the `scale` field read by both wrappers. Because both layers read the same
scale, they apply its square twice: a base area of `12` with scale `2` becomes `192`.

## Falling back to what the context already decided

[`UseContext`](/docs/reference/providers/use_context) lets an inner step use the context's existing
consumer-trait implementation. A default type parameter makes that the wrapper's behavior unless
the wiring names another provider.

A collection provides a useful case for this default: calculating its total area can ask the
context to measure each element. Here the context represents an application, and the shapes are
separate values, unlike the shape contexts in the scaling example. The provider trait
`ShapeAreaCalculator<Context, Shape>` expresses this separation:

```rust
pub struct SumAreas<Inner = UseContext>(pub PhantomData<Inner>);

#[cgp_impl(SumAreas<Inner>)]
#[use_provider(Inner: ShapeAreaCalculator<Shape>)]
impl<Shape, Inner> ShapeAreaCalculator<Vec<Shape>> {
    fn shape_area(&self, shapes: &Vec<Shape>) -> f64 {
        shapes.iter().map(|shape| Inner::shape_area(self, shape)).sum()
    }
}
```

Omitting the argument makes `SumAreas` stand for `SumAreas<UseContext>`. Each iteration then asks
the context for the element's area before adding it to the total. Wiring can select the rectangle
calculation once and use the summing provider for vectors:

```rust
delegate_components! {
    App {
        open ShapeAreaCalculatorComponent;

        @ShapeAreaCalculatorComponent.Rectangle: RectangleArea,
        @ShapeAreaCalculatorComponent.<Shape> Vec<Shape>: SumAreas,
    }
}
```

With `open ShapeAreaCalculatorComponent;`, the shape parameter determines the provider. A request
for `Vec<Rectangle>` reaches `SumAreas`, whose inner requests are for `Rectangle` and therefore
reach `RectangleArea`. The collection calculation can use `UseContext` safely here because each
inner lookup asks for a different trait instance from the one being computed.

To choose a different element calculation for this sum alone, the table can name
`SumAreas<AnotherCalculator>`. That inner step calls `AnotherCalculator` directly, while continuing
to pass the original context. The context must therefore meet that provider's requirements too.

Declare the provider struct explicitly to give its parameter the `UseContext` default. The
`#[cgp_impl(SumAreas<Inner>)]` block implements that existing struct; the `new` form declares a
provider but does not supply this default.

## Not every generic provider is one of these

A generic provider is higher-order when its parameter represents a provider used to perform work.
Other parameters may select data, field names, or configuration. This getter uses `Tag` as a field
name:

```rust
#[cgp_impl(new GetName<Tag>)]
impl<Tag> NameGetter
where
    Self: HasField<Tag, Value = String>,
{
    fn name(&self) -> &str {
        self.get_field(PhantomData::<Tag>)
    }
}
```

Here `Tag` identifies the `String` field the context must supply. The getter uses it to select
data, without calling it or requiring a provider-trait implementation. `ScaledArea` uses its
parameter differently: `Inner` supplies a calculation, so the wrapper both constrains it with a
provider-trait bound and invokes it.

## What it costs

An inner-provider parameter adds a choice to the API and a bound to the implementation. Use it when
one transformation needs to work with independently selected implementations. A fixed implementation
with nothing to vary does not benefit from an extra provider parameter.

The default needs care when an inner request could return to the same wrapper. If the wrapper
asks for the very component instance being resolved, `UseContext` creates a cycle rather than a
base calculation. The sum avoids this because it requests element calculations from a collection
calculation. Other wrappers likewise need an independently available inner implementation.

A failed requirement inside `ScaledArea<RectangleArea>` can also be difficult to assign to the
right layer. Checking each provider separately against the context with
[`#[check_providers]`](/docs/reference/macros/check_components) helps distinguish a missing scaling
dependency from missing rectangle dimensions.

As compositions grow deeper, both the compiler and the reader have more work to follow. Rust must
resolve the nested bounds for each context that uses them, while a reader traces the provider types
and their delegation steps. The explicit `Inner::area(self)` call helps show the inner choice, once
the reader is familiar with provider-style calls.

## Where to go next

These pages explain related forms of composition and their checks:

- [Aggregate providers](./aggregate-providers.md): Sharing a group of wiring choices through one provider.
- [Checking your wiring](./check-traits.md): Checking the context and individual provider layers.
- [Handlers](./handlers.md): Higher-order providers used as computation and pipeline combinators.
- [`#[use_provider]`](/docs/reference/attributes/use_provider): Inner-provider bounds and context arguments.
- [`UseContext`](/docs/reference/providers/use_context): Forwarding to the context's consumer trait.
- [`#[cgp_impl]`](/docs/reference/macros/cgp_impl): Provider definitions and explicit provider structs.
- [`delegate_components!`](/docs/reference/macros/delegate_components): Wiring and `open` dispatch.
- [Comparison: ML modules](/docs/comparisons/ml-modules): higher-order providers as functors, and the wiring table in place of functor application.
- [Comparison: Policy-based design](/docs/comparisons/policy-based-design): a host template with policy parameters as a higher-order provider.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
