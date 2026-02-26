---
sidebar_position: 2
---

## Using CGP functions with Rust traits

Now that we have understood how to write context-generic functions with `#[cgp_fn]`, let's look at some more advanced use cases.

Suppose that in addition to `rectangle_area`, we also want to define a context-generic `circle_area` function using `#[cgp_fn]`. We can easily write it as follows:

```rust
use core::f64::consts::PI;

#[cgp_fn]
pub fn circle_area(&self, #[implicit] radius: f64) -> f64 {
    PI * radius * radius
}
```

But suppose that we also want to implement a *scaled* version of `circle_area`, we now have to implement another `scaled_circle_area` function as follows:

```rust
#[cgp_fn]
#[uses(CircleArea)]
pub fn scaled_circle_area(&self, #[implicit] scale_factor: f64) -> f64 {
    self.circle_area() * scale_factor * scale_factor
}
```

We can see that both `scaled_circle_area` and `scaled_rectangle_area` share the same structure. The only difference is that `scaled_circle_area` depends on `CircleArea`, but `scaled_rectangle_area` depends on `RectangleArea`.

This repetition of scaled area computation can become tedious if there are many more shapes that we want to support in our application. Ideally, we would like to be able to define an area calculation trait as the common interface to calculate the area of all shapes, such as the following `CanCalculateArea` trait:

```rust
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

Now we can try to implement the `CanCalculateArea` trait on our contexts. For example, suppose that we have the following contexts defined:

```rust
#[derive(HasField)]
pub struct PlainRectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(HasField)]
pub struct ScaledRectangle {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

#[derive(HasField)]
pub struct ScaledRectangleIn2dSpace {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
    pub pos_x: f64,
    pub pos_y: f64,
}

#[derive(HasField)]
pub struct PlainCircle {
    pub radius: f64,
}

#[derive(HasField)]
pub struct ScaledCircle {
    pub radius: f64,
    pub scale_factor: f64,
}
```

We can implement `CanCalculateArea` for each context as follows:

```rust
impl CanCalculateArea for PlainRectangle {
    fn area(&self) -> f64 {
        self.rectangle_area()
    }
}

impl CanCalculateArea for ScaledRectangle {
    fn area(&self) -> f64 {
        self.rectangle_area()
    }
}

impl CanCalculateArea for ScaledRectangleIn2dSpace {
    fn area(&self) -> f64 {
        self.rectangle_area()
    }
}

impl CanCalculateArea for PlainCircle {
    fn area(&self) -> f64 {
        self.circle_area()
    }
}

impl CanCalculateArea for ScaledCircle {
    fn area(&self) -> f64 {
        self.circle_area()
    }
}
```

There are quite a lot of boilerplate implementation that we need to make! If we keep multiple rectangle contexts in our application, like `PlainRectangle`, `ScaledRectangle`, and `ScaledRectangleIn2dSpace`, then we need to implement `CanCalculateArea` for all of them. But fortunately, the existing CGP functions like `rectangle_area` and `circle_area` help us simplify the the implementation body of `CanCalculateArea`, as we only need to forward the call.

Next, let's look at how we can define a unified `scaled_area` CGP function:

```rust
#[cgp_fn]
#[uses(CanCalculateArea)]
pub fn scaled_area(&self, #[implicit] scale_factor: f64) -> f64 {
    self.area() * scale_factor * scale_factor
}
```

Now we can call `scaled_area` on any context that contains a `scale_factor` field, *and* also implements `CanCalculateArea`. That is, we no longer need separate scaled area calculation functions for rectangles and circles!

## Overlapping implementations with CGP components

The earlier implementation of `CanCalculateArea` by our shape contexts introduce quite a bit of boilerplate. It would be nice if we can automatically implement the traits for our contexts, if the context contains the required fields.

For example, a naive attempt might be to write something like the following blanket implementations:

```rust
impl<Context> CanCalculateArea for Context
where
    Self: RectangleArea,
{
    fn area(&self) -> f64 {
        self.rectangle_area()
    }
}

impl<Context> CanCalculateArea for Context
where
    Self: CircleArea,
{
    fn area(&self) -> f64 {
        self.circle_area()
    }
}
```

But if we try that, we would get an error on the second implementation of `CanCalculateArea` with the following error:

```
conflicting implementations of trait `CanCalculateArea`
```

In short, we have run into the infamous [**coherence problem**](https://github.com/Ixrec/rust-orphan-rules) in Rust, which forbids us to write multiple trait implementations that may *overlap* with each other.

The reason for this restriction is pretty simple to understand. For example, suppose that we define a context that contains the fields `width`, `height`, but *also* `radius`, which implementation should we expect the Rust compiler to choose?

```rust
#[derive(HasField)]
pub struct IsThisRectangleOrCircle {
    pub width: f64,
    pub height: f64,
    pub radius: f64,
}
```

Although there are solid reasons why Rust disallows overlapping and orphan implementations, in practice it has fundamentally shaped the mindset of Rust developers to avoid a whole universe of design patterns just to work around the coherence restrictions.

CGP provides ways to partially workaround the coherence restrictions, and enables overlapping implementations through **named** implementation. The ways to do so is straightforward. First, we apply the `#[cgp_component]` macro to our `CanCalculateArea` trait:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

The `#[cgp_component]` macro generates an additional trait called `AreaCalculator`, which we call a **provider trait**. The original `CanCalculateArea` trait is now called a **consumer trait** to allow us to distinguish the two traits.

Using the `AreaCalculator` provider trait, we can now define implementations that resemble blanket implementations using the `#[cgp_impl]` macro:

```rust
#[cgp_impl(new RectangleAreaCalculator)]
impl<Context> AreaCalculator for Context
where
    Self: RectangleArea,
{
    fn area(&self) -> f64 {
        self.rectangle_area()
    }
}

#[cgp_impl(new CircleAreaCalculator)]
impl<Context> AreaCalculator for Context
where
    Self: CircleArea,
{
    fn area(&self) -> f64 {
        self.circle_area()
    }
}
```

Compared to the vanilla Rust implementation, we change the trait name to use the provider trait `AreaCalculator` instead of the consumer trait `CanCalculateArea`. Additionally, we use the `#[cgp_impl]` macro to give the implementation a **name**, `RectangleAreaCalculator`. The `new` keyword in front denotes that we are defining a new provider of that name for the first time.

CGP providers like `RectangleAreaCalculator` are essentially **named implementation** of provider traits like `AreaCalculator`. Unlike regular Rust traits, each provider can freely implement the trait **without any coherence restriction**.

Additionally, the `#[cgp_impl]` macro also provides additional syntactic sugar, so we can simplify our implementation to follows:

```rust
#[cgp_impl(new RectangleAreaCalculator)]
#[uses(RectangleArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        self.rectangle_area()
    }
}

#[cgp_impl(new CircleAreaCalculator)]
#[uses(CircleArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        self.circle_area()
    }
}
```

When we write blanket implementations that are generic over the context type, we can omit the generic parameter and just refer to the generic context as `Self`.

`#[cgp_impl]` also support the same short hand as `#[cgp_fn]`, so we can use `#[uses]` to import the CGP functions `RectangleArea` and `CircleArea` to be used in our implementations.

In fact, with `#[cgp_impl]`, we can skip defining the CGP functions altogether, and inline the function bodies directly:


```rust
#[cgp_impl(new RectangleAreaCalculator)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[cgp_impl(new CircleAreaCalculator)]
impl AreaCalculator {
    fn area(&self, #[implicit] radius: f64) -> f64 {
        PI * radius * radius
    }
}
```

Similar to `#[cgp_fn]`, we can use implicit arguments through the `#[implicit]` attribute. `#[cgp_impl]` would automatically fetch the fields from the context the same way as `#[cgp_fn]`.

### Calling providers directly

Although we have defined the providers `RectangleArea` and `CircleArea`, they are not automatically applied to our shape contexts. Because the coherence restrictions are still enforced by Rust, we still need to do some manual steps to implement the consumer trait on our shape contexts.

But before we do that, we can use a provider by directly calling it on a context. For example:

```rust
let rectangle = PlainRectangle {
    width: 2.0,
    height: 3.0,
};

let area = RectangleAreaCalculator::area(&rectangle);
assert_eq!(area, 6.0);
```

Because at this point we haven't implemented CanCalculateArea for `PlainRectangle`, we can't use the method call syntax `rectangle.area()` to calculate the area just yet. But we can use the explicit syntax `RectangleAreaCalculator::area(&rectangle)` to specifically *choose* `RectangleAreaCalculator` to calculate the area of `rectangle`.

The explicit nature of providers means that we can explicitly choose to use multiple providers on a context, even if they are overlapping. For example, we can use both `RectangleAreaCalculator` and `CircleAreaCalculator` on the `IsThisRectangleOrCircle` context that we have defined earlier:

```rust
let rectangle_or_circle = IsThisRectangleOrCircle {
    width: 2.0,
    height: 3.0,
    radius: 4.0,
};

let rectangle_area = RectangleAreaCalculator::area(&rectangle_or_circle);
assert_eq!(rectangle_area, 6.0);

let circle_area = CircleAreaCalculator::area(&rectangle_or_circle);
assert_eq!(circle_area, 16.0 * PI);
```

The reason we can do so without Rust complaining is that we are explicitly choosing the provider that we want to use with the context. This means that every time we want to calculate the area of the context, we would have to choose the provider again.

### Explicit implementation of consumer traits

To ensure consistency on the chosen provider for a particular context, we can **bind** a provider with the context by implementing the consumer trait *using* the chosen provider. One way to do so is for us to manually implement the consumer trait.

It is worth noting that even though we have annotated the `CanCalculateArea` trait with `#[cgp_component]`, the original trait is still there, and we can still use it like any regular Rust trait. So we can implement the trait manually to forward the implementation to the providers we want to use, like:


```rust
impl CanCalculateArea for PlainRectangle {
    fn area(&self) -> f64 {
        RectangleAreaCalculator::area(self)
    }
}

impl CanCalculateArea for ScaledRectangle {
    fn area(&self) -> f64 {
        RectangleAreaCalculator::area(self)
    }
}

impl CanCalculateArea for ScaledRectangleIn2dSpace {
    fn area(&self) -> f64 {
        RectangleAreaCalculator::area(self)
    }
}

impl CanCalculateArea for PlainCircle {
    fn area(&self) -> f64 {
        CircleAreaCalculator::area(self)
    }
}

impl CanCalculateArea for ScaledCircle {
    fn area(&self) -> f64 {
        CircleAreaCalculator::area(self)
    }
}
```

If we compare to before, the boilerplate is still there, and we are only replacing the original calls like `self.rectangle_area()` with the explicit provider calls. The syntax `RectangleAreaCalculator::area(self)` is used, because we are explicitly using the `area` implementation from `RectangleAreaCalculator`, which is not yet bound to `self` at the time of implementation.

Through the unique binding of provider through consumer trait implementation, we have effectively recovered the coherence requirement of Rust traits. This binding forces us to make a **choice** of which provider we want to use for a context, and that choice cannot be changed on the consumer trait after the binding is done.

For example, we may choose to treat the `IsThisRectangleOrCircle` context as a circle, by forwarding the implementation to `CircleAreaCalculator`:

```rust
impl CanCalculateArea for IsThisRectangleOrCircle {
    fn area(&self) -> f64 {
        CircleAreaCalculator::area(self)
    }
}
```

With this, when we call the `.area()` method on a `IsThisRectangleOrCircle` value, it would always use the circle area implementation:

```rust
let rectangle_or_circle = IsThisRectangleOrCircle {
    width: 2.0,
    height: 3.0,
    radius: 4.0,
};

let area = rectangle_or_circle.area();
assert_eq!(area, 16.0 * PI);

let rectangle_area = RectangleAreaCalculator::area(&rectangle_or_circle);
assert_eq!(rectangle_area, 6.0);

let circle_area = CircleAreaCalculator::area(&rectangle_or_circle);
assert_eq!(circle_area, 16.0 * PI);
```

It is also worth noting that even though we have bound the `CircleAreaCalculator` provider with `IsThisRectangleOrCircle`, we can still explicitly use a different provider like `RectangleAreaCalculator` to calculate the area. There is no violation of coherence rules here, because an explict provider call works the same as an explicit CGP function call, such as:

```rust
let rectangle_area = rectangle_or_circle.rectangle_area();
assert_eq!(rectangle_area, 6.0);

let circle_area = rectangle_or_circle.circle_area();
assert_eq!(circle_area, 16.0 * PI);
```

In a way, CGP providers are essentially **named** CGP functions that implement some provider traits. So they can be used in similar ways as CGP functions, albeit with more verbose syntax.

### Configurable static dispatch with `delegate_components!`

To shorten this further, we can use the `delegate_components!` macro to define an **implementation table** that maps a CGP component to our chosen providers. So we can rewrite the above code as:

```rust
delegate_components! {
    PlainRectangle {
        AreaCalculatorComponent: RectangleAreaCalculator,
    }
}

delegate_components! {
    ScaledRectangle {
        AreaCalculatorComponent: RectangleAreaCalculator,
    }
}

delegate_components! {
    ScaledRectangleIn2dSpace {
        AreaCalculatorComponent: RectangleAreaCalculator,
    }
}

delegate_components! {
    PlainCircle {
        AreaCalculatorComponent: CircleAreaCalculator,
    }
}

delegate_components! {
    ScaledCircle {
        AreaCalculatorComponent: CircleAreaCalculator,
    }
}
```

What the above code effectively does is to build **lookup tables** at **compile time** for Rust's trait system to know which provider implementation it should use to implement the consumer trait. The example lookup tables contain the following entries:

| Context | Component | Provider|
|--|--|--|
| `PlainRectangle` | `AreaCalculatorComponent` | `RectangleAreaCalculator` |
| `ScaledRectangle` | `AreaCalculatorComponent` | `RectangleAreaCalculator` |
| `ScaledRectangleIn2dSpace` | `AreaCalculatorComponent` | `RectangleAreaCalculator` |
| `PlainCircle` | `AreaCalculatorComponent` | `CircleAreaCalculator` |
| `ScaledCircle` | `AreaCalculatorComponent` | `CircleAreaCalculator` |


The type `AreaCalculatorComponent` is called a **component name**, and it is used as a key in the table to identify the CGP trait `CanCalculateArea` that we have defined earlier. By default, the component name of a CGP trait uses the provider trait name followed by a `Component` suffix.

Behind the scenes, `#[cgp_component]` generates a blanket implementation for the consumer trait, which it will automatically use to perform lookup on the tables we defined. If an entry is found and the requirements are satisfied, Rust would automatically implement the trait for us by forwarding it to the corresponding provider.

Using `delegate_component!`, we no longer need to implement the consumer traits manually on our context. Instead, we just need to specify key value pairs to map trait implementations to the providers that we have chosen for the context.

:::note
If you prefer explicit implementation over using `delegate_components!`, you can always choose to implement the consumer trait explicitly like we did earlier.

Keep in mind that `#[cgp_component]` keeps the original `CanCalculateArea` trait intact. So you can still implement the trait manually like any regular Rust trait.
:::

### No change to `scaled_area`

Now that we have turned `CanCalculateArea` into a CGP component, you might wonder: what do we need to change to use `CanCalculateArea` from `scaled_area`? And the answer is **nothing changes** and `scaled_area` stays the same as before:

```rust
#[cgp_fn]
#[uses(CanCalculateArea)]
pub fn scaled_area(&self, #[implicit] scale_factor: f64) -> f64 {
    self.area() * scale_factor * scale_factor
}
```

### Zero-cost and safe static dispatch

It is worth noting that the automatic implementation of CGP traits through `delegate_components!` are entirely safe and does not incur any runtime overhead. Behind the scene, the code generated by `delegate_components!` are *semantically equivalent* to the manual implementation of `CanCalculateArea` traits that we have shown in the earlier example.

CGP does **not** use any extra machinery like vtables to lookup the implementation at runtime - all the wirings happen only at compile time. Furthermore, the static dispatch is done entirely in **safe Rust**, and there is **no unsafe** operations like pointer casting or type erasure. When there is any missing dependency, you get a compile error immediately, and you will never need to debug any unexpected CGP error at runtime.

Furthermore, the compile-time resolution of the wiring happens *entirely within Rust's trait system*. CGP does **not** run any external compile-time processing or resolution algorithm through its macros. As a result, there is **no noticeable** compile-time performance difference between CGP code and vanilla Rust code that use plain Rust traits.

These properties are what makes CGP stands out compared to other programming frameworks. Essentially, CGP strongly follows Rust's zero-cost abstraction principles. We strive to provide the best-in-class modular programming framework that does not introduce performance overhead at both runtime and compile time. And we strive to enable highly modular code in low-level and safety critical systems, all while guaranteeing safety at compile time.

## Importing providers with `#[use_provider]`

Earlier, we have defined a general `CanCalculateArea` component that can be used by CGP functions like `scaled_area` to calculate the scaled area of any shape that contains a `scale_factor` field. But this means that if someone calls the `area` method, they would always get the unscaled version of the area.

What if we want to configure it such that shapes that contain a `scale_factor` would always apply the scale factor as `area` is called? One approach is that we could implement separate scaled area providers for each inner shape provider, such as:

```rust
#[cgp_impl(new ScaledRectangleAreaCalculator)]
#[use_provider(RectangleAreaCalculator: AreaCalculator)]
impl AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        RectangleAreaCalculator::area(self) * scale_factor * scale_factor
    }
}

#[cgp_impl(new ScaledCircleAreaCalculator)]
#[use_provider(CircleAreaCalculator: AreaCalculator)]
impl AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        CircleAreaCalculator::area(self) * scale_factor * scale_factor
    }
}
```

In the example above, we use a new `#[use_provider]` attribute provided by `#[cgp_impl]` to *import a provider* to be used within our provider implementation.

To implement the provider trait `AreaCalculator` for `ScaledRectangleAreaCalculator`, we use `#[use_provider]` to import the base `RectangleAreaCalculator`, and require it to also implement `AreaCalculator`.

Similarly, the implementation of `ScaledCircleAreaCalculator` depends on `CircleAreaCalculator` to implement `AreaCalculator`.

By importing other providers, `ScaledRectangleAreaCalculator` and `ScaledCircleAreaCalculator` can skip the need to understand what are the internal requirements for the imported providers to implement the provider traits. We can focus on just applying the `scale_factor` argument to the resulting base area, and then return the result.

We can now wire the `ScaledRectangle` and `ScaledCircle` to use the new scaled area calculator providers, while leaving `PlainRectangle` and `PlainCircle` use the base area calculators:

```rust
delegate_components! {
    PlainRectangle {
        AreaCalculatorComponent:
            RectangleAreaCalculator,
    }
}

delegate_components! {
    ScaledRectangle {
        AreaCalculatorComponent:
            ScaledRectangleAreaCalculator,
    }
}

delegate_components! {
    PlainCircle {
        AreaCalculatorComponent:
            CircleAreaCalculator,
    }
}

delegate_components! {
    ScaledCircle {
        AreaCalculatorComponent:
            ScaledCircleAreaCalculator,
    }
}
```

With that, we can write some basic tests, and verify that calling `.area()` on scaled shapes now return the scaled area:

```rust
let rectangle = PlainRectangle {
    width: 3.0,
    height: 4.0,
};

assert_eq!(rectangle.area(), 12.0);

let scaled_rectangle = ScaledRectangle {
    scale_factor: 2.0,
    width: 3.0,
    height: 4.0,
};

let circle = PlainCircle {
    radius: 3.0,
};

assert_eq!(circle.area(), 9.0 * PI);

assert_eq!(scaled_rectangle.area(), 48.0);

let scaled_circle = ScaledCircle {
    scale_factor: 2.0,
    radius: 3.0,
};

assert_eq!(scaled_circle.area(), 36.0 * PI);
```

## Higher-order providers

In the previous section, we have defined two separate providers `ScaledRectangleAreaCalculator` and `ScaledCircleAreaCalculator` to calculate the scaled area of rectangles and circles. The duplication shows the same issue as we had in the beginning with separate `scaled_rectangle` and `scaled_circle` CGP functions defined.

If we want to support scaled area *provider implementation* for all possible shapes, we'd need define a generalized `ScaledAreaCalculator` as a **higher order provider** to work with all inner `AreaCalculator` providers. This can be done as follows:

```rust
#[cgp_impl(new ScaledAreaCalculator<InnerCalculator>)]
#[use_provider(InnerCalculator: AreaCalculator)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        let base_area = InnerCalculator::area(self);

        base_area * scale_factor * scale_factor
    }
}
```

Compared to the concrete `ScaledRectangleAreaCalculator` and `ScaledCircleAreaCalculator`, the `ScaledAreaCalculator` provider contains a **generic** `InnerCalculator` parameter to denote an inner provider that would be used to perform the inner area calculation.

Aside from the generic `InnerCalculator` type, everything else in `ScaledAreaCalculator` stays the same as before. We use `#[use_provider]` to require `InnerCalculator` to implement the `AreaCalculator` provider trait, and then use it to calculate the base area before applying the scale factors.

We can now update the `ScaledRectangle` and `ScaledCircle` contexts to use the `ScaledAreaCalculator` that is composed with the respective base area calculator providers:

```rust
delegate_components! {
    ScaledRectangle {
        AreaCalculatorComponent:
            ScaledAreaCalculator<RectangleAreaCalculator>,
    }
}

delegate_components! {
    ScaledCircle {
        AreaCalculatorComponent:
            ScaledAreaCalculator<CircleAreaCalculator>,
    }
}
```

If specifying the combined providers are too mouthful, we also have the option to define **type aliases** to give the composed providers shorter names:

```rust
pub type ScaledRectangleAreaCalculator =
    ScaledAreaCalculator<RectangleAreaCalculator>;

pub type ScaledCircleAreaCalculator =
    ScaledAreaCalculator<CircleAreaCalculator>;
```

This also shows that CGP providers are just plain Rust types. By leveraging generics, we can “pass” a provider as a type argument to a higher provider to produce new providers that have the composed behaviors.

## Summary

Over the course of this tutorial series, we have worked through the full arc from plain Rust code to configurable static dispatch with CGP.

In the introduction, we identified the two fundamental limitations of conventional Rust approaches: explicit parameter threading and tight coupling between methods and concrete context structs.

In the first tutorial, we addressed those limitations with `#[cgp_fn]`, which lets us write context-generic functions that extract implicit arguments from any conforming context. We also introduced `CanCalculateArea` as a unified interface for area calculation, and showed that implementing it manually for every context introduces its own boilerplate.

In this tutorial, we resolved the remaining boilerplate using CGP components. We annotated `CanCalculateArea` with `#[cgp_component]` to generate a provider trait, defined named provider implementations with `#[cgp_impl]`, and wired them to contexts using `delegate_components!`. We then saw how `#[use_provider]` enables providers to compose with other providers, and how higher-order providers like `ScaledAreaCalculator` use Rust generics to work across all inner calculators without duplication.

Every step of this process is safe, zero-cost Rust: all wiring happens at compile time through the trait system, with no runtime overhead and no unsafe code. To continue exploring CGP, the [Hello World tutorial](../hello) offers a broader introduction to CGP’s capabilities across a wider range of features.
