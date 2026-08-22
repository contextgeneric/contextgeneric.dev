---
sidebar_label: 'Higher-order providers'
sidebar_position: 7
---

# Higher-order providers

Providers parameterized by other providers, so one implementation can be expressed in terms of
another.

This page answers *how does one implementation build on another without choosing which one?* It shows
the shape, the one piece of syntax that makes it look strange, and the two things it buys: composition
without a context, and a wrapper that falls back to whatever the context already decided. It closes on
why this is the step most often taken too early.

## Choosing is not the only thing you can do with a provider

Wiring picks one implementation from several. That covers a lot, and it does not cover the case where
you want *the same transformation over any of them*.

A scaled area is that case. Scaling knows how to multiply by a factor squared; it does not know how to
compute an area, and there is no reason for it to. The base calculation is not something the wrapper
should choose. It is something the wrapper should take:

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

`ScaledArea` is a **higher-order provider**: a provider with another provider as a parameter, bound to
supply part of the work. Wiring `ScaledArea<RectangleArea>` reads as "compute the rectangle area, then
scale it", and `ScaledArea<CircleArea>` reuses the same scaling over a different base:

```rust
delegate_components! { ScaledRectangle { AreaCalculatorComponent: ScaledArea<RectangleArea> } }
delegate_components! { ScaledCircle    { AreaCalculatorComponent: ScaledArea<CircleArea>    } }
```

This is passing a function to a function, done in types. And because a provider is a name rather than a
value, nesting them costs nothing at runtime: the composition is resolved during compilation, and only
a direct call executes.

## The extra `<Self>`, and the attribute that fills it in

One detail makes these look stranger than they are, and it is worth meeting deliberately rather than in
an error message.

The [trait split](./consumer-and-provider-traits.md) gives a provider trait the context as an explicit
parameter. So when a wrapper requires its inner provider to implement the same capability, the
requirement has to name that context slot:

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

`AreaCalculator<Self>` has no counterpart in the consumer trait a reader has been looking at, so it
arrives as an unexplained argument. [`#[use_provider]`](/docs/reference/attributes/use_provider) exists
to remove it: write `Inner: AreaCalculator` and the attribute supplies the `<Self>`, which is the whole
of what it does. That is the version in the first snippet, and it is the one to write.

The attribute does *not* change the call. `Inner::area(self)` stays as it is: you invoke the inner
provider as an associated function with the context passed explicitly, because that is the shape the
provider trait actually has. It is the one place in a well-written provider where the trait split is
still visible, and trying to make it read as `self.area()` would be calling the context's own wiring,
which is a different thing entirely.

## Composition without a context to name

The quiet payoff is that composing two providers costs nothing to declare:

```rust
pub type ScaledScaledRectangleArea = ScaledArea<ScaledArea<RectangleArea>>;
```

A type alias, with no bounds on it at all. Compare what composing two constrained generic functions
normally takes: the composed signature restating both sets of constraints, in Rust as much as in
Haskell. Here the requirements are discharged where a context wires the stack, not where the stack is
written, so a composition is just a name for a shape and can be published, reused, and further wrapped
without accumulating a `where` clause.

## Falling back to what the context already decided

A wrapper often wants a default: use whatever the context is already wired to, unless told otherwise.
[`UseContext`](/docs/reference/providers/use_context) is the provider that means exactly that, and a
default parameter makes it the fallback:

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

Written unparameterized in a table, `SumAreas` means `SumAreas<UseContext>`, so summing a collection
asks the context how to measure each element:

```rust
delegate_components! {
    App {
        open ShapeAreaCalculatorComponent;

        @ShapeAreaCalculatorComponent.Rectangle: RectangleArea,
        @ShapeAreaCalculatorComponent.<Shape> Vec<Shape>: SumAreas,
    }
}
```

Naming an inner provider instead, `SumAreas<SomethingElse>`, pins that step regardless of what the
context would have said, which is how one entry overrides a nested decision without disturbing the rest
of the table. The default only exists when the provider is declared with an explicit struct; one
declared inline by the attribute has no parameter to default.

## Not every generic provider is one of these

A provider with a type parameter is not automatically higher-order, and the distinction is worth
keeping because most generic providers are the other kind:

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

`Tag` here is a field name, used as a key. Nothing is delegated to it, nothing implements a capability
on its behalf, and none of the machinery on this page applies. A provider is higher-order when a
parameter is *bound to a provider trait* and invoked to do part of the work.

## What it costs

**It is the step most often taken too early.** A provider that is not wrapping anything has nothing to
gain from a parameter, and adding one buys a type argument at every wiring site, a bound to get right,
and a call form that reads unlike the rest of the code. Reach for this when one implementation should
genuinely be expressed in terms of another, not when it merely could be.

**A failure names the stack, not the layer.** Checking a context tells you `ScaledArea<RectangleArea>`
does not work; it does not say which half is at fault. The
[`#[check_providers]`](/docs/reference/macros/check_components) form fixes that by asserting against
each layer separately, and a nested stack is the main reason it exists. But it is another thing to
remember, and without it a deep stack is genuinely hard to debug.

**Compile-time work grows with depth.** Each layer is another round of trait resolution, and a stack
that is several deep and instantiated at many types is one of the places CGP's compile-time cost is most
visible.

**And the call form stays visible.** `Inner::area(self)` is the one construct here that still reads
inside-out, and no attribute hides it. It is a small cost on every higher-order provider written.

## Where to go next

[Aggregate providers](./aggregate-providers.md) is the other thing a provider parameter is confused
with: bundling several wiring choices under one name, which composes tables rather than behaviour.
[Checking your wiring](./check-traits.md) covers the per-layer check this page needs.

[Handlers](./handlers.md) is where composition of this kind is the whole point: a family of
computation components whose combinators are higher-order providers, chained into pipelines.

For the constructs, [`#[use_provider]`](/docs/reference/attributes/use_provider) completes the inner
bound, [`UseContext`](/docs/reference/providers/use_context) is the fallback,
[`#[cgp_impl]`](/docs/reference/macros/cgp_impl) writes the provider, and
[`delegate_components!`](/docs/reference/macros/delegate_components) carries the `open` statement the
last example uses.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
