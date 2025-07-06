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

## `FinalizeExtract` Trait

While Rust’s type system can infer that a type like `PartialShape<IsVoid, IsVoid>` is uninhabitable, this inference only works when the compiler has access to the fully concrete type. To support this behavior more generically within CGP’s extensible variant system, the `FinalizeExtract` trait is introduced. This trait provides a mechanism to *discharge* an empty partial variant after all possible cases have been matched:

```rust
pub trait FinalizeExtract {
    fn finalize_extract<T>(self) -> T;
}
```

At first glance, the `finalize_extract` method might appear misleading. It accepts a `self` value and claims to return a value of *any* type `T`. This may seem unsound, but the key detail is that it is only ever implemented for types that are *uninhabited* — in other words, types that can never actually exist at runtime. Examples include `Void` and a fully exhausted partial variant like `PartialShape<IsVoid, IsVoid>`.

The implementation is both safe and surprisingly simple:

```rust
impl FinalizeExtract for PartialShape<IsVoid, IsVoid> {
    fn finalize_extract<T>(self) -> T {
        match self {}
    }
}
```

Here, we use an empty `match` expression on `self`, which works because the compiler knows that `PartialShape<IsVoid, IsVoid>` has no possible value. Since it is impossible to construct such a value, the match is guaranteed to be unreachable. Rust verifies this at compile time, ensuring both safety and correctness.

By leveraging the `Void` type in this way, CGP allows us to exhaustively extract every variant from a partial enum and confidently conclude that no cases remain. This eliminates the need for runtime assertions, unreachable branches, or panics. Instead, the type system itself guarantees that all variants have been handled, enabling a clean and fully type-safe approach to enum decomposition.

# Implementation of Casts

With the basic traits for extensible variants implemented, we will next look at how to implement the `CanUpcast` and `CanDowncast` traits to support upcasting and downcasting between enums with compatible variants.

## `HasFields` Implementation

Similar to the record merging operation in extensible records, we need to also implement `HasFields` for enums, so that the our generic implementation can iterate over each variant in the enum.

The `HasFields` implementation for our example `Shape` is as follows:

```rust
impl HasFields for Shape {
    type Fields = Sum![
        Field<symbol!("Circle"), Circle>,
        Field<symbol!("Rectangle"), Rectangle>,
    ];
}
```

Instead of using the `Product!` macro to construct a type-level list, we use the `Sum!` macro to construct a *type-level sum* of all variants in the enum. The `Sum!` macro desugars to follows:

```rust
impl HasFields for Shape {
    type Fields = Either<
        Field<symbol!("Circle"), Circle>,
        Either<
            Field<symbol!("Rectangle"), Rectangle>,
            Void,
        >,
    >;
}
```

While `Product!` desugars to a chain of `Cons` that ends with `Nil`, `Sum!` desugars to a chain of `Either` that ends with `Void`. The `Either` type is defined to be structurally the same as `Result`, just with different variant names:

```rust
pub enum Either<A, B> {
    Left(A),
    Right(B),
}
```

## `CanUpcast` Trait

With the `HasFields` implementation for enums, we can now implement `CanUpcast` as follows:

```rust
pub trait CanUpcast<Target> {
    fn upcast(self, _tag: PhantomData<Target>) -> Target;
}
```

The `CanUpcast` trait is parameterized by a `Target` type that we want to upcast to. The `upcast` method takes in `self` and converts it to the `Target` value, and the extra `PhantomData` parameter is used to help the compiler infer the `Target` type.

The trait is automatically implemented with the following blanket implementation:

```rust
impl<Context, Source, Target, Remainder> CanUpcast<Target> for Context
where
    Context: HasFields + HasExtractor<Extractor = Source>,
    Context::Fields: FieldsExtractor<Source, Target, Remainder = Remainder>,
    Remainder: FinalizeExtract,
{
    fn upcast(self, _tag: PhantomData<Target>) -> Target {
        match Context::Fields::extract_from(self.to_extractor()) {
            Ok(target) => target,
            Err(remainder) => remainder.finalize_extract(),
        }
    }
}
```

The way `CanUpcast` works is as follows. It first requires the source `Context` type to implement `HasFields` and `HasExtractor`. It then require the context `Fields` to implement a helper trait `FieldsExtractor`, which performs the actual extraction of variants from the source partial variants to the `Target` type. Finally, it requires the `Remainder` returned from the field extraction operation to implement `FinalizeExtract`, to ensure that all variants in the source enum are exhaustively extracted.

In the body for `upcast`, we first call `self.extractor()` to convert the source enum to its partial variants. We then use `Fields::extract_from` to extract the variants to the target enum. Finally, we handle the remainder case by calling `finalize_extract()`, as we expect there to be no remaining variant left in the source after extraction has completed.

## `FieldsExtractor` Trait

The helper `FieldsExtractor` trait is defined as follows:

```rust
pub trait FieldsExtractor<Source, Target> {
    type Remainder;

    fn extract_from(source: Source) -> Result<Target, Self::Remainder>;
}
```

The trait is parameterized by a `Source` type, which is the partial variants for the source enum, and a `Target` type which is the *non-partial* target enum. It has a `Remainder` type to represent any remaining variant that has not been extracted.

The `extract_from` method accepts a `Source` partial variants, and conditionally either returns the `Target` if the extraction is successful, or the `Remainder` for any unextracted variant that remains in `Source`.

We then have a blanket implementation of `FieldsExtractor` for the head of the `Sum!` fields:

```rust
impl<Source, Target, Tag, Value, RestFields, Remainder> FieldsExtractor<Source, Target>
    for Either<Field<Tag, Value>, RestFields>
where
    Source: ExtractField<Tag, Value = Value>,
    Target: FromVariant<Tag, Value = Value>,
    RestFields: FieldsExtractor<Source::Remainder, Target, Remainder = Remainder>,
{
    type Remainder = Remainder;

    fn extract_from(source: Source) -> Result<Target, Remainder> {
        match source.extract_field(PhantomData) {
            Ok(field) => Ok(Target::from_variant(PhantomData, field)),
            Err(remainder) => RestFields::extract_from(remainder),
        }
    }
}
```

We first pattern match the head of the sum as `Field<Tag, Value>`. We then require that the `Source` partial variants to implement `ExtractField<Tag>`, and the `Target` enum to implement `FromVariant<Tag>`. Additionally, the `Value` type for both `ExtractField` and `FromVariant` must be the same as `Value`.

The implementation then requires the remaining fields to also implement `FieldsExtractor`, with the remainder of `Source` being the new source. The final `Remainder` after all field extraction is completed is then returned as the `Remainder` type.

In the implementation for `extract_from`, we call `extract_field` on the `source` value. If the field extraction is successful, we then call `Target::from_variant` to convert the extracted value into `Target`. Otherwise, we take the remainder of `source`, and recursively call `extract_from` with the remaining fields.

When we reach the end of the sum, the field extraction operation ends with an implementation of `Void`:

```rust
impl<Source, Target> FieldsExtractor<Source, Target> for Void {
    type Remainder = Source;

    fn extract_from(source: Source) -> Result<Target, Source> {
        Err(source)
    }
}
```

The `Void` implementation simply sets the final remainder of `Source` as the `Remainder` type, and returns its source as the remainder to indicate that the extraction has failed.

## Example Use of `Upcast`

To better understand how the `FieldsExtractor` operation is done, we will try and navigate through an example upcast operation. Suppose we have a `ShapePlus` enum that is a *superset* of the original `Shape` type:

```rust
#[derive(HasFields, FromVariant, ExtractField)]
pub enum ShapePlus {
    Triangle(Triangle),
    Circle(Circle),
    Rectangle(Rectangle),
}
```

We can perform an upcast from `Shape` to `ShapePlus` as follows:

```rust
let shape = Shape::Circle(Circle { radius: 5.0 });
let shape_plus = shape.upcast(PhantomData::<ShapePlus>);
```

Behind the scene, this is what happens:

- The blanket implementation of `CanUpcast` checks the following:
  - The source `Shape` implements `HasFields`, with the `Fields` type being:
    ```rust
    Sum![
        Field<symbol!("Circle"), Circle>,
        Field<symbol!("Rectangle"), Rectangle>,
    ]
    ```
  - The source `Shape` implements `HasExtractor`, with the extractor type being `PartialShape<IsPresent, IsPresent>`.
  - `Fields` implements `FieldsExtractor` for `PartialShape<IsPresent, IsPresent>` as the source, and `ShapePlus` as the target.
  - The `Remainder` type returned is `PartialShape<IsVoid, IsVoid>`, which implements `FinalizeExtract`.
- The head implementation of `FieldsExtractor` is matched with the following:
  - The current `Tag` is `symbol!("Circle")`, and the current `Value` is `Circle`.
  - The current `Source` is `PartialShape<IsPresent, IsPresent>`, and the current `Target` is `ShapePlus`.
  - `PartialShape<IsPresent, IsPresent>` implements `ExtractField<symbol!("Circle")>`.
    - The `Value` matches `Circle`, and the `Remainder` is `PartialShape<IsVoid, IsPresent>`.
  - `ShapePlus` implements `FromVariant<symbol!("Circle")>`, with the `Value` matching `Circle`.
- The tail `Either<Field<symbol!("Rectangle"), Rectangle>, Void>` implements `FieldsExtractor` as follows:
  - The current `Tag` is `symbol!("Rectangle")`, and the current `Value` is `Rectangle`.
  - The current `Source` is `PartialShape<IsVoid, IsPresent>`, and the current `Target` is `ShapePlus`.
  - `PartialShape<IsVoid, IsPresent>` implements `ExtractField<symbol!("Rectangle")>`.
    - The `Value` matches `Rectangle`, and the `Remainder` is `PartialShape<IsVoid, IsVoid>`.
  - `ShapePlus` implements `FromVariant<symbol!("Rectangle")>`, with the `Value` matching `Rectangle`.
- The tail `Void` implements `FieldExtractor` by returning the source `PartialShape<IsVoid, IsVoid>` as the `Remainder`.

As we can see, what `Upcast` really does is to go through each variant in `Shape`, try and match the variant, and then re-insert the value as a variant in `ShapePlus`. After we finish the extraction from all fields in `Shape`, the result remainder should have all variants becoming `Void`, and thus we can safely discharge it with `FinalizeExtract`.

By deconstructing the upcast operation into the various extensible variants traits, we are able to generically implement `Upcast` entirely in safe Rust to support casting between *any* compatible enums. In particular, we get the operation almost for *free*, as almost all of the implementation are not tied specifically to make `Upcast` work, and can be reused to support other extensible variants operations like `Downcast`.

# Conclusion
