---
slug: extensible-datatypes-part-4
title: 'Programming Extensible Data Types in Rust with CGP - Part 4: Implementing Extensible Variants'
authors: [soares]
tags: [deepdive]
---

This is the **fourth** part of the blog series on **Programming Extensible Data Types in Rust with CGP**. You can read the [first](/blog/extensible-datatypes-part-1), [second](/blog/extensible-datatypes-part-2) and [third](/blog/extensible-datatypes-part-3) parts here.

In the third part of the series, [**Implementing Extensible Records**](/blog/extensible-datatypes-part-3), we have walked through the internal implementation of extensible records, and learned about concepts such as partial records and builder dispatchers.

In this final fourth part of the series, we will have the same walk through for the internal implementation details for **extensible variants**.

<!-- truncate -->

## Recap

As a recap, we have covered the new release of [**CGP v0.4.2**](https://github.com/contextgeneric/cgp/releases/tag/v0.4.2) which now supports the use of **extensible records and variants**, allowing developers to write code that operates on *any struct containing specific fields* or *any enum containing specific variants*, without needing their concrete definition.

In the first part of the series, [**Modular App Construction and Extensible Builders**](/blog/extensible-datatypes-part-1), we demonstrated an example use of the **extensible builder pattern**, which uses **extensible records** to support modular construction of an application context.

Similarly, in the second part of the series, [**Modular Interpreters and Extensible Visitors**](/blog/extensible-datatypes-part-2), we saw how the modular visitor pattern allows us to implement evaluation and to-Lisp conversion for each variant of a language expression enum using separate visitor providers.


## Discussion


Discuss on [Reddit](https://www.reddit.com/r/rust/comments/1md3emg/the_design_and_implementation_of_extensible/), [GitHub](https://github.com/orgs/contextgeneric/discussions/16) or [Discord](https://discord.gg/Hgk3rCw6pQ).


## Acknowledgement

Thank you April Gonçalves for your generous donation support on [Ko-fi](https://ko-fi.com/maybevoid)! ☺️

---

## The Design and Implementation of Extensible Variants

Now that we've covered how extensible records work in CGP, we can turn our attention to **extensible variants**. At first glance, it might seem like a completely different mechanism — but surprisingly, the approach used to implement extensible variants is very similar to that of extensible records. In fact, many of the same principles apply, just in the “opposite direction”.

This close relationship between records and variants is rooted in **category theory**. In that context, records are known as **products**, while variants (or enums) are referred to as **sums** or **coproducts**. These terms highlight a deep **duality** between the two: just as products *combine* values, coproducts represent a *choice* among alternatives. CGP embraces this theoretical foundation and leverages it to create a unified design for both extensible records and extensible variants.

This duality is not just theoretical — it has practical implications for software architecture. Our design in CGP builds directly on prior research into **extensible data types**, particularly in the context of functional programming and type systems. For more on the background, see the paper on [Extensible Data Types](https://dl.acm.org/doi/10.1145/3290325), as well as this excellent [intro to category theory](https://bartoszmilewski.com/2015/01/07/products-and-coproducts/) by Bartosz Milewski.

With this in mind, we’ll now explore the CGP constructs that support extensible variants. As you go through the examples and implementation details, we encourage you to look for the parallels and contrasts with extensible records.

---

## Base Implementation

### `FromVariant` Trait

Just as extensible records use the `HasField` trait to extract values from a struct, extensible variants in CGP use the `FromVariant` trait to *construct* an enum from a single variant value. The trait is defined as follows:

```rust
pub trait FromVariant<Tag> {
    type Value;

    fn from_variant(_tag: PhantomData<Tag>, value: Self::Value) -> Self;
}
```

Like `HasField`, the `FromVariant` trait is parameterized by a `Tag`, which identifies the name of the variant. It also defines an associated `Value` type, representing the data associated with that variant. Unlike `HasField`, which extracts a value, `from_variant` takes in a `Value` and returns an instance of the enum.

### Example: Deriving `FromVariant`

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
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;

    fn from_variant(_tag: PhantomData<Symbol!("Circle")>, value: Self::Value) -> Self {
        Shape::Circle(value)
    }
}

impl FromVariant<Symbol!("Rectangle")> for Shape {
    type Value = Rectangle;

    fn from_variant(_tag: PhantomData<Symbol!("Rectangle")>, value: Self::Value) -> Self {
        Shape::Rectangle(value)
    }
}
```

This allows the `Shape` enum to be constructed generically using just the tag and value.

### Restrictions on Enum Shape

To ensure ergonomics and consistency, CGP restricts the kinds of enums that can derive `FromVariant`. Specifically, supported enums must follow the **sums of products** pattern — each variant must contain *exactly one unnamed field*.

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

These more complex variants are not supported because they would make it harder to represent variant fields as simple types, which would, in turn, lead to less ergonomic APIs. By restricting each variant to a single unnamed field, CGP ensures that types like `FromVariant::Value` remain straightforward and intuitive.

If you need to represent more complex data in a variant, we recommend wrapping that data in a dedicated struct. This way, you can still take advantage of CGP's extensible variant system while maintaining type clarity and composability.

### Partial Variants

Just as CGP supports partially constructed structs through *partial records*, it also enables **partial variants** to work with **partially deconstructed** enums in a similarly flexible way. Partial variants allow you to pattern match on each variant of an enum incrementally, while safely excluding any variants that have already been handled. This makes it possible to build exhaustive and type-safe match chains that evolve over time.

Consider the `Shape` enum we explored earlier. CGP would generate a corresponding `PartialShape` enum that represents the partial variant form of `Shape`:

```rust
pub enum PartialShape<F0: MapType, F1: MapType> {
    Circle(F0::Map<Circle>),
    Rectangle(F1::Map<Rectangle>),
}
```

### `HasExtractor` trait

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

### `IsVoid` Type Mapper

The key distinction between partial records and partial variants lies in how we represent the **absence** of data. For partial variants, CGP introduces the `IsVoid` type mapper to indicate that a variant has already been extracted and is no longer available:

```rust
pub enum Void {}
pub struct IsVoid;

impl MapType for IsVoid {
    type Map<T> = Void;
}
```

The `Void` type is defined as an empty enum with no variants. This means that it is **impossible** to construct a value of type `Void`, and any code that attempts to match on a `Void` value will be statically unreachable. This makes it a safe and expressive way to model a variant that no longer exists in a given context.

Conceptually, `Void` serves the same purpose as Rust’s built-in [**never type**](https://doc.rust-lang.org/reference/types/never.html) or the [`Infallible`](https://doc.rust-lang.org/std/convert/enum.Infallible.html) type from the standard library. However, CGP defines `Void` explicitly to distinguish its special role in the context of extensible variants.

While `IsNothing` is used for absent fields in partial records, we use `IsVoid` to represent removed or matched variants in partial variants. This ensures that once a variant has been extracted, it cannot be matched again — preserving both soundness and safety in CGP’s type-driven pattern matching.

### `ExtractField` Trait

Once an enum has been converted into its partial variant form, we can begin incrementally pattern matching on each variant using the `ExtractField` trait. This trait enables safe, step-by-step extraction of variant values, and is defined as follows:

```rust
pub trait ExtractField<Tag> {
    type Value;
    type Remainder;

    fn extract_field(self, _tag: PhantomData<Tag>) -> Result<Self::Value, Self::Remainder>;
}
```

Just like `FromVariant` and `HasField`, the `ExtractField` trait takes a `Tag` type to identify the variant, and includes an associated `Value` type representing the variant’s inner data. Additionally, it defines a `Remainder` type, which represents the **remaining** variants that have not yet been matched.

The `extract_field` method consumes the value and returns a `Result`, where a successful match yields the extracted `Value`, and a failed match returns the `Remainder`. Although this uses the `Result` type, the `Err` case is not really an error in the traditional sense — rather, it represents the remaining variants yet to be handled, much like how errors represent alternative outcomes in Rust.

#### Example Implementation of `ExtractField`

To understand how `ExtractField` works in practice, let’s look at an implementation for extracting the `Circle` variant from a `PartialShape`:

```rust
impl<F1: MapType> ExtractField<Symbol!("Circle")> for PartialShape<IsPresent, F1> {
    type Value = Circle;
    type Remainder = PartialShape<IsVoid, F1>;

    fn extract_field(self, _tag: PhantomData<Symbol!("Circle")>) ->
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

#### Example Use of `ExtractField`

With `ExtractField`, we can now incrementally extract and match against variants in a safe and ergonomic way. Here’s an example of computing the area of a shape using this approach:

```rust
pub fn compute_area(shape: Shape) -> f64 {
    match shape
        .to_extractor() // PartialShape<IsPresent, IsPresent>
        .extract_field(PhantomData::<Symbol!("Circle")>)
    {
        Ok(circle) => PI * circle.radius * circle.radius,
        // PartialShape<IsVoid, IsPresent>
        Err(remainder) => match remainder.extract_field(PhantomData::<Symbol!("Rectangle")>) {
            Ok(rectangle) => rectangle.width * rectangle.height,
            // PartialShape<IsVoid, IsVoid>
            // No need to match on `Err`
        },
    }
}
```

In this example, we begin by converting the `Shape` value into a `PartialShape` with all variants present using `to_extractor`. We then call `extract_field` to try extracting the `Circle` variant. If successful, we compute the circle's area. If not, we receive a remainder value where the `Circle` variant is now marked as `IsVoid`. This remainder is then used to attempt extracting the `Rectangle` variant. If that succeeds, we compute the area accordingly.

By the time we reach the second `Err` case, the remainder has the type `PartialShape<IsVoid, IsVoid>`, which cannot contain any valid variant. Because of this, we can safely omit any further pattern matching, and the compiler guarantees that there are no unreachable or unhandled cases.

What makes this approach so powerful is that the Rust type system can statically verify that it is impossible to construct a valid value for `PartialShape<IsVoid, IsVoid>`. We no longer need to write boilerplate `_ => unreachable!()` code or use runtime assertions. The type system ensures exhaustiveness and soundness entirely at compile time, enabling safer and more maintainable implementation of extensible variants.

### Short-Circuiting Remainder

In our earlier implementation of `compute_area`, we used nested `match` expressions to handle the `Result` returned from each call to `extract_field`. If you are familiar with the `?` operator in Rust, you might be wondering why we didn’t use it here to simplify the logic.

The reason is that we want to short circuit and return the `Ok` variant as soon as a match succeeds, while the `Err` case contains a remainder type that changes with each call to `extract_field`. This behavior is the inverse of how `Result` is typically used in Rust, where the `Err` variant is the one that gets returned early, and the `Ok` type changes as the computation progresses.

To better understand what we are trying to achieve, consider the following pseudocode that illustrates the intent more clearly:

```rust
pub fn compute_area(shape: Shape) -> Result<f64, Infallible> {
    let remainder = shape
        .to_extractor()
        .extract_field(PhantomData::<Symbol!("Circle")>)
        .map(|circle| PI * circle.radius * circle.radius)⸮;

    let remainder = remainder
        .extract_field(PhantomData::<Symbol!("Rectangle")>)
        .map(|rectangle| rectangle.width * rectangle.height)⸮;

    match remainder {}
}
```

In this pseudocode, we introduce a fictional operator `⸮`, which behaves like the opposite of `?`. Instead of short circuiting on `Err`, it short circuits on `Ok`, returning the value immediately. If the result is `Err`, it binds the remainder to the `remainder` variable and continues.

In this setup, each call to `extract_field` uses `.map` to transform a successful match into the final `f64` result. If the match succeeds, `⸮` returns early. Otherwise, we continue with the remainder, which gradually becomes more constrained until it is fully uninhabited. Once all variants have been tried, the final `match remainder {}` statically asserts that no remaining case is possible.

This highlights a subtle but important point: the `compute_area` function never actually returns an `Err` in practice. To satisfy the function’s signature, we return a `Result<f64, Infallible>`, where `Infallible` indicates that failure is not possible.

Some readers may suggest alternative approaches, such as flipping the result to `Result<Remainder, Value>` so that the `?` operator could be used to return the value directly. While that might make the surface syntax cleaner, it reverses the intuitive meaning of the result. In this case, `Remainder` is the exceptional path, and `Value` is what we expect when the extraction succeeds.

The introduction of `⸮` is not meant to advocate for a new language feature. Rather, it serves to clarify the control flow and encourage you to think about how this pattern relates to existing Rust constructs like `?`, `.await`, and combinations such as `.await?`. In practice, we do not need to manually write functions like `compute_area` or invent new operators. The extensible visitor pattern we will explore later provides a mechanism that effectively captures this logic for us.

We will revisit this idea when we discuss how the visitor pattern automates this process. For now, let’s continue by looking at how to finalize an empty remainder.

### `FinalizeExtract` Trait

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
#### `FinalizeExtractResult` Trait

When working with results of type `Result<Output, Remainder>`, where the `Remainder` type is guaranteed to be inhabitable, it is often useful to have a convenient way to directly extract the `Output` value. To achieve this, CGP defines the `FinalizeExtractResult` trait, which provides a helper method to finalize and unwrap such results. Its definition includes a blanket implementation for all `Result` types where the error type implements `FinalizeExtract`:

```rust
pub trait FinalizeExtractResult {
    type Output;

    fn finalize_extract_result(self) -> Self::Output;
}

impl<T, E> FinalizeExtractResult for Result<T, E>
where
    E: FinalizeExtract,
{
    type Output = T;

    fn finalize_extract_result(self) -> T {
        match self {
            Ok(value) => value,
            Err(remainder) => remainder.finalize_extract(),
        }
    }
}
```

With `FinalizeExtractResult`, any result value can call `finalize_extract_result()` to obtain the `Output` directly, as long as the remainder type implements `FinalizeExtract`. This allows functions that work with extractable variants to become simpler and more readable. For example, the implementation of `compute_area` can be written as:

```rust
pub fn compute_area(shape: Shape) -> f64 {
    match shape
        .to_extractor()
        .extract_field(PhantomData::<Symbol!("Circle")>)
    {
        Ok(circle) => PI * circle.radius * circle.radius,
        Err(remainder) => {
            let rectangle = remainder
                .extract_field(PhantomData::<Symbol!("Rectangle")>)
                .finalize_extract_result();

            rectangle.width * rectangle.height
        }
    }
}
```

When handling the remainder after the `Circle` variant was extracted, we use `finalize_extract_result` after calling `remainder.extract_field()` to get the `Rectangle` variant.

This trait provides a small but valuable ergonomic improvement, especially when performing generic extractions and finalizations. It allows developers to avoid repetitive pattern matching and ensures that the final output can be obtained with a single, clear method call.

---

## Implementation of Casts

With the foundational traits for extensible variants in place, we can now explore how to implement the `CanUpcast` and `CanDowncast` traits. These traits enable safe and generic upcasting and downcasting between enums that share compatible variants.

### `HasFields` Implementation

Just as extensible records rely on `HasFields` for iterating over their fields, extensible variants use a similar mechanism to iterate over their variants. This allows the generic casting implementation to iterate over each variant of an enum.

For example, the `HasFields` implementation for the `Shape` enum is defined as follows:

```rust
impl HasFields for Shape {
    type Fields = Sum![
        Field<Symbol!("Circle"), Circle>,
        Field<Symbol!("Rectangle"), Rectangle>,
    ];
}
```

Here, instead of using the `Product!` macro (which is used for structs), we use the `Sum!` macro to build a *type-level sum* representing all variants in the enum. The `Sum!` macro expands to a nested structure of `Either`, similar to how `Product!` expands into a chain of `Cons`.

For example, the `Sum!` expression above desugars to:

```rust
impl HasFields for Shape {
    type Fields = Either<
        Field<Symbol!("Circle"), Circle>,
        Either<
            Field<Symbol!("Rectangle"), Rectangle>,
            Void,
        >,
    >;
}
```

Where `Either` is defined in a similar fashion to Rust's standard `Result` type, but with variant names that reflect the sum type structure:

```rust
pub enum Either<A, B> {
    Left(A),
    Right(B),
}
```

In this way, we represent the enum's variants as a nested sum, with `Void` as the terminating type to signify the end of the variant choices.

### `CanUpcast` Implementation

With `HasFields` implemented, we are ready to define the `CanUpcast` trait. This trait allows a source enum to be upcasted to a target enum that is a superset of the source:

```rust
pub trait CanUpcast<Target> {
    fn upcast(self, _tag: PhantomData<Target>) -> Target;
}
```

The trait is generic over the `Target` type we wish to upcast to. The `upcast` method takes the original enum and converts it into the target enum, using `PhantomData` to assist with type inference.

The implementation is provided generically through a blanket implementation:

```rust
impl<Context, Source, Target, Remainder> CanUpcast<Target> for Context
where
    Context: HasFields + HasExtractor<Extractor = Source>,
    Context::Fields: FieldsExtractor<Source, Target, Remainder = Remainder>,
    Remainder: FinalizeExtract,
{
    fn upcast(self, _tag: PhantomData<Target>) -> Target {
        Context::Fields::extract_from(self.to_extractor()).finalize_extract_result()
    }
}
```

Here’s how it works. First, the `Context` type (the source enum) must implement both `HasFields` and `HasExtractor`. The `HasFields` trait provides a type-level sum of variants, and `HasExtractor` converts the enum into its corresponding partial variants. Next, the associated `Fields` type must implement the helper trait `FieldsExtractor`, which handles the actual extraction of variants into the target type. The `Remainder` returned by this operation must then implement `FinalizeExtract`, which guarantees that all source variants have been accounted for.

In the method body, we begin by calling `self.to_extractor()` to convert the source enum into a value with partial variants. We then use `Fields::extract_from` to extract the relevant variants into the target enum. Finally, we call `finalize_extract_result()` to discharge the remainder in `Err`, and return the `Target` result in `Ok`.

### `FieldsExtractor` Trait

The `FieldsExtractor` trait serves as a helper for casting between enums. It is defined as follows:

```rust
pub trait FieldsExtractor<Source, Target> {
    type Remainder;

    fn extract_from(source: Source) -> Result<Target, Self::Remainder>;
}
```

This trait is parameterized by two types: `Source`, which represents the partial variants of the source enum, and `Target`, which is the fully constructed destination enum. It also defines a `Remainder` associated type to capture any variant in the source that could not be extracted into the target.

The `extract_from` method attempts to convert the given partial variants from the `Source` into a complete `Target`. If successful, it returns the constructed `Target` value. Otherwise, it returns the remainder of the `Source` that could not be matched.

The core implementation of `FieldsExtractor` operates recursively over the `Sum!` list of fields. For the head of the list, the implementation is written as:

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

In this implementation, we deconstruct the head of the sum into a `Field<Tag, Value>` type. We then require that the `Source` type supports `ExtractField<Tag>`, which allows us to attempt extracting the field corresponding to that tag. We also require the `Target` enum to support `FromVariant<Tag>`, so that once the field is extracted, we can reconstruct the target enum from it. In both traits, the associated `Value` type must be consistent.

If the extraction succeeds, we pass the value into `Target::from_variant` to construct the result. If it fails, we take the `Remainder` returned from `extract_field`, and recursively call `extract_from` on the rest of the fields. The associated `Remainder` type continues to track whatever remains after each recursive step.

Eventually, the recursion reaches the end of the `Sum!` list, which is represented by the `Void` type. At this point, we provide the base case:

```rust
impl<Source, Target> FieldsExtractor<Source, Target> for Void {
    type Remainder = Source;

    fn extract_from(source: Source) -> Result<Target, Source> {
        Err(source)
    }
}
```

In this final case, the trait simply sets the entire `Source` as the `Remainder`, indicating that none of the fields matched. This implementation ends the recursive search through the variants and signals that the cast could not be completed.

This pattern allows us to generically extract variants from an extensible enum, one field at a time, while safely and efficiently handling any unmatched cases using Rust’s powerful type system.

### Example Use of `Upcast`

To better understand how the `FieldsExtractor` operation works, let’s walk through a concrete example of an upcast. Suppose we define a new enum `ShapePlus` that extends the original `Shape` type by including an additional variant:

```rust
#[derive(HasFields, FromVariant, ExtractField)]
pub enum ShapePlus {
    Triangle(Triangle),
    Rectangle(Rectangle),
    Circle(Circle),
}
```

We can then perform an upcast from `Shape` to `ShapePlus` with the following code:

```rust
let shape = Shape::Circle(Circle { radius: 5.0 });
let shape_plus = shape.upcast(PhantomData::<ShapePlus>);
```

Behind the scenes, the upcast proceeds through a series of trait-based checks and operations:

First, the blanket implementation of `CanUpcast` verifies several conditions:

* The source type `Shape` must implement `HasFields`, with the `Fields` type resolving to:
  ```rust
  Sum![
      Field<Symbol!("Circle"), Circle>,
      Field<Symbol!("Rectangle"), Rectangle>,
  ]
  ```
* `Shape` must also implement `HasExtractor`, with its associated `Extractor` type being `PartialShape<IsPresent, IsPresent>`.
* The `Fields` type must implement `FieldsExtractor`, with `PartialShape<IsPresent, IsPresent>` as the source and `ShapePlus` as the target.
* The result of the extraction yields a remainder of type `PartialShape<IsVoid, IsVoid>`, which in turn implements `FinalizeExtract`.

Next, the `FieldsExtractor` implementation for the head of the sum begins processing:

* The current `Tag` is `Symbol!("Circle")`, and the associated `Value` is of type `Circle`.
* The `Source` is `PartialShape<IsPresent, IsPresent>`, and the `Target` is `ShapePlus`.
* The source implements `ExtractField<Symbol!("Circle")>`, which succeeds with `Circle` as the extracted value and `PartialShape<IsVoid, IsPresent>` as the remainder.
* The target `ShapePlus` implements `FromVariant<Symbol!("Circle")>`, again with `Circle` being the `Value` type.

The extractor then proceeds to the next variant in the sum:

* The current `Tag` is `Symbol!("Rectangle")`, with `Rectangle` as the `Value`.
* The updated `Source` is now `PartialShape<IsVoid, IsPresent>`, and the `Target` remains `ShapePlus`.
* This source implements `ExtractField<Symbol!("Rectangle")>`, yielding `Rectangle` as the value and `PartialShape<IsVoid, IsVoid>` as the final remainder.
* The target once again supports `FromVariant<Symbol!("Rectangle")>` using the matching `Rectangle` type.
* At the end of the chain, the `Void` variant is reached. The `FieldsExtractor` implementation for `Void` simply returns the remainder, which in this case is `PartialShape<IsVoid, IsVoid>`.

What this process shows is that the `Upcast` operation works by examining each variant in the source type `Shape`, extracting each present value, and reinserting it into the target type `ShapePlus`. Once all fields have been processed, the remaining variants are guaranteed to be uninhabited. At that point, we can safely discharge the remainder using the `FinalizeExtract` trait.

By breaking down the upcast into individual type-driven steps over extensible variants, we can implement upcasting entirely in safe Rust. Even more importantly, this implementation is fully generic and reusable. We are not writing code solely for the purpose of supporting `Upcast` — instead, we are building a reusable foundation that also supports operations like `Downcast` and other generic manipulations over extensible variants.

### `CanDowncast` Implementation

With the upcast operation in place, we can now turn to the implementation of `CanDowncast`. The `CanDowncast` trait is defined as follows:

```rust
pub trait CanDowncast<Target> {
    type Remainder;

    fn downcast(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder>;
}
```

This trait is used to convert a value of an enum type into another enum that represents a *subset* of its variants. Unlike `CanUpcast`, which guarantees success by moving into a larger enum, `CanDowncast` may *fail* if the source contains variants not present in the target. To account for this, the trait includes an associated `Remainder` type to capture any unmatched variants, and the `downcast` method returns a `Result` that either yields the successfully downcasted value or the remainder.

As with `CanUpcast`, we can define `CanDowncast` using a blanket implementation:

```rust
impl<Context, Source, Target, Remainder> CanDowncast<Target> for Context
where
    Context: HasExtractor<Extractor = Source>,
    Target: HasFields,
    Target::Fields: FieldsExtractor<Source, Target, Remainder = Remainder>,
{
    type Remainder = Remainder;

    fn downcast(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder> {
        Target::Fields::extract_from(self.to_extractor())
    }
}
```

With all the foundational components from `CanUpcast` already in place, the implementation of `CanDowncast` becomes remarkably straightforward. Instead of requiring the source `Context` to implement `HasFields`, we shift that requirement to the `Target`. We still use the `HasExtractor` trait to obtain the partial variant representation of the source. From there, we iterate over the target fields using `FieldsExtractor`, attempting to extract a match from the source. Because we are narrowing into a smaller enum, some variants may remain unmatched. In those cases, we simply return the remainder rather than attempting to finalize it, as we did in `CanUpcast`.

This difference highlights the key distinction between upcasting and downcasting in this model. The `Upcast` operation extracts from all fields in the source and expects the remainder to be empty, whereas `Downcast` extracts only those variants present in the target and leaves the unmatched remainder intact. Yet aside from this inversion of roles between source and target, the two implementations share the same reusable machinery — including `FieldsExtractor` — demonstrating the flexibility and composability of the CGP approach to extensible variants.

### Example Use of Downcast

With `CanDowncast` in place, we can now explore how to use it in practice. Consider the following example, where we attempt to downcast from a `ShapePlus` enum to a `Shape` enum:

```rust
let shape_plus = ShapePlus::Triangle(Triangle {
    base: 3.0,
    height: 4.0,
});

let area = match shape_plus.downcast(PhantomData::<Shape>) {
    Ok(shape) => match shape {
        Shape::Circle(circle) => PI * circle.radius * circle.radius,
        Shape::Rectangle(rectangle) => rectangle.width * rectangle.height,
    },
    // PartialShapePlus<IsPresent, IsVoid, IsVoid>
    Err(remainder) => match remainder.extract_field(PhantomData::<Symbol!("Triangle")>) {
        Ok(triangle) => triangle.base * triangle.height / 2.0,
    },
};
```

In this example, we start with a `ShapePlus` value that holds a `Triangle`. We then call `downcast`, attempting to convert it to a `Shape`, which does not include the `Triangle` variant. Internally, the downcast operation uses `Shape::Fields` to iterate over the variants defined in `Shape` and tries to extract each from the original `ShapePlus` value. If any of those variants are found — such as `Circle` or `Rectangle` — the match succeeds and we compute the corresponding area from `Shape`.

However, when the actual variant in this case is `Triangle`, which is not part of `Shape`, the downcast fails and we receive the remainder of the partial variant structure. This remainder, of type `PartialShapePlus<IsPresent, IsVoid, IsVoid>`, contains only the `Triangle` variant. We then use `extract_field` to retrieve the triangle and compute its area. At this point, no other variants remain to be handled.

One of the most impressive aspects of both upcast and downcast is that they work seamlessly even when the source and target enums define their variants in entirely different orders. Because the trait implementations, such as `ExtractField`, operate in a generic and order-independent way, the correctness and behavior of casting are preserved regardless of variant ordering. This level of flexibility makes the CGP approach to extensible variants both powerful and practical for real-world use.

---

## Implementation of Visitor Dispatcher

With the traits for extensible variants now in place, we can turn our attention to how CGP implements generalized **visitor dispatchers**, similar to the [builder dispatchers](/blog/extensible-datatypes-part-3/#builder-dispatcher) described in the previous part of this series.

### `MatchWithHandlers`

In the [examples from Part 2](/blog/extensible-datatypes-part-2/#dispatching-eval), we introduced dispatchers such as `MatchWithValueHandlers` and `MatchWithValueHandlersRef`, which delegate the handling of enum variants to different visitor handlers based on the `Input` type. These dispatchers are built on top of a more fundamental dispatcher called `MatchWithHandlers`, whose implementation is shown below:

```rust
#[cgp_provider]
impl<Context, Code, Input, Output, Remainder, Handlers> Computer<Context, Code, Input>
    for MatchWithHandlers<Handlers>
where
    Input: HasExtractor,
    DispatchMatchers<Handlers>:
        Computer<Context, Code, Input::Extractor, Output = Result<Output, Remainder>>,
    Remainder: FinalizeExtract,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: Input) -> Output {
        DispatchMatchers::compute(context, code, input.to_extractor()).finalize_extract_result()
    }
}
```

The `MatchWithHandlers` provider is parameterized by a `Handlers` type, which represents a type-level list of visitor handlers responsible for processing the variants of a generic `Input` enum. The implementation requires `Input` to implement the `HasExtractor` trait, which provides access to its partial variants.

Within the `compute` method, we first convert the input into its extractor form using `input.to_extractor()`. This partial variant is then passed to the lower-level dispatcher `DispatchMatchers<Handlers>`, which attempts to match and handle each variant. It returns a `Result<Output, Remainder>`, where a successful match produces an `Output`, and an unmatched remainder is returned otherwise. But since `Remainder` is expected to implement `FinalizeExtract`, we can call `finalize_extract_result()` to return the `Output` directly.

#### `DispatchMatchers`

In our earlier implementation of extensible builders via `BuildWithHandlers`, we [used `PipeHandlers`](/blog/extensible-datatypes-part-3/#buildwithhandlers-provider) to compose a pipeline of builder handlers that successively filled in partial records. For extensible visitors, we follow a similar pattern with a slight variation that reflects the different control flow.

The dispatcher `DispatchMatchers` is defined as follows:

```rust
pub type DispatchMatchers<Providers> = PipeMonadic<OkMonadic, Providers>;
```

This definition constructs a **monadic pipeline** of visitor handlers, using `OkMonadic` as the monad implementation.

### What is a Monad?!

At this point, many readers coming from a Rust background may be wondering what exactly a [monad](https://wiki.haskell.org/Monad) is, and how it relates to implementing extensible visitors. In this section, we will break down the concept in simplified terms using familiar Rust patterns and constructs.

A monad, often written as `M`, is a type that acts as a container for another value `T`, and it typically appears in the form `M<T>`. If you have worked with `Option<T>`, `Result<T, E>`, or asynchronous code using `impl Future<Output = T>`, then you have already used monadic types in Rust.

Monads are not just containers. They also provide a way to operate on the values they contain, typically through an operation known as "bind." In Rust, this concept appears through the use of operators like `?`, `.await`, and `.await?`, all of which allow you to "extract" or "unwrap" the value inside a container and propagate control based on the result.

With this understanding, we can think of `PipeMonadic` in CGP as a mechanism that automatically applies these unwrapping operations between steps in a pipeline. It takes the result from one handler and, using a monadic operator, unwraps it before passing it along as input to the next handler. This is how CGP builds a pipeline of computations where each step can short-circuit or continue depending on its output.

The real strength of this approach is that it generalizes well. You are not limited to a specific type like `Result`; you can apply the same logic to any monad-like type, including more complex combinations such as `impl Future<Output = Result<Result<Option<T>, E1>, E2>>`. In principle, you could imagine applying something like `.await???` to extract the inner value, and with monads, this can be abstracted and automated.

#### `OkMonadic` Monad Provider

In the case of `DispatchMatchers`, the monad provider we use is called `OkMonadic`. This corresponds to the custom operator `⸮` we introduced in the pseudocode in the [`compute_area` example](#short-circuiting-remainder), which short-circuits on the `Ok` variant and passes along the changing `Err` remainder.

When we say that `DispatchMatchers` is defined using `PipeMonadic<OkMonadic, Providers>`, we mean that CGP should build a handler pipeline where each step uses the `⸮` operator to either return early with `Ok(output)` or continue processing the `Err(remainder)` with the next handler.

Because of `PipeMonadic` and `OkMonadic`, we do not need to write this logic ourselves. CGP handles the monadic control flow automatically, allowing us to focus on the behavior of each handler without worrying about wiring them together manually.

If any of this still feels unclear, do not worry. We will walk through a concrete example next to clarify how it works in practice. We also plan to publish a separate blog post that dives deeper into how CGP implements monads in Rust, including the internals of `PipeMonadic` and related abstractions.

### Example Use of `MatchWithHandlers`

To understand how to use `MatchWithHandlers` directly, let's revisit the example of computing the area of a `Shape`. We start by defining two separate `Computer` providers that calculate the area for the `Circle` and `Rectangle` variants:

```rust
#[cgp_computer]
fn circle_area(circle: Circle) -> f64 {
    PI * circle.radius * circle.radius
}

#[cgp_computer]
fn rectangle_area(rectangle: Rectangle) -> f64 {
    rectangle.width * rectangle.height
}
```

#### `#[cgp_computer]` Macro

The `#[cgp_computer]` macro allows us to transform these pure functions into context-generic providers that can be referenced as types. Behind the scenes, this macro generates `Computer` implementations similar to the following:

```rust
#[cgp_provider]
impl<Context, Code> Computer<Context, Code, Circle> for CircleArea {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: Circle) -> f64 {
        circle_area(input)
    }
}
```

This macro simplifies the process of defining `Computer` providers by letting us write them as plain functions. Because the macro ignores the `Context` and `Code` types, the generated provider works with any `Context` and `Code` you supply.

#### `ComputeShapeArea` Handler

With `CircleArea` and `RectangleArea` defined, we can create a `ComputeShapeArea` handler by using `MatchWithHandlers` as a type alias:

```rust
pub type ComputeShapeArea = MatchWithHandlers::<
    Product![
        ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<CircleArea>>,
        ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<RectangleArea>>,
    ],
>;
```

Rather than passing providers directly to `MatchWithHandlers`, we wrap them with helper handlers. The `ExtractFieldAndHandle` handler extracts the variant value associated with a specific tag, such as `Symbol!("Circle")`, and forwards it to the inner handler `HandleFieldValue<CircleArea>`.

The inner handler `HandleFieldValue` receives the input as `Field<Symbol!("Circle"), Circle>`, extracts the `Circle` value, and passes it to `CircleArea`. We will explore the implementations of `ExtractFieldAndHandle` and `HandleFieldValue` shortly, but first, let's see how `ComputeShapeArea` is used.

As a whole, the instantiated `MatchWithHandlers` implements the `Computer` trait. We can call `compute` on it using `()` for both the `Context` and `Code` types like this:

```rust
let shape = Shape::Circle(Circle { radius: 5.0 });
let area = ComputeShapeArea::compute(&(), PhantomData::<()>, shape);
```

This works because the `Computer` instances defined with `#[cgp_computer]` are generic over any `Context` and `Code`.

Under the hood, `MatchWithHandlers` implements `ComputeShapeArea` roughly as the following pseudocode:

```rust
let remainder = shape.to_extractor();

let remainder = remainder
    .extract_field(Symbol!("Circle"))
    .map(|circle| CircleArea::compute(&(), PhantomData::<()>, circle))⸮;

let remainder = remainder
    .extract_field(Symbol!("Rectangle"))
    .map(|rectangle| RectangleArea::compute(&(), PhantomData::<()>, rectangle))⸮;

remainder.finalize_extract();
```

Here, `MatchWithHandlers` performs the same `⸮` short-circuit operation described earlier in the [short-circuiting remainder](#short-circuiting-remainder) section. For each `Ok` value returned by extraction, the corresponding `Computer` provider computes the area.

This example highlights how much boilerplate `MatchWithHandlers` abstracts away for us. Its implementation is essentially a monadic pipeline built using `PipeMonadic`, where `OkMonadic` provides the behavior of the `⸮` operator used in this pseudocode.

### `ExtractFieldAndHandle`

To better understand how the earlier `MatchWithHandlers` example works, let's examine the implementation of the `ExtractFieldAndHandle` provider:

```rust
#[cgp_provider]
impl<Context, Code, Input, Tag, Value, Provider, Output, Remainder>
    Computer<Context, Code, Input>
    for ExtractFieldAndHandle<Tag, Provider>
where
    Input: ExtractField<Tag, Value = Value, Remainder = Remainder>,
    Provider: Computer<Context, Code, Field<Tag, Value>, Output = Output>,
{
    type Output = Result<Output, Remainder>;

    fn compute(
        context: &Context,
        tag: PhantomData<Code>,
        input: Input,
    ) -> Result<Output, Remainder> {
        let value = input.extract_field(PhantomData::<Tag>)?;
        let output = Provider::compute(context, tag, value.into());
        Ok(output)
    }
}
```

While the type signature may seem complex, the behavior is straightforward. Given some partial variants `Input`, this handler attempts to extract a variant with the specified `Tag` using `ExtractField`. If extraction succeeds, it wraps the extracted variant as a tagged field `Field<Tag, Value>` and passes it to the inner `Provider` for processing. If extraction fails, it returns the remainder as an `Err`, allowing the next handler in the monadic pipeline to try.

Note that the inner `Provider` receives a tagged `Field<Tag, Value>` rather than a bare `Value`. This allows the provider to differentiate variants that share the same `Value` type but differ in their variant `Tag`. For example, consider:

```rust
pub enum FooBar {
    Foo(u64),
    Bar(u64),
}
```

Here, both `Foo` and `Bar` hold `u64` values. `ExtractFieldAndHandle` will pass these as `Field<Symbol!("Foo"), u64>` and `Field<Symbol!("Bar"), u64>` respectively, so the provider can handle them differently by matching on the `Tag`.

#### `HandleFieldValue`

The tagged `Field` input from `ExtractFieldAndHandle` is useful when multiple variants share the same `Value` type. However, in simpler cases like our `Shape` example, we often just want to handle the contained value directly, ignoring the tag. The `HandleFieldValue` wrapper simplifies this by “peeling off” the `Field` wrapper and passing only the inner value to the inner provider:

```rust
#[cgp_provider]
impl<Context, Code, Tag, Input, Output, Provider> Computer<Context, Code, Field<Tag, Input>>
    for HandleFieldValue<Provider>
where
    Provider: Computer<Context, Code, Input, Output = Output>,
{
    type Output = Output;

    fn compute(
        context: &Context,
        tag: PhantomData<Code>,
        input: Field<Tag, Input>,
    ) -> Self::Output {
        Provider::compute(context, tag, input.value)
    }
}
```

As shown, `HandleFieldValue` simply unwraps the input from `Field<Tag, Input>` and forwards the contained `Input` value to the inner provider.

#### Revisiting `ComputeShapeArea`

Now that we've understood `ExtractFieldAndHandle` and `HandleFieldValue`, let’s review what happens inside `ComputeShapeArea`:

```rust
pub type ComputeShapeArea = MatchWithHandlers::<
    Product![
        ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<CircleArea>>,
        ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<RectangleArea>>,
    ],
>;
```

* `MatchWithHandlers` uses `HasExtractor` to convert `Shape` into `PartialShape<IsPresent, IsPresent>`, then passes it to `ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<CircleArea>>`.
* `ExtractFieldAndHandle` attempts to extract the `Circle` variant from `PartialShape<IsPresent, IsPresent>`:
  * If successful, the extracted value is passed as `Field<Symbol!("Circle"), Circle>` to `HandleFieldValue<CircleArea>`.
    * `HandleFieldValue<CircleArea>` unwraps the `Circle` value and passes it to `CircleArea`.
  * Otherwise, the remainder `PartialShape<IsVoid, IsPresent>` is returned as an error.
* Next, `ExtractFieldAndHandle` tries to extract the `Rectangle` variant from `PartialShape<IsVoid, IsPresent>`:
  * If successful, the extracted value is passed as `Field<Symbol!("Rectangle"), Rectangle>` to `HandleFieldValue<RectangleArea>`.
    * `HandleFieldValue<RectangleArea>` unwraps the `Rectangle` and passes it to `RectangleArea`.
  * Otherwise, the remainder `PartialShape<IsVoid, IsVoid>` is returned as an error.
* Finally, `MatchWithHandlers` calls `FinalizeExtract` on `PartialShape<IsVoid, IsVoid>` to assert that the remainder is empty and discharge the impossible case.

### Unifying Variant Value Handlers

So far, we have seen how `MatchWithHandlers` can serve as a powerful low-level tool to implement extensible visitors. However, it requires explicitly listing a handler for each variant in the provided handler list. To make this process more ergonomic, we can build higher-level abstractions like `MatchWithValueHandlers`, which automatically derives the list of variant handlers passed to `MatchWithHandlers`.

Before implementing `MatchWithValueHandlers`, we first need to unify the variant handlers used in `MatchWithHandlers`. Instead of specifying separate handlers for each variant, we modify the variant handlers so that the same handler is used for all variants. For example:

```rust
pub type ComputeShapeArea = MatchWithHandlers<
    Product![
        ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
        ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
    ],
>;
```

Here, rather than using distinct `CircleArea` and `RectangleArea` handlers, we use a single handler, `ComputeArea`, for both variants. This creates a unified pattern of `ExtractFieldAndHandle<Tag, HandleFieldValue<ComputeArea>>` for each entry. Recognizing this repetition allows us to build further abstractions that simplify these common patterns.

To understand this better, let's explore how `ComputeArea` itself can be implemented. For many extensible variants such as `Shape`, a straightforward approach is to define a regular Rust trait that computes the area for each variant:

```rust
pub trait HasArea {
    fn area(self) -> f64;
}

impl HasArea for Circle {
    fn area(self) -> f64 {
        PI * self.radius * self.radius
    }
}

impl HasArea for Rectangle {
    fn area(self) -> f64 {
        self.width * self.height
    }
}
```

This `HasArea` trait is simple and intuitive. Each variant implements the `area` method in the usual Rust way. Notice that we do not hand-implement `HasArea` for the overall `Shape` type, we will do this later on, by using `MatchWithValueHandlers` to help us perform the dispatching.

Although `HasArea` is a plain Rust trait, it is easy to wrap it as a `Computer` provider using the `#[cgp_computer]` macro:

```rust
#[cgp_computer]
fn compute_area<T: HasArea>(shape: T) -> f64 {
    shape.area()
}
```

This generic function works for any type implementing `HasArea` and simply calls the `area` method. Applying `#[cgp_computer]` here generates the `ComputeArea` provider type that can then be used within `ComputeShapeArea`.

#### `ToFieldsHandler`

To simplify `ComputeShapeArea` further, we need a way to automatically generate the list of extractors passed to `MatchWithHandlers`. Concretely, we want to generate this:

```rust
Product![
    ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
    ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
]
```

Recall that `Shape` implements the `HasFields` trait, which exposes its variants as a type-level sum:

```rust
Sum![
    Field<Symbol!("Circle"), Circle>,
    Field<Symbol!("Rectangle"), Rectangle>,
]
```

This means we can programmatically extract the tags from `Shape::Fields` and replace each variant with an `ExtractFieldAndHandle` wrapper. We can perform this transformation entirely at the type level using a helper trait, `ToFieldHandlers`, defined as:

```rust
pub trait ToFieldHandlers<Provider> {
    type Handlers;
}

impl<Tag, Value, RestFields, Provider> ToFieldHandlers<Provider>
    for Either<Field<Tag, Value>, RestFields>
where
    RestFields: ToFieldHandlers<Provider>,
{
    type Handlers = Cons<ExtractFieldAndHandle<Tag, Provider>, RestFields::Handlers>;
}

impl<Provider> ToFieldHandlers<Provider> for Void {
    type Handlers = Nil;
}
```

In essence, `ToFieldHandlers` recursively walks through each entry in a type-level sum, replacing `Field<Tag, Value>` with `ExtractFieldAndHandle<Tag, Provider>`, and converts the entire structure into a type-level list.

Using `ToFieldHandlers`, we can now write the `ComputeShapeArea` type as:

```rust
pub type ComputeShapeArea = MatchWithHandlers<
    <<Shape as HasFields>::Fields as
        ToFieldHandlers<HandleFieldValue<ComputeArea>>
    >::Handlers
>;
```

This definition may look complex at first glance. However, it demonstrates the powerful behind-the-scenes transformation that automatically generates the list of variant handlers from `Shape` to be passed to `MatchWithHandlers`.

#### `HasFieldHandlers`

The process to generate variant handlers from `Shape` involves two steps: obtaining `Shape`’s fields from `HasFields`, and then applying `ToFieldHandlers` to those fields. To streamline this, we define another helper trait, `HasFieldHandlers`, that combines these steps:

```rust
pub trait HasFieldHandlers<Provider> {
    type Handlers;
}

impl<Context, Fields, Provider> HasFieldHandlers<Provider> for Context
where
    Context: HasFields<Fields = Fields>,
    Fields: ToFieldHandlers<Provider>,
{
    type Handlers = Fields::Handlers;
}
```

`HasFieldHandlers` unifies the requirements of `HasFields` and `ToFieldHandlers` into a single, convenient trait. This lets us simplify the definition of `ComputeShapeArea` even further:

```rust
pub type ComputeShapeArea = MatchWithHandlers<
    <Shape as HasFieldHandlers<HandleFieldValue<ComputeArea>>>::Handlers
>;
```

With `HasFieldHandlers`, the definition of `ComputeShapeArea` becomes much more concise. Instead of manually combining `HasFields` and `ToFieldHandlers`, we simply rely on `HasFieldHandlers` to generate them from `Shape` and pass the result to `MatchWithHandlers`.

More importantly, this pattern is entirely general: it can be applied to any input type that implements `HasFields`, not just `Shape`, and to any `Computer` provider, not just `ComputeArea`.

### `MatchWithValueHandlers`

The traits `HasFieldHandlers` and `ToFieldHandlers` serve as the helpers for us to perform *type-level metaprogramming* for us to implement high-level visitor dispatchers such as `MatchWithValueHandlers`, which is defined as follows:

```rust
pub type MatchWithValueHandlers<Provider> =
    UseInputDelegate<MatchWithFieldHandlersInputs<HandleFieldValue<Provider>>>;

delegate_components! {
    <Input: HasFieldHandlers<Provider>, Provider>
    new MatchWithFieldHandlersInputs<Provider> {
        Input: MatchWithHandlers<Input::Handlers>
    }
}
```

The design of `MatchWithValueHandlers` is similar to how we implemented the builder dispatchers using [type-level metaprogramming in part 3](/blog/extensible-datatypes-part-3/#type-level-metaprogramming). In this case, `MatchWithValueHandlers` is parameterized by a `Provider` that is expected to implement `Computer`, such as `ComputeArea`. The implementation of `MatchWithValueHandlers` is simply a type alias to use `UseInputDelegate` to dispatch the `Input` type given through `Computer` to `MatchWithFieldHandlersInputs`.

The implementation of `MatchWithFieldHandlersInputs` is defined through `delegate_components!`, with it having a generic mapping for any `Input` that implements `HasFieldHandlers<Provider>`. It then simply delegates the provider for that input to `MatchWithHandlers<Input::Handlers>`.

#### Example Instantiation of `MatchWithValueHandlers`

Because `MatchWithValueHandlers` and `MatchWithFieldHandlersInputs` rely on type-level metaprogramming, it can be difficult to grasp exactly how they work on first encounter. To make things more concrete, let’s walk through how this abstraction is applied to `Shape`. With the machinery we’ve built, the definition of `ComputeShapeArea` becomes as simple as:

```rust
pub type ComputeShapeArea = MatchWithValueHandlers<ComputeArea>;
```

This version of `ComputeShapeArea` is remarkably concise. It no longer mentions `Shape` directly, because it works with *any* compatible input type, including both `Shape` and extensions like `ShapePlus`.

Under the hood, this type alias resolves to `MatchWithHandlers` through the following steps:

* `MatchWithValueHandlers<ComputeArea>` expands to `UseInputDelegate<MatchWithFieldHandlersInputs<HandleFieldValue<ComputeArea>>>`.
* When the `Input` is `Shape`, `UseInputDelegate` locates the corresponding `DelegateComponent` mapping for `Shape` in `MatchWithFieldHandlersInputs<HandleFieldValue<ComputeArea>>`.
* That mapping exists because `Shape` implements `HasFieldHandlers<HandleFieldValue<ComputeArea>>`. As we saw earlier, this expands to:
  ```rust
  Product![
      ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
      ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
  ]
  ```
* This type-level list is then passed to `MatchWithHandlers`, which performs the variant dispatch using the logic we’ve already explored.

### Implementing `HasArea` for `Shape`

With `MatchWithValueHandlers`, implementing the `HasArea` trait for `Shape` becomes straightforward:

```rust
impl HasArea for Shape {
    fn area(self) -> f64 {
        MatchWithValueHandlers::<ComputeArea>::compute(&(), PhantomData::<()>, self)
    }
}
```

As in earlier examples, we use `()` for both the context and code parameters, since `ComputeArea` is generic over any `Context` and `Code`. Aside from this boilerplate, `MatchWithValueHandlers<ComputeArea>` takes care of automatically dispatching to the correct variant handler through `MatchWithHandlers`.

While `Shape` only has two variants, one of the major advantages of `MatchWithValueHandlers` is that it scales effortlessly to enums with many variants and more complex computations. For example, implementing `HasArea` for `ShapePlus` is just as simple:

```rust
impl HasArea for ShapePlus {
    fn area(self) -> f64 {
        MatchWithValueHandlers::<ComputeArea>::compute(&(), PhantomData::<()>, self)
    }
}
```

Although some boilerplate still remains, this approach is significantly simpler than manually matching each variant or relying on procedural macros. It also brings more flexibility and type safety. In the future, CGP may provide more ergonomic abstractions on top of this pattern, making common use cases like `HasArea` even easier to express.

### Dispatching to Context

In the earlier definition of `MatchWithValueHandlers`, we omitted one detail that it has a default type parameter for `Provider`:

```rust
pub type MatchWithValueHandlers<Provider = UseContext> =
    UseInputDelegate<MatchWithFieldHandlersInputs<HandleFieldValue<Provider>>>;
```

This means that if no generic parameter is specified, `MatchWithValueHandlers` will default to using `UseContext` as the provider for `Computer`. If you’ve read the earlier blog posts, you may recall that `UseContext` is a generic provider that delegates its behavior to the consumer trait implementation from the given context. For example, the implementation of `Computer` for `UseContext` looks like this:

```rust
#[cgp_provider]
impl<Context, Code, Input> Computer<Context, Code, Input> for UseContext
where
    Context: CanCompute<Code, Input>,
{
    type Output = Context::Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: Input) -> Self::Output {
        context.compute(code, input)
    }
}
```

Using `UseContext` as the provider for dispatchers like `MatchWithValueHandlers` is especially useful because it enables the concrete context to define how each variant should be handled. For example, we can implement an `App` context that provides `Computer` implementations for various shapes like `Shape` and `ShapePlus`:

```rust
#[cgp_context]
pub struct App;

delegate_components! {
    AppComponents {
        ComputerComponent: UseInputDelegate<new AreaComputers {
            [
                Circle,
                Rectangle,
                Triangle,
            ]:
                ComputeArea,
            [
                Shape,
                ShapePlus,
            ]: MatchWithValueHandlers,
        }>
    }
}
```

In this example, the `App` context uses `UseInputDelegate` to define how to compute areas. Variants like `Circle`, `Rectangle`, and `Triangle` are directly handled by `ComputeArea`. For `Shape` and `ShapePlus`, we use `MatchWithValueHandlers` without specifying a `Provider`, which means it defaults to `UseContext`.

With this setup, when `MatchWithValueHandlers` dispatches to the individual variants, it delegates to `App` itself instead of calling `ComputeArea` directly. This allows us to override the behavior of individual variants easily. For instance, if we want to replace the implementation for `Circle` with an optimized version, we can simply change the wiring in `AppComponents`:

```rust
delegate_components! {
    AppComponents {
        ComputerComponent: UseInputDelegate<new AreaComputers {
            Circle: OptimizedCircleArea,
            ...
        }>
    }
}
```

#### The Flexibility of `UseContext`

This kind of customization would be much harder to achieve if the dispatcher were tightly coupled to a concrete trait implementation, such as using `MatchWithValueHandlers<ComputeArea>` directly. In that case, the only way to change the behavior would be to modify the `HasArea` implementation for `Circle`, which would require ownership of either the `Circle` type or the `HasArea` trait.

While this level of indirection may seem unnecessary for a simple example like computing the area of a shape, it becomes crucial in more complex scenarios, such as the modular interpreter design discussed in [part 2](/blog/extensible-datatypes-part-2).

By routing the variant handling through `UseContext`, we also retain the flexibility to override the provider entirely. That means we can use `MatchWithValueHandlers<ComputeArea>` in cases where we don’t want to go through a `Context` at all, such as using `()` as the context. This optional `Provider` parameter gives us the best of both worlds: we can let the context provide the necessary wiring when needed, or directly specify a concrete provider when that makes more sense.

This pattern of using a provider parameter that defaults to `UseContext` is a recurring design strategy in CGP. It offers a powerful level of control, letting developers customize behavior in a modular and extensible way depending on their use case.

---

## Visitor Dispatcher by Reference

In the earlier examples, some careful readers may have noticed a significant flaw in the function signatures for computing the area of shapes, such as in `HasArea::area`. These methods require *owned* values of the shape variants, meaning that each time we compute the area, we must consume the shape entirely. This is not ideal, especially when we only need a reference and want to preserve the original value.

We started with the ownership-based visitor dispatcher because it is conceptually simpler. It avoids the need to reason about lifetimes, making it easier to understand the overall implementation of extensible visitor. However, in this section, we will show how a reference-based visitor dispatcher can be built *on top of* the ownership-based version. Rather than requiring a separate mechanism, the reference-based version is actually a *specialization* of the existing system.

We will now walk through how to implement the reference-based visitor dispatcher in detail. By the end, you will see how Rust’s type system enables us to safely and cleanly extend the original approach to support references, without compromising lifetime safety or clarity.

### Reference-Based Area Computation

To demonstrate how reference-based visitor dispatch works, let’s define a new trait `HasAreaRef` that computes the area using a shared reference:

```rust
pub trait HasAreaRef {
    fn area(&self) -> f64;
}

impl HasAreaRef for Circle {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
}

impl HasAreaRef for Rectangle { ... }
impl HasAreaRef for Triangle { ... }
```

In practice, you probably wouldn’t need both `HasArea` and `HasAreaRef`, but for clarity we use a separate trait here to clearly distinguish between ownership-based and reference-based computations.

Next, we define a new provider `ComputeAreaRef` using `#[cgp_computer]`, which implements `ComputerRef` by calling `HasAreaRef`:

```rust
#[cgp_computer]
fn compute_area_ref<T: HasAreaRef>(shape: &T) -> f64 {
    shape.area()
}
```

With this in place, we can now implement `HasAreaRef` for `Shape` by using `MatchWithValueHandlersRef`, the reference-based counterpart to `MatchWithValueHandlers`:

```rust
impl HasAreaRef for Shape {
    fn area(&self) -> f64 {
        MatchWithValueHandlersRef::<ComputeAreaRef>::compute_ref(&(), PhantomData::<()>, self)
    }
}
```

Likewise, the implementation for `ShapePlus` follows the same pattern:

```rust
impl HasAreaRef for ShapePlus {
    fn area(&self) -> f64 {
        MatchWithValueHandlersRef::<ComputeAreaRef>::compute_ref(&(), PhantomData::<()>, self)
    }
}
```

At first glance, using `MatchWithValueHandlersRef` to enable reference-based dispatch may seem straightforward — and in many ways, it is. As we’ll see next, the core logic mirrors the ownership-based version closely, with only a few additional considerations around generic lifetimes.

### `PartialRef` Variants

Although most of the higher-level support for reference-based extensible visitors is relatively straightforward, we first need to generate reference-aware partial variants within `#[derive(ExtractField)]`. For example, for the `Shape` enum, the macro generates the following reference-based partial variants:

```rust
pub enum PartialRefShape<'a, F0: MapType, F1: MapType> {
    Circle(F0::Map<&'a Circle>),
    Rectangle(F1::Map<&'a Rectangle>),
}
```

Compared to the owned version `PartialShape`, the `PartialRefShape` definition introduces a lifetime parameter `'a`, and each of its fields now contains a reference with that lifetime. This allows us to safely operate on borrowed variants without taking ownership.

We need a distinct `PartialRefShape` type rather than reusing `PartialShape` because Rust currently has no native mechanism to generically express a value that is either owned or borrowed based on some type-level condition. For instance, if Rust had a concept like a special `'owned` lifetime where `&'owned T` could be treated as just `T`, then it might be possible to unify the two representations. But such a feature does not exist, and arguably shouldn't.

Given that limitation, the cleanest solution is to define a separate enum that introduces a lifetime parameter and holds references explicitly. Since these types are generated by macros and used internally within CGP's dispatching infrastructure, the added complexity is well-contained and does not burden the end user.

#### `HasExtractorRef` Trait

In addition to the partial-ref variants, we need a new trait called `HasExtractorRef` that extracts data from a reference to the full enum:

```rust
pub trait HasExtractorRef {
    type ExtractorRef<'a>
    where
        Self: 'a;

    fn extractor_ref<'a>(&'a self) -> Self::ExtractorRef<'a>;
}
```

Compared to the `HasExtractor` trait, `HasExtractorRef` introduces a generic associated type `ExtractorRef` that is parameterized by a lifetime `'a` and requires the constraint `Self: 'a`. It also defines an `extractor_ref` method that takes a `&'a self` reference and returns the corresponding partial-ref variants as `ExtractorRef<'a>`.

Other than the addition of the lifetime parameter, implementing `HasExtractorRef` for `Shape` is straightforward, as shown below:

```rust
impl HasExtractorRef for Shape {
    type ExtractorRef<'a> = PartialRefShape<'a, IsPresent, IsPresent>
    where Self: 'a;

    fn extractor_ref<'a>(&'a self) -> Self::ExtractorRef<'a> {
        match self {
            Self::Circle(value) => PartialRefShape::Circle(value),
            Self::Rectangle(value) => PartialRefShape::Rectangle(value),
        }
    }
}
```

With `extractor_ref`, it is now possible to extract data from a borrowed `Shape` without cloning each variant, enabling efficient reference-based dispatching.

#### `ExtractField` Implementation

Fortunately, beyond the partial-ref variants and the `HasExtractorRef` trait, most other traits can be reused as if we were working with owned values. This works because `PartialRefShape` holds what are effectively "owned" variant values in the form of references like `&'a Circle` and `&'a Rectangle`. For example, we can implement `ExtractField` for `PartialRefShape` like this:

```rust
impl<'a, F1: MapType> ExtractField<Symbol!("Circle")> for PartialRefShape<'a, IsPresent, F1> {
    type Value = &'a Circle;
    type Remainder = PartialShape<'a, IsVoid, F1>;

    fn extract_field(
        self,
        _tag: PhantomData<Symbol!("Circle")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            PartialRefShape::Circle(value) => Ok(value),
            PartialRefShape::Rectangle(value) => Err(PartialRefShape::Rectangle(value)),
        }
    }
}
```

We can reuse traits like `ExtractField` because the associated types such as `Value` do not need to be the owned values themselves — they can be references to those values instead. This lets us treat extensible variants as if they contain references to their fields, allowing us to manipulate them just like owned values.

### `MatchWithHandlersRef`

Because reference-based dispatching relies on `HasExtractorRef`, we also need to adapt downstream constructs like `MatchWithHandlers` to work with references instead of owned values. This adaptation is provided by `MatchWithHandlersRef`, which uses `HasExtractorRef` in place of `HasExtractor`:

```rust
#[cgp_provider]
impl<'a, Context, Code, Input, Output, Remainder, Handlers> Computer<Context, Code, &'a Input>
    for MatchWithHandlersRef<Handlers>
where
    Input: HasExtractorRef,
    DispatchMatchers<Handlers>:
        Computer<Context, Code, Input::ExtractorRef<'a>, Output = Result<Output, Remainder>>,
    Remainder: FinalizeExtract,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: &'a Input) -> Output {
        DispatchMatchers::compute(context, code, input.extractor_ref()).finalize_extract_result()
    }
}
```

In this implementation, `MatchWithHandlersRef` handles `Computer` over a borrowed input `&'a Input`. It requires the input type to implement `HasExtractorRef` so it can extract the appropriate partial-ref variant. The extracted value, which is of type `Input::ExtractorRef<'a>`, is then passed to `DispatchMatchers`, which processes it using the same monadic pipeline as in the owned-value case. After dispatching to the handlers, the `Output` from `Result<Output, Remainder>` is extracted by calling `finalize_extract_result`, which relies on the `Remainder` type to implement `FinalizeExtract`.

One subtle but important point is that `MatchWithHandlersRef` still implements `Computer` rather than `ComputerRef`. The same is true for the handlers invoked through `DispatchMatchers`, which also expect `Computer` implementations. As a result, reference-based visitor dispatching requires an intermediate conversion step. Variant handlers that originally implement `ComputerRef` must first be "lifted" into providers implementing `Computer`.

After constructing the reference-based pipeline, `MatchWithHandlersRef` can then unlift the entire pipeline to implement `ComputerRef`. This layered approach ensures that reference-based dispatching reuses the same infrastructure as the ownership-based version, while preserving type safety and proper lifetime handling.

### `PromoteRef`

In the same way that traits like `ExtractField` can operate on borrowed fields, the `Computer` trait can also work with borrowed inputs. In fact, the `#[cgp_computer]` macro expansion for the `compute_area_ref` function produces the following `Computer` implementation:

```rust
impl<Context, Code, T: HasAreaRef> Computer<Context, Code, &T> for ComputeAreaRef {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, shape: &T) -> f64 {
        compute_area_ref(shape)
    }
}
```

Here, the `Computer` implementation for `ComputeAreaRef` accepts any reference `&T` as the input type, as long as `T` implements `HasAreaRef`. This demonstrates that the `Computer` trait itself is flexible enough to handle borrowed inputs directly, without the need for additional traits.

However, to make development more ergonomic, CGP provides the `ComputerRef` trait. Using `ComputerRef` eliminates the need to explicitly write `&T` in input type parameters and avoids the complexity of higher-ranked trait bounds in where clauses or input delegation. This makes `ComputerRef` better suited for working with borrowed inputs in a clean and consistent way.

To bridge `Computer` and `ComputerRef`, CGP offers the `PromoteRef` adapter. This adapter converts a provider that implements `Computer` for borrowed inputs into a `ComputerRef` provider. For example, the `ComputerRef` implementation for `ComputeAreaRef` is defined as follows:

```rust
delegate_components! {
    ComputeAreaRef {
        ComputerRefComponent: PromoteRef<Self>,
    }
}
```

This means that `ComputeAreaRef` implements `ComputerRef` through `PromoteRef<ComputeAreaRef>`, automatically lifting its `Computer` implementation for `&T` into a `ComputerRef` implementation.

#### Promotion from `Computer` to `ComputerRef`

The `PromoteRef` adapter allows a provider that implements `Computer` for borrowed inputs to become a provider that implements `ComputerRef`. Its implementation is as follows:

```rust
#[cgp_provider]
impl<Context, Code, Input, Provider, Output> ComputerRef<Context, Code, Input>
    for PromoteRef<Provider>
where
    Provider: for<'a> Computer<Context, Code, &'a Input, Output = Output>,
{
    type Output = Output;

    fn compute_ref(context: &Context, tag: PhantomData<Code>, input: &Input) -> Self::Output {
        Provider::compute(context, tag, input)
    }
}
```

Here, `PromoteRef` implements `ComputerRef` as long as the inner `Provider` supports a higher-ranked trait bound, meaning it can implement `Computer` for all lifetimes `'a` of `&'a Input`. This pattern hides the complexity of higher-ranked trait bounds, so end users do not need to think about them when using `ComputerRef`.

One important detail is that `PromoteRef` requires the inner `Computer` provider to always produce the same `Output` type for any lifetime `'a`. This means that `PromoteRef` cannot be used if the `Output` type borrows from the input reference, because `ComputerRef` defines a single `Output` type that is independent of the lifetime of the input. This limitation follows naturally from the design of `ComputerRef`. When the output must borrow from the input, the user should implement `Computer` directly instead of using `ComputerRef`.

#### Promotion from `ComputerRef` to `Computer`

`PromoteRef` also works in the opposite direction, allowing a provider that implements `ComputerRef` to become a provider that implements `Computer` for borrowed inputs:

```rust
#[cgp_provider]
impl<Context, Code, Input, Provider> Computer<Context, Code, &Input> for PromoteRef<Provider>
where
    Provider: ComputerRef<Context, Code, Input>,
{
    type Output = Provider::Output;

    fn compute(context: &Context, tag: PhantomData<Code>, input: &Input) -> Self::Output {
        Provider::compute_ref(context, tag, input)
    }
}
```

In this implementation, `PromoteRef` wraps a `Provider` that implements `ComputerRef` and forwards the call to `compute_ref`. As a result, it produces a `Computer` implementation that works with `&Input`.

With these two implementations, `PromoteRef` provides a bidirectional bridge between `Computer` and `ComputerRef`. This flexibility allows a single provider to adapt to whichever trait is more convenient for the task, whether the interface expects `Computer` or `ComputerRef`.

### `MatchWithValueHandlersRef`

To support reference-based dispatching in CGP, we only need to introduce a few reference-specific constructs while keeping most of the implementation very similar to the original `MatchWithValueHandlers`. The definition of `MatchWithValueHandlersRef` is shown below:

```rust
pub type MatchWithValueHandlersRef<Provider> =
    UseInputDelegate<MatchWithFieldHandlersInputsRef<HandleFieldValue<PromoteRef<Provider>>>>;

delegate_components! {
    <Input: HasFieldHandlers<Provider>, Provider>
    new MatchWithFieldHandlersInputsRef<Provider> {
        Input:
            PromoteRef<MatchWithHandlersRef<Input::Handlers>>
    }
}
```

When comparing this definition to `MatchWithValueHandlers`, the most notable differences are the use of `MatchWithFieldHandlersInputsRef` and the additional wrapping with `PromoteRef` to facilitate conversions between `Computer` and `ComputerRef`.

#### Example use of `MatchWithValueHandlersRef`

To better understand how `MatchWithValueHandlersRef` works in practice, let us walk through what happens when we call `MatchWithValueHandlersRef<ComputeAreaRef>` on `Shape`:

```rust
impl HasAreaRef for Shape {
    fn area(&self) -> f64 {
        MatchWithValueHandlersRef::<ComputeAreaRef>::compute_ref(&(), PhantomData::<()>, self)
    }
}
```

The `Provider` argument to `MatchWithHandlers` is `ComputeAreaRef`, which is expanded into `HandleFieldValue<PromoteRef<ComputeAreaRef>>` when passed to `MatchWithFieldHandlersInputsRef`. The `UseInputDelegate` type is expected to implement `ComputerRef` for `Input = Shape`, and it delegates the work to `MatchWithFieldHandlersInputsRef`.

Next, `MatchWithFieldHandlersInputsRef` is invoked with `Input` as `Shape` and `Provider` as `HandleFieldValue<PromoteRef<ComputeAreaRef>>`. The `Shape` type must implement `HasFieldHandlers<HandleFieldValue<PromoteRef<ComputeAreaRef>>>`, and its `Handlers` expand to:

```rust
Product![
    ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<PromoteRef<ComputeAreaRef>>>,
    ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<PromoteRef<ComputeAreaRef>>>,
]
```

The delegate entry maps to `PromoteRef<MatchWithHandlersRef<Input::Handlers>>`, which becomes:

```rust
PromoteRef<MatchWithHandlersRef<Product![
    ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<PromoteRef<ComputeAreaRef>>>,
    ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<PromoteRef<ComputeAreaRef>>>,
]>>
```

In order to implement `ComputerRef`, `PromoteRef<MatchWithHandlersRef<Input::Handlers>>` requires `MatchWithHandlersRef<Input::Handlers>` to implement `Computer`. For `MatchWithHandlersRef<Input::Handlers>` to implement `Computer`, its inner provider `HandleFieldValue<PromoteRef<ComputeAreaRef>>` must satisfy the following constraints:

* `Computer<(), (), Field<Symbol!("Circle"), &Circle>>`
* `Computer<(), (), Field<Symbol!("Rectangle"), &Rectangle>>`

After `HandleFieldValue` unwraps the actual field values, the inner provider `PromoteRef<ComputeAreaRef>` must implement:

* `Computer<(), (), &Circle>`
* `Computer<(), (), &Rectangle>`

Finally, `PromoteRef` requires `ComputeAreaRef` to implement:

* `ComputerRef<(), (), Circle>`
* `ComputerRef<(), (), Rectangle>`

This example shows that the key to implementing `MatchWithValueHandlersRef` lies in understanding when to use `Computer` versus `ComputerRef`, and where to apply `PromoteRef` to bridge the two. While the type expansion can appear intimidating, most of the complexity is hidden behind these abstractions.

In practice, the usage of `MatchWithValueHandlersRef` feels almost identical to the ownership-based version, and the underlying implementation shares the same structure aside from a few reference-specific details.

---

## Future Work

The modular design of extensible variants makes it straightforward to extend the pattern for new use cases. There are several scenarios that are not yet supported in this initial version. While none of these are technically difficult to implement, the focus for this release has been on the core functionality and the writing of these blog posts. The following areas are planned for future work.

### Additional Arguments

At present, extensible visitors do not support forwarding additional arguments to individual visitor handlers. This limitation prevents traits that require extra arguments, such as:

```rust
pub trait HasArea {
    fn area(&self, scale_factor: f64) -> f64;
}
```

Here, the `area` method needs a `scale_factor` argument that must be passed through the visitor dispatcher to the variant handlers. To support this, we can create adapters similar to `ExtractFieldAndHandle` that bundle the extra arguments into the `Input`. We would then define alternative dispatchers, similar to `MatchWithValueHandlers`, which operate on these bundled inputs.

### `&mut` References

The current reference-based dispatch system is hard-coded to use shared references (`&`). As a result, it does not support `&mut` references for mutable operations such as:

```rust
pub trait CanScale {
    fn scale(&mut self, factor: f64);
}
```

To support mutable references, the design of partial-ref variants needs to be generalized to work with both `&` and `&mut`. This likely requires an abstraction similar to `MapType`, but for mapping the type of reference used for each field.

#### Simpler Dispatchers

Although extensible visitors were designed with complex use cases like modular interpreters in mind, they are equally powerful for simpler needs, such as implementing plain Rust traits like `HasArea`. While this is already possible with the current infrastructure, the ergonomics leave much to be desired.

Users must first understand and use the `#[cgp_computer]` macro to define helper providers like `ComputeArea`, and then manually implement the trait by invoking `MatchWithValueHandlers::<ComputeArea>::compute()` with dummy context and code. For those unfamiliar with CGP, these steps impose unnecessary friction and cognitive load.

To improve usability, CGP could offer a procedural macro to automate this boilerplate. For instance, a trait could be annotated as follows:

```rust
#[cgp_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}
```

The `#[cgp_dispatch]` macro would parse the trait definition and generate the necessary code to integrate it with the extensible visitor framework. This includes generating a blanket implementation for the trait so that it is automatically implemented for compatible enums like `Shape` or `ShapePlus`.

The generated implementation would resemble:

```rust
impl<Context> HasArea for Context
where
    Context: HasExtractorRef,
    MatchWithValueHandlersRef<ComputeArea>: ComputerRef<(), (), Context, Output = f64>,
{
    fn area(&self) -> f64 {
        MatchWithValueHandlersRef::<ComputeArea>::compute_ref(&(), PhantomData, self)
    }
}
```

With such a macro in place, using extensible visitors to implement common traits would become as easy as annotating the trait with `#[cgp_dispatch]`, removing the need to understand the inner workings of CGP for simple use cases.

### Custom Partial Records Updater

Currently, partial records only support a small set of operations like `TakeField` and `BuildField`. This makes it difficult to customize behavior, such as overriding existing field values, filling empty fields with defaults, or taking a default value from an empty field.

To support these scenarios, more generalized interfaces for interacting with partial records are needed. A promising approach is to use *natural transformations* to implement generic field transformers. For example, a builder transformer would convert `IsNothing` fields into `IsPresent` fields, while an overrider transformer would convert either `IsNothing` or `IsPresent` fields into `IsPresent` fields. This would allow for flexible and reusable field manipulation strategies.

### Explanation for Computation Hierarchy

Beyond `Computer`, `ComputerRef`, and `Handler`, CGP defines several other traits that represent different kinds of computations. For example, `TryComputer` supports computations that may fail, and `TryComputerRef` handles fallible computations that operate on reference inputs. These traits make it possible to model a wide range of behaviors, from straightforward value computations to error-aware or reference-based processing.

CGP also provides constructs such as `Promote` that allow seamless conversion between different types of computation providers. In addition, it supports multiple ways to compose these providers. Two notable examples are `PipeHandlers` and `PipeMonadic`. Monadic composition in particular requires delicate explanation, because CGP’s approach to monads does not behave exactly like the familiar monads in Haskell. Understanding how these monadic pipelines operate is essential for developers who want to create more sophisticated and composable computation flows.

My original plan was to dedicate a fifth part of this series to explain the complete hierarchy of computation traits in CGP. However, the implementation details for extensible data types have already required extensive coverage, and attempting to include computation hierarchy in the same series would make it overwhelming. As a result, I have decided to split that explanation into its own dedicated post, or potentially a separate series, to provide the depth and clarity it deserves.

---

## Conclusion

We have now reached the end of our deep dive into the implementation details of extensible variants and extensible visitors. To recap the journey, we began by defining the `FromVariant` trait, which allows constructing an enum from one of its variants. We then introduced partial variants, which mirror the structure of partial records but use `IsVoid` to indicate the absence of a variant.

From there, we defined the `HasExtractor` trait to convert an enum into its partial variants, followed by the `ExtractField` trait to extract a single variant from those partial variants. This led to the concept of remainders, representing what is left after a variant is extracted, and the `FinalizeExtract` trait, which finalizes a remainder once all its variants have been handled.

We then examined how upcasting and downcasting for enums are implemented. We explored how the `HasFields` implementation for enums represents a type-level sum of fields and how `FieldExtractor` is used to move fields between source and target partial variants. The difference between `CanUpcast` and `CanDowncast` boils down to choosing whether the `HasFields` implementation comes from the source or the target enum.

Next, we delved into the implementation of extensible visitors, beginning with `MatchWithHandlers` and `DispatchMatchers`. We saw that `DispatchMatchers` is structured as a monadic pipeline that short-circuits when it encounters an `Ok` value. We examined the role of field adapters like `ExtractFieldAndHandle` and `HandleFieldValue`, and we explored how `#[cgp_computer]` transforms a regular trait method into a `Computer` provider. We then discussed how `ToFieldsHandler` and `HasFieldHandlers` convert the tags in an enum’s `HasFields` implementation into the appropriate providers for use with `MatchWithHandlers`. Finally, we looked at how the top-level dispatcher `MatchWithValueHandlers` is assembled through type-level metaprogramming, combining all the earlier components into a cohesive system.

We concluded with the reference-based implementation of extensible visitors. This introduced reference-specific constructs such as partial-ref variants and the `HasExtractorRef` trait. We examined how `MatchWithHandlersRef` uses `HasExtractorRef` to extract borrowed variants, and how `PromoteRef` bridges between `Computer` and `ComputerRef` providers. We then saw that `MatchWithValueHandlersRef` is implemented with only minimal differences from its ownership-based counterpart, relying on `MatchWithHandlersRef` and `PromoteRef` to interleave `Computer` and `ComputerRef` computations.

### End of Series

We have reached the conclusion of this series on extensible data types. By now, you should have a clearer understanding of the design patterns that extensible data types make possible and how they can be applied to solve real-world problems in Rust.

Although some of the implementation details can be challenging, I hope this series has given you a solid sense of how extensible data types are structured, and why a type-driven approach allows our system to remain both modular and flexible as it grows.

More importantly, I hope these articles have helped you recognize the design patterns that underpin CGP. Learning to identify and apply these patterns will make your own CGP code more effective and give you tools you can use well beyond this particular topic.

The design and implementation of extensible data types push the boundaries of what CGP can achieve. They show how CGP can model advanced language features that are often only possible through direct integration into the Rust compiler. In day-to-day development, you may not need to reach for every advanced technique demonstrated here, but understanding that these patterns exist will broaden your perspective on what is possible.

Even if you have not fully absorbed every concept presented, I hope this series has inspired you to begin learning CGP from the fundamentals. It is important to realize that many of the basic CGP patterns may initially seem unnecessary or overengineered, yet they are the foundation that makes advanced patterns like extensible data types and [Hypershell](/blog/hypershell-release/) achievable.

Finally, you do not need to create entirely new language features or DSLs for CGP to prove valuable. In upcoming posts, we will explore more foundational and intermediate CGP patterns that can help you build practical and maintainable Rust applications. Thank you for following this series and for your support of the CGP project. Exciting developments are on the horizon, and I look forward to sharing them with you.
