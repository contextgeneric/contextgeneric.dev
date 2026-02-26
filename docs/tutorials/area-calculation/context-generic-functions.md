---
sidebar_position: 1
---

# Context-Generic Functions

In the previous part of this tutorial, we identified two problems with plain Rust code: explicit function parameters accumulate quickly as call chains grow longer, and grouping fields into a concrete context struct creates tight coupling between implementations and a specific type. In this tutorial, we will address both of these problems at once using `#[cgp_fn]` — CGP’s mechanism for defining functions that accept **implicit arguments** extracted automatically from any conforming context.

By the end of this tutorial, we will have defined `rectangle_area`, `scaled_rectangle_area`, and `circle_area` as context-generic functions, tested them on multiple context types, and introduced the `CanCalculateArea` trait as a unified interface for area calculation across different shapes.

## Introducing `#[cgp_fn]` and `#[implicit]` arguments

CGP v0.6.2 introduces a new `#[cgp_fn]` macro, which we can apply to plain Rust functions and turn them into *context-generic* methods that accept *implicit arguments*. With that, we can rewrite the example `rectangle_area` function as follows:

```rust
#[cgp_fn]
pub fn rectangle_area(
    &self,
    #[implicit] width: f64,
    #[implicit] height: f64,
) -> f64 {
    width * height
}
```

Compared to before, our `rectangle_area` function contains a few extra constructs:

- `#[cgp_fn]` is used to augment the plain function.
- `&self` is given to access a reference to a *generic context* value.
- `#[implicit]` is applied to both `width` and `height`, indicating that the arguments will be automatically extracted from `&self`.

Aside from these extra annotations, the way we define `rectangle_area` remains largely the same as how we would define it previously as a plain Rust function.

With the CGP function defined, let's define a minimal `PlainRectangle` context type and test calling `rectangle_area` on it:

```rust
#[derive(HasField)]
pub struct PlainRectangle {
    pub width: f64,
    pub height: f64,
}
```

To enable context-generic capabilities on a context, we first need to apply `#[derive(HasField)]` on `PlainRectangle` to generate generic field access implementations. After that, we can just call `rectangle_area` on it:

```rust
let rectangle = PlainRectangle {
    width: 2.0,
    height: 3.0,
};

let area = rectangle.rectangle_area();
assert_eq!(area, 6.0);
```

And that's it! CGP implements all the heavyweight machinery behind the scene using Rust's trait system. But you don't have to understand any of that to start using `#[cgp_fn]`.

### Importing other CGP functions with `#[uses]`

Now that we have defined `rectangle_area` as a context-generic function, let's take a look at how to also define `scaled_rectangle_area` and call `rectangle_area` from it:

```rust
#[cgp_fn]
#[uses(RectangleArea)]
pub fn scaled_rectangle_area(
    &self,
    #[implicit] scale_factor: f64,
) -> f64 {
    self.rectangle_area() * scale_factor * scale_factor
}
```

Compared to `rectangle_area`, the implementation of `scaled_rectangle_area` contains an additional `#[uses(RectangleArea)]` attribute, which is used for us to "import" the capability to call `self.rectangle_area()`. The import identifier is in CamelCase, because `#[cgp_fn]` converts a function like `rectangle_area` into a *trait* called `RectangleArea`.

In the argument, we can also see that we only need to specify an implicit `scale_factor` argument. In general, there is no need for us to know which capabilities are required by an imported construct like `RectangleArea`. That is, we can just define `scaled_rectangle_area` without knowing the internal details of `rectangle_area`.

With `scaled_rectangle_area` defined, we can now define a *second* `ScaledRectangle` context that contains both the rectangle fields and the `scale_factor` field:

```rust
#[derive(HasField)]
pub struct ScaledRectangle {
    pub scale_factor: f64,
    pub width: f64,
    pub height: f64,
}
```

Similar to `PlainRectangle`, we only need to apply `#[derive(HasField)]` on it, and now we can call both `rectangle_area` and `scaled_rectangle_area` on it:

```rust
let rectangle = ScaledRectangle {
    scale_factor: 2.0,
    width: 3.0,
    height: 4.0,
};

let area = rectangle.rectangle_area();
assert_eq!(area, 12.0);

let scaled_area = rectangle.scaled_rectangle_area();
assert_eq!(scaled_area, 48.0);
```

It is also worth noting that there is no need for us to modify `PlainRectangle` to add a `scale_factor` on it. Instead, both `PlainRectangle` and `ScaledRectangle` can **co-exist** in separate locations, and all CGP constructs with satisfied requirements will work transparently on all contexts.

This means that we can still call `rectangle_area` on both `PlainRectangle` and `ScaledRectangle`. But we can call `scaled_rectangle_area` only on `ScaledRectangle`, since `PlainRectangle` lacks a `scale_factor` field.

### Using `#[cgp_fn]` without `#[implicit]` arguments

Even though `#[cgp_fn]` provides a way for us to use implicit arguments, it is not the only reason why we'd use it over plain Rust functions. The other reason to use `#[cgp_fn]` is to write functions that can call other CGP functions.

As an example, suppose that we want to write a helper function to print the rectangle area. A naive approach would be to define this as a method on a concrete context like `PlainRectangle`:

```rust
impl PlainRectangle {
    pub fn print_rectangle_area(&self) {
        println!("The area of the rectangle is {}", self.rectangle_area());
    }
}
```

This works, but if we also want to use `print_scaled_rectangle_area` on another context like `ScaledRectangle`, we would have to rewrite the same method on it:

```rust
impl ScaledRectangle {
    pub fn print_rectangle_area(&self) {
        println!("The area of the rectangle is {}", self.rectangle_area());
    }
}
```

One way we can avoid this boilerplate is to use `#[cgp_fn]` and `#[uses]` to import `RectangleArea`, and then print out the value:

```rust
#[cgp_fn]
#[uses(RectangleArea)]
pub fn print_rectangle_area(&self) {
    println!("The area of the rectangle is {}", self.rectangle_area());
}
```

This way, `print_rectangle_area` would automatically implemented on any context type where `rectangle_area` is also automatically implemented.

## How it works

*This section explores the internals of `#[cgp_fn]` and is supplementary to the tutorial. If you are comfortable with what you have built so far and would like to continue to the next concepts, feel free to skip ahead — a detailed understanding of these mechanics is not required to use CGP functions effectively.*

Now that we have gotten a taste of the power unlocked by `#[cgp_fn]`, let's take a sneak peek of how it works under the hood. Behind the scene, a CGP function like `rectangle_area` is roughly desugared to the following plain Rust code:

```rust
pub trait RectangleArea {
    fn rectangle_area(&self) -> f64;
}

pub trait RectangleFields {
    fn width(&self) -> &f64;

    fn height(&self) -> &f64;
}

impl<Context> RectangleArea for Context
where
    Self: RectangleFields,
{
    fn rectangle_area(&self) -> f64 {
        let width = self.width().clone();
        let height = self.height().clone();

        width * height
    }
}
```

As we can see from the desugared code, there are actually very little magic happening within the `#[cgp_fn]` macro. Instead, the macro mainly acts as **syntactic sugar** to turn the function into the plain Rust constructs we see above.

First, a `RectangleArea` trait is defined with the CamelCase name derived from the function name. The trait contains similar function signature as `rectangle_area`, except that the implicit arguments are removed from the interface.

Secondly, a *getter trait* that resembles the `RectangleFields` above is used to access the `width` and `height` fields of a generic context.

Finally, a [**blanket implementation**](https://blog.implrust.com/posts/2025/09/blanket-implementation-in-rust/) of `RectangleArea` is defined to work with any `Context` type that contains both the `width` and `height` fields. This means that there is no need for any context type to implement `RectangleArea` manually.

Inside the function body, the macro desugars the implicit arguments into local `let` bindings that calls the getter methods and bind the field values to local variables. After that, the remaining function body follows the original function definition.

:::note

### Borrowed vs owned implicit arguments

The `width()` and and `height()` methods on `RectangleFields` return a borrowed `&f64`. This is because all field access are by default done through borrowing the field value from `&self`. However, when the implicit argument is an *owned value*, CGP will automatically call `.clone()` on the field value and require that the `Clone` bound of the type is satisfied.

We can rewrite the `rectangle_area` to accept the implicit `width` and `height` arguments as *borrowed* references, such as:

```rust
#[cgp_fn]
pub fn rectangle_area(
    &self,
    #[implicit] width: &f64,
    #[implicit] height: &f64,
) -> f64 {
    (*width) * (*height)
}
```

This way, the field access of the implicit arguments will be **zero copy** and not involve any cloning of values. It is just that in this case, we still need to dereference the `&f64` values to perform multiplication on them. And since `f64` can be cloned cheaply, we just opt for implicitly cloning the arguments to become owned values.

:::

To make `RectangleArea` automatically implemented for a context like `PlainRectangle`, the `#[derive(HasField)]` macro generates getter trait implementations that are equivalent to follows:

```rust
impl RectangleFields for PlainRectangle {
    fn width(&self) -> &f64 {
        &self.width
    }

    fn height(&self) -> &f64 {
        &self.height
    }
}
```

With the getter traits implemented, the requirements for the blanket implementation of `RectangleArea` is satisfied. And thus we can now call call `rectangle_area()` on a `PlainRectangle` value.

### Zero cost field access

The plain Rust expansion demonstrates a few key properties of CGP. Firstly, CGP makes heavy use of the existing machinery provided by Rust's trait system to implement context-generic abstractions. It is also worth understanding that CGP macros like `#[cgp_fn]` and `#[derive(HasField)]` mainly act as **syntactic sugar** that perform simple desugaring of CGP code into plain Rust constructs like we shown above.

This means that there is **no hidden logic at both compile time and runtime** used by CGP to resolve dependencies like `width` and `height`. The main complexity of CGP lies in how it introduces new language syntax and leverages Rust's trait system to enable new language features. But you don't need to understand new machinery beyond the trait system to understand how CGP works.

Furthermore, implicit arguments like `#[implicit] width: f64` are automatically desugared by CGP to use getter traits similar to `RectangleFields`. And contexts like `PlainRectangle` implement `RectangleFields` by simply returning the field value. This means that implicit argument access are **zero cost** and are as cheap as direct field access from a concrete context.

The important takeaway from this is that CGP follows the same **zero cost abstraction** philosophy of Rust, and enables us to write highly modular Rust programs without any runtime overhead.

### Auto getter fields

When we walk through the desugared Rust code, you might wonder: since `RectangleArea` requires the context to implement `RectangleFields`, does this means that a context type like `PlainRectangle` must know about it beforehand and explicitly implement `RectangleFields` before we can use `RectangleArea` on it?

The answer is yes for the simplified desugared code that we have shown earlier. But CGP actually employs a more generalized trait called `HasField` that can work generally for all possible structs. This means that there is **no need** to specifically generate a `RectangleFields` trait to be used by `RectangleArea`, or implemented by `PlainRectangle`.

The full explanation of how `HasField` works is beyond the scope of this tutorial. But the general idea is that an instance of `HasField` is implemented for every field inside a struct that uses `#[derive(HasField)]`. This is then used by implementations like `RectangleArea` to access a specific field by its field name.

In practice, this means that both `RectangleArea` and `PlainRectangle` can be defined in totally different crate without knowing each other. They can then be imported inside a third crate, and `RectangleArea` would still be automatically implemented for `PlainRectangle`.

### Comparison to Scala implicit parameters

### Desugaring `scaled_rectangle_area`

Similar to `rectangle_area`, the desugaring of `scaled_rectangle_area` follows the same process:

```rust
pub trait ScaledRectangleArea {
    fn scaled_rectangle_area(&self) -> f64;
}

pub trait ScaleFactorField {
    fn scale_factor(&self) -> &f64;
}

impl<Context> ScaledRectangleArea for Context
where
    Self: RectangleArea + ScaleFactorField,
{
    fn scaled_rectangle_area(&self) -> f64 {
        let scale_factor = self.scale_factor().clone();

        self.rectangle_area() * scale_factor * scale_factor
    }
}
```

Compared to `rectangle_area`, the desugared code for `scaled_rectangle_area` contains an additional trait bound `Self: RectangleArea`, which is generated from the `#[uses(RectangleArea)]` attribute. This also shows that importing a CGP construct is equivalent to applying it as a trait bound on `Self`.

It is also worth noting that trait bounds like `RectangleField` only appear in the `impl` block but not on the trait definition. This implies that they are *impl-side dependencies* that hide the dependencies behind a trait impl without revealing it in the trait interface.

Aside from that, `ScaledRectangleArea` also depends on field access traits that are equivalent to `ScaleFactorField` to retrieve the `scale_factor` field from the context. In actual, it also uses `HasField` to retrieve the `scale_factor` field value, and there is no extra getter trait generated.

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

## Summary

In this tutorial, we introduced `#[cgp_fn]` and defined several context-generic functions: `rectangle_area`, `scaled_rectangle_area`, `circle_area`, and `scaled_area`. We saw how `#[implicit]` arguments are automatically extracted from any conforming context, and how `#[uses]` lets one CGP function call another without threading dependencies manually. We also expanded the example to cover multiple shape types and introduced `CanCalculateArea` as a unified interface — though implementing it for each context still requires a separate `impl` block.

In the next tutorial, [Configurable Static Dispatch](static-dispatch), we will see how CGP’s component system eliminates this boilerplate. Instead of writing explicit trait implementations for every context, we will use `#[cgp_component]` and `delegate_components!` to configure the wiring between contexts and providers in a concise, table-driven way.
