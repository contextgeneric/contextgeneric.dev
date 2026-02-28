# Area Calculation Tutorial

In this tutorial series, we will explore CGP — Context-Generic Programming — through a concrete, hands-on example: computing the area of different shapes. We will start with familiar plain Rust code, identify its limitations, and progressively introduce the CGP tools that address them. No prior knowledge of CGP is required, though a basic familiarity with Rust traits will be helpful.

## Plain Functions and Explicit Dependencies

To make the walkthrough approachable to Rust programmers of all programming levels, we will use a simple use case of calculating the area of different shape types. For example, if we want to calculate the area of a rectangle, we might write a `rectangle_area` function as follows:

```rust
pub fn rectangle_area(width: f64, height: f64) -> f64 {
    width * height
}
```

The `rectangle_area` function accepts two explicit arguments `width` and `height`, which is not too tedious to pass around with. The implementation body is also intentionally trivial, so that this tutorial can remain comprehensible. But in real world applications, a plain Rust function may need to work with many more parameters to implement complex functionalities, and their function body may be significantly more complex.

Furthermore, we may want to implement other functions that call the `rectangle_area` function, and perform additional calculation based on the returned value. For example, suppose that we want to calculate the area of a rectangle value that contains an additional *scale factor*, we may want to write a `scaled_rectangle_area` function such as follows:

```rust
pub fn scaled_rectangle_area(
    width: f64,
    height: f64,
    scale_factor: f64,
) -> f64 {
    rectangle_area(width, height) * scale_factor * scale_factor
}
```

As we can see, the `scaled_rectangle_area` function mainly works with the `scale_factor` argument, but it needs to also accept `width` and `height` and explicitly pass the arguments to `rectangle_area`. (we will pretend that the implementation of `rectangle_area` is complex, so that it is not feasible to inline the implementation here)

This simple example use case demonstrates the problems that arise when dependencies need to be threaded through plain functions by the callers. Even with this simple example, the need for three parameters start to become slightly tedious. And things would become much worse for real world applications.

## Concrete Context Methods

Since passing function arguments explicitly can quickly get out of hand, in Rust we typically define *context types* that group dependencies into a single struct entity to manage the parameters more efficiently.

For example, we might define a `Rectangle` context and re-implement `rectangle_area` and `scaled_rectangle_area` as *methods* on the context:

```rust
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

impl Rectangle {
    pub fn rectangle_area(&self) -> f64 {
        self.width * self.height
    }

    pub fn scaled_rectangle_area(&self) -> f64 {
        self.rectangle_area() * self.scale_factor * self.scale_factor
    }
}
```

With a unified context, the method signatures of `rectangle_area` and `scaled_rectangle_area` become significantly cleaner. They both only need to accept a `&self` parameter. `scaled_rectangle_area` also no longer needs to know which fields are accessed by `rectangle_area`. All it needs to do is call `self.rectangle_area()`, and then apply the `scale_factor` field to the result.

The use of a common `Rectangle` context struct can result in cleaner method signatures, but it also introduces *tight coupling* between the individual methods and the context. As the application grows, the context type may become increasingly complex, and simple functions like `rectangle_area` would become increasingly coupled with unrelated dependencies.

For example, perhaps the application may need to assign *colors* to individual rectangles, or track their positions in a 2D space. So the `Rectangle` type may grow to become something like:

```rust
pub struct ComplexRectangle {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
    pub color: Color,
    pub pos_x: f64,
    pub pos_y: f64,
}
```

As the context grows, it becomes significantly more tedious to call a method like `rectangle_area`, even if we don't care about using other methods. We would still need to first construct a `ComplexRectangle` with most of the fields having default value, before we can call `rectangle_area`.

Furthermore, a concrete context definition also limits how it can be extended. Suppose that a third party application now wants to use the provided methods like `scaled_rectangle_area`, but also wants to store the rectangles in a *3D space*, it would be tough to ask the upstream project to introduce a new `pos_z` field, which can potentially break many existing code. In the worst case, the last resort for extending the context is to fork the entire project to make the changes.

Ideally, what we really want is to have some ways to pass around the fields in a context *implicitly* to functions like `rectangle_area` and `scaled_rectangle_area`. As long as a context type contains the required fields, e.g. `width` and `height`, we should be able to call `rectangle_area` on it without needing to implement it for the specific context.

## Next Steps

We have now identified the two core limitations of conventional Rust approaches: explicit parameter threading becomes unwieldy as the call stack grows deeper, and concrete context methods create tight coupling between implementations and a specific struct.

In the first tutorial, Context-Generic Functions, we will see how the `#[cgp_fn]` macro and `#[implicit]` arguments address both of these limitations at once, allowing us to write a single `rectangle_area` function that works cleanly across any context that provides the required fields. We will also explore how CGP functions can import each other via `#[uses]`, and take an optional look at how the macro desugars into plain Rust traits under the hood.

In the second tutorial, Static Dispatch, we will introduce a second shape — the circle — and define a unified `CanCalculateArea` trait as a common interface across all shapes. We will run into Rust's coherence restrictions when trying to provide blanket implementations, and then resolve this with CGP's `#[cgp_component]` macro and named providers. Finally, we will see how `delegate_components!` wires contexts to providers at compile time, and how higher-order providers allow provider implementations to compose generically, with zero runtime overhead.