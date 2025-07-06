+++

title = "Programming Extensible Data Types in Rust with CGP - Part 4: Implementing Extensible Variants"

description = ""

authors = ["Soares Chen"]

+++


# Implementation of Extensible Variants

Now that we've covered how extensible records work in CGP, we can turn our attention to **extensible variants**. At first glance, it might seem like a completely different mechanism — but surprisingly, the approach used to implement extensible variants is very similar to that of extensible records. In fact, many of the same principles apply, just in the “opposite” direction.

This close relationship between records and variants is rooted in **category theory**. In that context, records are known as **products**, while variants (or enums) are referred to as **sums** or **coproducts**. These terms highlight a deep **duality** between the two: just as products combine values, coproducts represent a choice among alternatives. CGP embraces this theoretical foundation and leverages it to create a unified design for both extensible records and extensible variants.

This duality is not just theoretical — it has practical implications for software architecture. Our design in CGP builds directly on prior research into **extensible data types**, particularly in the context of functional programming and type systems. For more on the background, see the paper on [Extensible Data Types](https://dl.acm.org/doi/10.1145/3290325), as well as this excellent [intro to category theory](https://bartoszmilewski.com/2015/01/07/products-and-coproducts/) by Bartosz Milewski.

With this in mind, we’ll now explore the CGP constructs that support extensible variants. As you go through the examples and implementation details, we encourage you to look for the parallels and contrasts with extensible records.

# Base Implementation

## `FromVariant` Trait

Just as extensible records use the `HasField` trait to extract values from a struct, extensible variants in CGP use the `FromVariant` trait to *construct* an enum from a single variant value. The trait is defined as follows:

```rust
pub trait FromVariant<Tag> {
    type Value;

    fn from_variant(_tag: PhantomData<Tag>, value: Self::Value) -> Self;
}
```

Like `HasField`, the `FromVariant` trait is parameterized by a `Tag`, which identifies the name of the variant. It also defines an associated `Value` type, representing the data associated with that variant. Unlike `HasField`, which extracts a value, `from_variant` takes in a `Value` and returns an instance of the enum.

## Example: Deriving `FromVariant`

To see how this works in practice, consider the following `Shape` enum:

```rust
#[derive(FromVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

Using the `#[derive(FromVariant)]` macro, the following trait implementations will be automatically generated:

```rust
impl FromVariant<symbol!("Circle")> for Shape {
    type Value = Circle;

    fn from_variant(_tag: PhantomData<symbol!("Circle")>, value: Self::Value) -> Self {
        Shape::Circle(value)
    }
}

impl FromVariant<symbol!("Rectangle")> for Shape {
    type Value = Rectangle;

    fn from_variant(_tag: PhantomData<symbol!("Rectangle")>, value: Self::Value) -> Self {
        Shape::Rectangle(value)
    }
}
```

This allows the `Shape` enum to be constructed generically using just the tag and value.

## Limitations on Enum Shape

To ensure ergonomics and consistency, CGP restricts the kinds of enums that can derive `FromVariant`. Specifically, supported enums must follow the **sums of products** pattern—each variant must contain *exactly one unnamed field*.

The following forms, for example, are **not** supported:

```rust
pub enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

pub enum Shape {
    Circle {
        radius: f64,
    },
    Rectangle {
        width: f64,
        height: f64,
    },
}
```

These more complex variants are not supported because they would make it harder to represent variant fields as simple types, which would, in turn, lead to less ergonomic APIs. By limiting each variant to a single unnamed field, CGP ensures that types like `FromVariant::Value` remain straightforward and intuitive.

If you need to represent more complex data in a variant, we recommend wrapping that data in a dedicated struct. This way, you can still take advantage of CGP's extensible variant system while maintaining type clarity and composability.

## Partial Variants

Just as CGP supports partially constructed structs through *partial records*, it also enables *partial variants* to work with enums in a similarly flexible way. Partial variants allow you to pattern match on each variant of an enum incrementally, while safely excluding any variants that have already been handled. This makes it possible to build exhaustive and type-safe match chains that evolve over time.

Consider the `Shape` enum we explored earlier. CGP would generate a corresponding `PartialShape` enum that represents the partial variant form of `Shape`:

```rust
pub enum PartialShape<F0: MapType, F1: MapType> {
    Circle(F0::Map<Circle>),
    Rectangle(F1::Map<Rectangle>),
}
```

## `HasExtractor` trait

To enable the transformation from a regular enum into its partial variant form, CGP provides the `HasExtractor` trait. This trait defines an associated type named `Extractor`, which represents the full set of partial variants for a given enum, and a method `to_extractor`, which performs the conversion:

```rust
pub trait HasExtractor {
    type Extractor;

    fn to_extractor(self) -> Self::Extractor;
}
```

For the `Shape` enum, an implementation of `HasExtractor` would look like the following:

```rust
impl HasExtractor for Shape {
    type Extractor = PartialShape<IsPresent, IsPresent>;

    fn to_extractor(self) -> Self::Extractor {
        match self {
            Shape::Circle(circle) => PartialShape::Circle(circle),
            Shape::Rectangle(rectangle) => PartialShape::Rectangle(rectangle),
        }
    }
}
```

This implementation makes it possible to work with a `Shape` value as a `PartialShape`, where each variant is wrapped in an `IsPresent` marker, indicating that the variant is still available to be matched.

## `IsVoid` Type Mapper

The key distinction between partial records and partial variants lies in how we represent the absence of data. For partial variants, CGP introduces the `IsVoid` type mapper to indicate that a variant has already been extracted and is no longer available:

```rust
pub enum Void {}
pub struct IsVoid;

impl MapType for IsVoid {
    type Map<T> = Void;
}
```

The `Void` type is defined as an empty enum with no variants. This means that it is impossible to construct a value of type `Void`, and any code that attempts to match on a `Void` value will be statically unreachable. This makes it a safe and expressive way to model a variant that no longer exists in a given context.

Conceptually, `Void` serves the same purpose as Rust’s built-in [**never type**](https://doc.rust-lang.org/reference/types/never.html) or the [`Infallible`](https://doc.rust-lang.org/std/convert/enum.Infallible.html) type from the standard library. However, CGP defines `Void` explicitly to distinguish its special role in the context of extensible variants.

While `IsNothing` is used for absent fields in partial records, we use `IsVoid` to represent removed or matched variants in partial enums. This ensures that once a variant has been extracted, it cannot be matched again — preserving both soundness and safety in CGP’s type-driven pattern matching.

## `ExtractField` Trait

Once an enum has been converted into its partial variant form, we can begin incrementally pattern matching on each variant using the `ExtractField` trait. This trait enables safe, step-by-step extraction of variant values, and is defined as follows:

```rust
pub trait ExtractField<Tag> {
    type Value;
    type Remainder;

    fn extract_field(self, _tag: PhantomData<Tag>) -> Result<Self::Value, Self::Remainder>;
}
```

Just like `FromVariant` and `HasField`, the `ExtractField` trait takes a `Tag` type to identify the variant, and includes an associated `Value` type representing the variant’s inner data. Additionally, it defines a `Remainder` type, which represents the remaining variants that have not yet been matched.

The `extract_field` method consumes the value and returns a `Result`, where a successful match yields the extracted `Value`, and a failed match returns the `Remainder`. Although this uses the `Result` type, the `Err` case is not really an error in the traditional sense — rather, it represents the remaining variants yet to be handled, much like how errors represent alternative outcomes in Rust.

### Example Implementation of `ExtractField`

To understand how `ExtractField` works in practice, let’s look at an implementation for extracting the `Circle` variant from a `PartialShape`:

```rust
impl<F1: MapType> ExtractField<symbol!("Circle")> for PartialShape<IsPresent, F1> {
    type Value = Circle;
    type Remainder = PartialShape<IsVoid, F1>;

    fn extract_field(self, _tag: PhantomData<symbol!("Circle")>) ->
        Result<Self::Value, Self::Remainder>
    {
        match self {
            PartialShape::Circle(circle) => Ok(circle),
            PartialShape::Rectangle(rectangle) => Err(PartialShape::Rectangle(rectangle))
        }
    }
}
```

In this implementation, we are working with a `PartialShape` in which the `Circle` variant is still marked as present. The trait is also generic over `F1: MapType`, which corresponds to the `Rectangle` variant, allowing the code to remain flexible regardless of whether the rectangle has already been extracted or not.

The associated `Remainder` type updates the `Circle` variant from `IsPresent` to `IsVoid`, signifying that it has been extracted and should no longer be considered valid. The use of the `Void` type ensures that this variant cannot be constructed again, making it safe to ignore in further matches.

Within the method body, we match on `self`. If the value is a `Circle`, we return it in the `Ok` case. Otherwise, we return the remaining `PartialShape`, reconstructing it with the other variant. Due to the type system’s enforcement, it is impossible to incorrectly return a `Circle` as part of the remainder once it has been marked as `IsVoid`. The compiler ensures that this branch is unreachable, preserving correctness by construction.

### Example Use of `ExtractField`

With `ExtractField`, we can now incrementally extract and match against variants in a safe and ergonomic way. Here’s an example of computing the area of a shape using this approach:

```rust
let shape = Shape::Circle(Circle { radius: 5.0 });

let area = match shape
    .to_extractor() // PartialShape<IsPresent, IsPresent>
    .extract_field(PhantomData::<symbol!("Circle")>)
{
    Ok(circle) => PI * circle.radius * circle.radius,
    // PartialShape<IsVoid, IsPresent>
    Err(remainder) => match remainder.extract_field(PhantomData::<symbol!("Rectangle")>) {
        Ok(rectangle) => rectangle.width * rectangle.height,
        // PartialShape<IsVoid, IsVoid>
        // No need to match on `Err`
    },
};
```

In this example, we begin by converting the `Shape` value into a `PartialShape` with all variants present using `to_extractor`. We then call `extract_field` to try extracting the `Circle` variant. If successful, we compute the circle's area. If not, we receive a remainder value where the `Circle` variant is now marked as `IsVoid`. This remainder is then used to attempt extracting the `Rectangle` variant. If that succeeds, we compute the area accordingly.

By the time we reach the second `Err` case, the remainder has the type `PartialShape<IsVoid, IsVoid>`, which cannot contain any valid variant. Because of this, we can safely omit any further pattern matching, and the compiler guarantees that there are no unreachable or unhandled cases.

What makes this approach so powerful is that the Rust type system can statically verify that it is impossible to construct a valid value for `PartialShape<IsVoid, IsVoid>`. We no longer need to write boilerplate `_ => unreachable!()` code or use runtime assertions. The type system ensures exhaustiveness and soundness entirely at compile time, enabling safer and more maintainable implementation of extensible variants.

# Conclusion
