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

Just as we used *partial records* to support partially *constructed* structs, CGP also defines **partial variants** to support partially **extracted** enums. Partial variants allow us to pattern match on each variant in an enum one at a time, while safely ruling out cases that have been *previously matched* before.

For example, the `Shape` example earlier would have a `PartialShape` enum defined as its partial variants:

```rust
pub enum PartialShape<F0: MapType, F1: MapType> {
    Circle(F0::Map<Circle>),
    Rectangle(F1::Map<Rectangle>),
}
```

## `HasExtractor` trait

To support the conversion from an enum to its partial variants, CGP provides the `HasExtractor` trait, which is defined as follows:

```rust
pub trait HasExtractor {
    type Extractor;

    fn to_extractor(self) -> Self::Extractor;
}
```

The `HasExtractor` trait contains an `Extractor` associated type representing the partial variants with all cases present. It also provides a `to_extractor` method to convert the enum into its partial variants.

Following is the example implementation of `HasExtractor` for the `Shape` enum:

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

## `IsVoid` Type Mapper

The main difference between partial records and partial variants is that we use the `IsVoid` type mapper to represent an *extracted variant*:

```rust
pub enum Void {}
pub struct IsVoid;

impl MapType for IsVoid {
    type Map<T> = Void;
}
```

The `Void` type is simply defined as an enum with *zero* variant. As a result, `Void` can *never be constructed*, since there is no valid variant to construct it from. It also means that if we have a value of type `Void`, we can use `match value {}` to match it to *anything* we want, since there is no valid case to match in the `match` body.

The `Void` type is conceptually equivalent to the [**never type**](https://doc.rust-lang.org/reference/types/never.html) or [`Infallible`](https://doc.rust-lang.org/std/convert/enum.Infallible.html) in Rust. We mainly define it as a distinct type to distinguish its special use in CGP.

We use `IsVoid` instead of `IsNothing` to represent the **absence** of a variant, as it means that a valid value can never be found for that variant.

## `ExtractField` Trait

Once we convert an enum to its partial variants, we can perform *partial* pattern matching on it, one variant at a time, through the `ExtractField` trait, which is defined as follows:

```rust
pub trait ExtractField<Tag> {
    type Value;

    type Remainder;

    fn extract_field(self, _tag: PhantomData<Tag>) -> Result<Self::Value, Self::Remainder>;
}
```

Similar to `FromVariant` and `HasField`, `ExtractField` has a `Tag` type for the variant name, and a `Value` associated type for the variant's type. Additionally, there is a `Remainder` associated type representing the *remaining* of the partial variants that have not yet been matched on.

The `extract_field` method takes a `self` value, and returns a `Result` of *either* the `Value` when the variant is matched, or the `Remainder` when the match fails.

It is worth noting that even though we used `Result` here, the `Remainder` is not really an error, but rather the remaining parts that need to be handled similar to how we handle errors.

### Example Implementation of `ExtractField`

To understand how `ExtractField` works, let's look at the implementation of `ExtractField` for the `Circle` variant of `Shape`:

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

The `ExtractField` implementation works with a `PartialShape`, with `IsPresent` used on indicate that we can extract the `Circle` variant from `PartialShape` if it is not yet extracted.

Similar to the implementation of `BuildField`, the `ExtractField` implementation is parameterized by `F1: MapType` for the `Rectangle` variant, to indicate that our code works regardless of whether `Rectangle` has previously been extracted or not.

The `Remainder` type of the implementation updates the `Circle` variant status from `IsPresent` to `IsVoid`, to indicate that the `Circle` variant has been extracted. The `Void` type makes it no longer possible to construct the `Circle` variant, thus making it an "invalid" case that can be skipped later.

In the `extract_field` method body, we perform a `match` on `self`. If the `Circle` variant is matched, we simply return `Ok(circle)` to indicate a successful extraction. For every other cases like `Rectangle`, we return an `Err` case with the original variant reconstructed.

It is worth noting that due to the type safety of `Void`, it is not possible to safely implement `extract_field` in any other way. For example, we cannot accidentally return the `Circle` variant back as an `Err` case, because it is impossible to construct the variant with a value of type `Void`.

### Example Use of `ExtractField`

Using `ExtractField`, we can now extract the variants from `Shape` in multiple steps, and perform incremental matching such as follows:

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

The above code shows an example of how to calculate the area of a shape through `ExtractField`. We first call `to_extractor` to convert the shape into a `PartialShape` with all variants present. We then call `extract_field` with the `Circle` variant, and calculate the circle's area in the success case `Ok`.

In case the matching failed, we get the remainder in the `Err` case, with the remainder partial variants having the type `PartialShape<IsVoid, IsPresent>`. We then call `extract_field` on the remainder again with the `Rectangle` variant, and once again match against the result.

This time, for the `Ok` case, we get a `Rectangle` value and use it to calculate the rectangle's area. For the `Err` case, the remainder this time has the type `PartialShape<IsVoid, IsVoid>`, indicating that all possible variants have been extracted. Since the type does not contain any valid value, we can safely *omit the entire branch* and the code will still compile.

As we can see, the Rust compiler is smart enough to know that a type like `PartialShape<IsVoid, IsVoid>` cannot have any valid value, and allows us to skip ahead without having to manually prove that each variant is in fact impossible. Thanks to this, we are able to leverage the type system to safely diverge after all possible variants are ruled out, without needing runtime exceptions to assert that the case cannot be reached.

# Conclusion
