---
sidebar_label: 'Higher-order providers'
sidebar_position: 7
---

# Higher-order providers

A **higher-order provider** takes another provider as a type parameter and uses it to perform part
of its work. A wrapper can then reuse the same transformation with different inner implementations.
This page explains the inner-provider bound, composition before a context is chosen, and delegation
back to the context through `UseContext`.

## Choosing is not the only thing you can do with a provider

A provider parameter separates a reusable transformation from the calculation it transforms.
For example, scaling an area requires multiplying a base area by a scale factor squared. The
scaling logic can accept any provider that calculates the base area:

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

`ScaledArea<Inner>` requires `Inner` to implement `AreaCalculator` for the same context.
It passes that context to `Inner::area`, then applies the `scale` value read from the context's
field. The wrapper supplies the scaling logic; its type parameter selects the base calculation.

Wiring chooses the combination for each context. A rectangle and a circle can reuse the wrapper
with different inner providers:

```rust
delegate_components! { ScaledRectangle { AreaCalculatorComponent: ScaledArea<RectangleArea> } }
delegate_components! { ScaledCircle    { AreaCalculatorComponent: ScaledArea<CircleArea>    } }
```

Here `ScaledRectangle` and `ScaledCircle` hold the shape data and the scale factor. Each must
satisfy both the wrapper's field requirement and its selected inner provider's requirements.
`RectangleArea` needs the rectangle's dimensions, while `CircleArea` needs the circle's radius.

Provider composition uses types rather than stored provider values. The compiler resolves the
inner call statically, without a runtime provider lookup. The selected methods still execute their
calculations; inlining and other optimizations determine the final machine code.

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

[`#[use_provider]`](/docs/reference/attributes/use_provider) generates that bound from the shorter
`Inner: AreaCalculator` form in the first example. It inserts the context argument and adds the
requirement to the implementation's `where` clause. Prefer this form when declaring an inner
provider dependency.

The attribute changes bounds, not calls. `Inner::area(self)` invokes the selected inner provider
with the context passed explicitly. A call to `self.area()` would instead use the context's consumer
trait implementation, which may route back to the wrapper rather than to `Inner`.

## Composition without a context to name

A type alias can name a provider composition before any context uses it:

```rust
pub type ScaledScaledRectangleArea = ScaledArea<ScaledArea<RectangleArea>>;
```

The provider structs carry their dependencies on their trait implementations, so this alias does
not need to repeat those bounds. It names a composition that can be published, reused, and wrapped
again without selecting a context or proving that a particular context satisfies it.

The requirements are checked when code uses the composition or explicitly checks it for a context.
A wiring entry alone does not prove they hold. For this alias, the context needs the dimensions
required by `RectangleArea` and the `scale` field used by both wrappers. Both layers read the same
scale value, so nesting them applies its square twice. For base area `12` and scale `2`, the result
is `192`.

## Falling back to what the context already decided

[`UseContext`](/docs/reference/providers/use_context) lets an inner step use the context's existing
consumer-trait implementation. A default type parameter makes that the wrapper's behavior unless
the wiring names another provider.

Summing areas illustrates a useful default because the collection and its elements need different
implementations. In this example, `ShapeAreaCalculator<Context, Shape>` calculates the area of a
separate `Shape` value. The context represents an application that chooses how to measure each
shape, rather than the shape itself:

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

`SumAreas` means `SumAreas<UseContext>` when its type argument is omitted. The wrapper asks the
context to calculate each element's area, then adds the results. An application can select a
rectangle provider and reuse the summing provider for vectors:

```rust
delegate_components! {
    App {
        open ShapeAreaCalculatorComponent;

        @ShapeAreaCalculatorComponent.Rectangle: RectangleArea,
        @ShapeAreaCalculatorComponent.<Shape> Vec<Shape>: SumAreas,
    }
}
```

`open ShapeAreaCalculatorComponent;` enables provider selection by the component's shape parameter.
A call for `Vec<Rectangle>` selects `SumAreas`. Each inner call then asks for the `Rectangle`
trait and reaches `RectangleArea`. The inner lookup is different from the collection lookup,
so it has an independent implementation to resolve to.

An explicit `SumAreas<AnotherCalculator>` uses that provider directly for each element, bypassing
the context's choice for that inner step. It still passes the same context, which must satisfy
`AnotherCalculator`'s requirements.

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

`GetName<Tag>` requires the context to supply a `String` field identified by `Tag`. It does not
require `Tag` to implement a provider trait or invoke it as a provider. By contrast, `ScaledArea`
both constrains `Inner` with a provider-trait bound and calls it for the base calculation.

## What it costs

An inner-provider parameter adds a choice to the API and a bound to the implementation. Use it when
one transformation needs to work with independently selected implementations. A fixed implementation
with nothing to vary does not benefit from an extra provider parameter.

`UseContext` can create a resolution cycle if the inner request selects the same wrapper again.
For example, a wrapper wired for a component cannot use that same component as its only base
implementation. The summing example avoids this by requesting the element trait from the
collection provider. Ensure that the inner lookup resolves to an independently available implementation.

A nested composition can make failures harder to locate. An error on `ScaledArea<RectangleArea>`
may not immediately distinguish the scaling requirement from the rectangle requirements.
The [`#[check_providers]`](/docs/reference/macros/check_components) form gives each provider a
separate assertion against the same context, helping identify which layer lacks a dependency.

Deeper compositions add trait-resolution work, especially when instantiated for many contexts.
They also add types and delegation steps for a reader to follow. The explicit `Inner::area(self)`
call makes the inner choice visible, but requires familiarity with provider-style calls.

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
