+++

title = "Programming Extensible Data Types in Rust with CGP - Part 4: Implementing Extensible Variants"

description = ""

authors = ["Soares Chen"]

+++


# Implementation of Extensible Variants

Now that we've covered how extensible records work in CGP, we can turn our attention to **extensible variants**. At first glance, it might seem like a completely different mechanism — but surprisingly, the approach used to implement extensible variants is very similar to that of extensible records. In fact, many of the same principles apply, just in the “opposite direction”.

This close relationship between records and variants is rooted in **category theory**. In that context, records are known as **products**, while variants (or enums) are referred to as **sums** or **coproducts**. These terms highlight a deep **duality** between the two: just as products *combine* values, coproducts represent a *choice* among alternatives. CGP embraces this theoretical foundation and leverages it to create a unified design for both extensible records and extensible variants.

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

## Restrictions on Enum Shape

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

These more complex variants are not supported because they would make it harder to represent variant fields as simple types, which would, in turn, lead to less ergonomic APIs. By restricting each variant to a single unnamed field, CGP ensures that types like `FromVariant::Value` remain straightforward and intuitive.

If you need to represent more complex data in a variant, we recommend wrapping that data in a dedicated struct. This way, you can still take advantage of CGP's extensible variant system while maintaining type clarity and composability.

## Partial Variants

Just as CGP supports partially constructed structs through *partial records*, it also enables **partial variants** to work with **partially deconstructed** enums in a similarly flexible way. Partial variants allow you to pattern match on each variant of an enum incrementally, while safely excluding any variants that have already been handled. This makes it possible to build exhaustive and type-safe match chains that evolve over time.

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

Just like `FromVariant` and `HasField`, the `ExtractField` trait takes a `Tag` type to identify the variant, and includes an associated `Value` type representing the variant’s inner data. Additionally, it defines a `Remainder` type, which represents the **remaining** variants that have not yet been matched.

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
pub fn compute_area(shape: Shape) -> f64 {
    match shape
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
    }
}
```

In this example, we begin by converting the `Shape` value into a `PartialShape` with all variants present using `to_extractor`. We then call `extract_field` to try extracting the `Circle` variant. If successful, we compute the circle's area. If not, we receive a remainder value where the `Circle` variant is now marked as `IsVoid`. This remainder is then used to attempt extracting the `Rectangle` variant. If that succeeds, we compute the area accordingly.

By the time we reach the second `Err` case, the remainder has the type `PartialShape<IsVoid, IsVoid>`, which cannot contain any valid variant. Because of this, we can safely omit any further pattern matching, and the compiler guarantees that there are no unreachable or unhandled cases.

What makes this approach so powerful is that the Rust type system can statically verify that it is impossible to construct a valid value for `PartialShape<IsVoid, IsVoid>`. We no longer need to write boilerplate `_ => unreachable!()` code or use runtime assertions. The type system ensures exhaustiveness and soundness entirely at compile time, enabling safer and more maintainable implementation of extensible variants.

## Short Circuiting Remainder

The earlier implementation of `compute_area` involves the use of nested `match` on the `Result` returned from `extract_field`. If you are familiar with the use of `?` on `Result`, you might wonder why don't we use it in the method implementation.

The main reason `?` wouldn't work with our example is because we want to short circuit and return the `Ok` variant early, while the remainder type in the `Err` variant changes in each call to `extract_field`. This is the *inverse* of how we usually expect `Result` to be used, where the `Err` variant is returned early, while the type in the `Ok` variant changes with each method call.

To really simplify the code, we can imagine it to be written as the following pseudocode:

```rust
pub fn compute_area(shape: Shape) -> Result<f64, Infallible> {
    let remainder = shape
        .to_extractor()
        .extract_field(PhantomData::<symbol!("Circle")>)
        .map(|circle| PI * circle.radius * circle.radius)⸮;

    let remainder = remainder
        .extract_field(PhantomData::<symbol!("Rectangle")>)
        .map(|rectangle| rectangle.width * rectangle.height)⸮;

    match remainder {}
}
```

In the above pseudocode, we *invent* a new operator `⸮`, which behaves in the opposite way as `?`. It short circuits and return the `Ok` variant early, or binds the remainder value in the `Err` variant to the `remainder` variable in the `let` binding.

In the example, we use `.map` to map the `Ok` variant from the shape to the `f64` area result, and return early through `⸮`. Otherwise, the `remainder` variable gradually shrinks, until it becomes inhabitable in the end. We then use `match remainder {}` to statically assert that the case cannot be reach.

In other words, our new `compute_area` function don't really "return" anything in the `Err` variant. But to complete the type signature, we use `Result<f64, Infallible>` as the return type, with `Infallible` representing that the operation can never fail with an `Err` variant.

At this point, some readers may point out that there are other ways to simplify the Rust code without introducing the `⸮` operator. For example, we could make `extract_field` return `Result<Remainder, Value>` so that we can short circuit `Err(Value)` through `?`. However, doing so may be confusing, as it makes more sense that `Remainder` is the "exceptional" case here.

Furthermore, we introduce `⸮` as an exercise for you to think about how it relates to other existing operators such as `?`, `.await`, and `.await?`. In practice, we won't need Rust to officially support `⸮`, nor would we need to manually implement functions like `compute_area`. Instead, the extensible visitor pattern would apply something like `⸮` for us in the implementation to automatically implement `compute_area` for us.

We will revisit this topic in the later section about the implementation of extensible visitor. For now, let's move on to how we finalize an empty remainder.

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

With the foundational traits for extensible variants in place, we can now explore how to implement the `CanUpcast` and `CanDowncast` traits. These traits enable safe and generic upcasting and downcasting between enums that share compatible variants.

## `HasFields` Implementation

Just as extensible records rely on `HasFields` for iterating over their fields, extensible variants use a similar mechanism to iterate over their variants. This allows the generic casting implementation to iterate over each variant of an enum.

For example, the `HasFields` implementation for the `Shape` enum is defined as follows:

```rust
impl HasFields for Shape {
    type Fields = Sum![
        Field<symbol!("Circle"), Circle>,
        Field<symbol!("Rectangle"), Rectangle>,
    ];
}
```

Here, instead of using the `Product!` macro (which is used for structs), we use the `Sum!` macro to build a *type-level sum* representing all variants in the enum. The `Sum!` macro expands to a nested structure of `Either`, similar to how `Product!` expands into a chain of `Cons`.

For example, the `Sum!` expression above desugars to:

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

Where `Either` is defined in a similar fashion to Rust's standard `Result` type, but with variant names that reflect the sum type structure:

```rust
pub enum Either<A, B> {
    Left(A),
    Right(B),
}
```

In this way, we represent the enum's variants as a nested sum, with `Void` as the terminating type to signify the end of the variant choices.

## `CanUpcast` Implementation

With `HasFields` implemented, we are ready to define the `CanUpcast` trait. This trait allows an enum to be upcast to another enum that includes a subset of its variants:

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
        match Context::Fields::extract_from(self.to_extractor()) {
            Ok(target) => target,
            Err(remainder) => remainder.finalize_extract(),
        }
    }
}
```

Here’s how it works. First, the `Context` type (the source enum) must implement both `HasFields` and `HasExtractor`. The `HasFields` trait provides a type-level sum of variants, and `HasExtractor` converts the enum into its corresponding partial variants. Next, the associated `Fields` type must implement the helper trait `FieldsExtractor`, which handles the actual extraction of variants into the target type. The `Remainder` returned by this operation must then implement `FinalizeExtract`, which guarantees that all source variants have been accounted for.

In the method body, we begin by calling `self.to_extractor()` to convert the source enum into a value with partial variants. We then use `Fields::extract_from` to extract the relevant variants into the target enum. Finally, we call `finalize_extract()` on the remainder.

## `FieldsExtractor` Trait

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

## Example Use of `Upcast`

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
      Field<symbol!("Circle"), Circle>,
      Field<symbol!("Rectangle"), Rectangle>,
  ]
  ```
* `Shape` must also implement `HasExtractor`, with its associated extractor being `PartialShape<IsPresent, IsPresent>`.
* The `Fields` type must implement `FieldsExtractor`, with `PartialShape<IsPresent, IsPresent>` as the source and `ShapePlus` as the target.
* The result of the extraction yields a remainder of type `PartialShape<IsVoid, IsVoid>`, which in turn implements `FinalizeExtract`.

Next, the `FieldsExtractor` implementation for the head of the sum begins processing:

* The current `Tag` is `symbol!("Circle")`, and the associated `Value` is of type `Circle`.
* The `Source` is `PartialShape<IsPresent, IsPresent>`, and the `Target` is `ShapePlus`.
* The source implements `ExtractField<symbol!("Circle")>`, which succeeds with `Circle` as the extracted value and `PartialShape<IsVoid, IsPresent>` as the remainder.
* The target `ShapePlus` implements `FromVariant<symbol!("Circle")>`, again using the `Circle` type.

The extractor then proceeds to the next variant in the sum:

* The current `Tag` is `symbol!("Rectangle")`, with `Rectangle` as the `Value`.
* The updated `Source` is now `PartialShape<IsVoid, IsPresent>`, and the `Target` remains `ShapePlus`.
* This source implements `ExtractField<symbol!("Rectangle")>`, yielding `Rectangle` as the value and `PartialShape<IsVoid, IsVoid>` as the final remainder.
* The target once again supports `FromVariant<symbol!("Rectangle")>` using the matching `Rectangle` type.
* At the end of the chain, the `Void` variant is reached. The `FieldsExtractor` implementation for `Void` simply returns the remainder, which in this case is `PartialShape<IsVoid, IsVoid>`.

What this process shows is that the `Upcast` operation works by examining each variant in the source type `Shape`, extracting each present value, and reinserting it into the target type `ShapePlus`. Once all fields have been processed, the remaining variants are guaranteed to be uninhabited. At that point, we can safely discharge the remainder using the `FinalizeExtract` trait.

By breaking down the upcast into individual type-driven steps over extensible variants, we can implement upcasting entirely in safe Rust. Even more importantly, this implementation is fully generic and reusable. We are not writing code solely for the purpose of supporting `Upcast` — instead, we are building a reusable foundation that also supports operations like `Downcast` and other generic manipulations over extensible variants.

## `CanDowncast` Implementation

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

## Example Use of Downcast

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
    Err(remainder) => match remainder.extract_field(PhantomData::<symbol!("Triangle")>) {
        Ok(triangle) => triangle.base * triangle.height / 2.0,
    },
};
```

In this example, we start with a `ShapePlus` value that holds a `Triangle`. We then call `downcast`, attempting to convert it to a `Shape`, which does not include the `Triangle` variant. Internally, the downcast operation uses `Shape::Fields` to iterate over the variants defined in `Shape` and tries to extract each from the original `ShapePlus` value. If any of those variants are found — such as `Circle` or `Rectangle` — the match succeeds and we compute the corresponding area from `Shape`.

However, when the actual variant in this case is `Triangle`, which is not part of `Shape`, the downcast fails and we receive the remainder of the partial variant structure. This remainder, of type `PartialShapePlus<IsPresent, IsVoid, IsVoid>`, contains only the `Triangle` variant. We then use `extract_field` to retrieve the triangle and compute its area. At this point, no other variants remain to be handled.

One of the most impressive aspects of both upcast and downcast is that they work seamlessly even when the source and target enums define their variants in entirely different orders. Because the trait implementations, such as `ExtractField`, operate in a generic and order-independent way, the correctness and behavior of casting are preserved regardless of variant ordering. This level of flexibility makes the CGP approach to extensible variants both powerful and practical for real-world use.

# Visitor Dispatcher

With the traits for extensible variants defined, we can now turn our attention to how CGP implements generalized **visitor dispatchers**, similar to how the [builder dispatchers](/blog/extensible-datatypes-part-3/#builder-dispatcher) in our previous blog post was implemented.

## `MatchWithHandlers` Provider

In the [examples in part 2](/blog/extensible-datatypes-part-2/#dispatching-eval), we have used dispatchers like `MatchWithValueHandlers` and `MatchWithValueHandlersRef` to dispatch the handling of variant values to different handlers based on the `Input` type. Under the hood, these dispatchers are built upon a more fundamental dispatcher called `MatchWithHandlers`, which is implemented as follows:

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

    fn compute(_context: &Context, code: PhantomData<Code>, input: Input) -> Output {
        let res = DispatchMatchers::compute(_context, code, input.to_extractor());

        match res {
            Ok(output) => output,
            Err(remainder) => remainder.finalize_extract(),
        }
    }
}
```

The `MatchWithHandlers` provider is parameterized by a `Handlers` type, which represents a type-level list of visitor handlers that will process the variants from a generic `Input` type. Inside the `Computer` implementation, it requires `Input` to be an enum that implements `HasExtractor`.

Inside the method body for `compute`, it first calls `input.to_extractor` to convert the input into partial variants, and then pass it to a lower level `DispatchMatchers<Handlers>` handler, which performs the actual dispatching logic and returns `Result<Output, Remainder>` as the output. The implementation then expects the `Remainder` that is returned to be inhabitable through `FinalizeExtract`. With that, it simply match on the return result to return the `Output`, and in the error case, it uses `finalize_extract()` to assert that the case cannot be reached.


## `DispatchMatchers` Provider

Recall that in the extensible builder implementation of `BuildWithHandlers`, we used `PipeHandlers` to form a pipeline of builders to gradually fill in the partial records returned by the previous builder handler. For the case of extensible visitors, the pipeline implementation is slightly different, but is still elegantly defined as follows:

```rust
pub type DispatchMatchers<Providers> = PipeMonadic<OkMonadic, Providers>;
```

What this means is that `DispatchMatchers` forms a **monadic** pipeline of visitor handlers, that is based on the monad implementation provided by `OkMonadic`.

## What is a Monad?!

At this point, most of the readers coming from Rust background would probably be confused on *what the heck is a [monad](https://wiki.haskell.org/Monad)*, and what does it have to do with implementing extensible visitors. So in this section, I will try to explain monads in much more simplified and less intimidating ways, using existing Rust concepts.

To put it simply, a monad `M` is typically a container types that "contains" some value type `T` in the form of `M<T>`. We have in fact commonly used various monadic types in Rust, including `Option<T>`, `Result<T, E>`, and `impl Futures<Output = T>`.

In addition to "containing" values, a monad provide a "bind" operation to "extract" the values out of it. In Rust, we have also frequently used this operation through the use of `?`, `.await`, or `.await?`.

Learning all these, we can now understand `PipeMonadic` as simply asking CGP to automatically "apply" operators like `?`, `.await`, or `.await?` to the previous handler in the pipeline, to "extract" the result before passing it as an input to the next handler.

The main strength of the monadic implementation is that we can not only use existing operators like `?`, but also perform similar operation on other types that have similar properties. This includes a composition of the existing types that we are already familiar with, such as `impl Futures<Output = Result<Result<Option<T>, E1>, E2>>`. With monads, you can imagine that we can now implement an operator that is roughly equivalent to `.await???` to "extract" the `T` value out of the nested container.

For the case of `DispatchMatchers`, the "monad provider" we used is called `OkMonadic`. Essentially, this is the `⸮` operator that we have used in the pseudocode in the [`compute_area` example](#short-circuiting-remainder) to short circuit on the `Ok` variant and have a changing remainder type inside `Err`.

So what we are really saying here is that, to implement `DispatchMatchers`, we just need to form a pipeline of handlers, where we use `⸮` to early return an `Ok(output)` from the returned `Result<Output, Remainder>` from the previous handler. Otherwise, we get back `Remainder` from `⸮`, and feed it as the input to the next handler.

Thanks to `PipeMonadic` and `OkMonadic`, we do not need to implement this extraction manually. Instead, we just use the existing facilities provided by CGP to automatically implement this monadic pipeline for us.

If you are still confused at this point, don't worry, as we will walk through a concrete example next. We will also have a blog post in the near future to explain about how CGP implements monads in Rust to support operations like `PipeMonadic`.

## Example Use of `MatchWithHandlers`

```rust
let circle = Shape::Circle(Circle { radius: 5.0 });

let area = MatchWithHandlers::<
    Product![
        ExtractFieldAndHandle<symbol!("Circle"), HandleFieldValue<ComputeArea>>,
        ExtractFieldAndHandle<symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
    ],
>::compute(&(), PhantomData::<()>, circle);
```

# Conclusion
