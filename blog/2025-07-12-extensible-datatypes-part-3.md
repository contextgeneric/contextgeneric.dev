---
slug: extensible-datatypes-part-3
title: 'Programming Extensible Data Types in Rust with CGP - Part 3: Implementing Extensible Records'
authors: [soares]
tags: [deepdive]
---

This is the **third** part of the blog series on **Programming Extensible Data Types in Rust with CGP**. You can read the [first](/blog/extensible-datatypes-part-1) and [second](/blog/extensible-datatypes-part-2) parts here.

At this point, you’ve likely seen how these patterns can make real-world applications more modular and maintainable. If these examples have convinced you of CGP’s practical value, that’s great. But if you still feel the examples are not grounded enough in production use cases, you are welcome to pause here and revisit CGP later.

The next 2 parts of the series are aimed at readers who want to go deeper — those interested in how CGP implements extensible data types under the hood and who might even want to contribute to CGP itself by helping to build the real-world examples you’re looking for.

We will first explore the implementation of extensible records in this part, followed by the implementation of extensible variants in the coming Part 4.

<!-- truncate -->

## Recap

As a recap, we have covered the new release of [**CGP v0.4.2**](https://github.com/contextgeneric/cgp/releases/tag/v0.4.2) which now supports the use of **extensible records and variants**, allowing developers to write code that operates on *any struct containing specific fields* or *any enum containing specific variants*, without needing their concrete definition.

In the first part of the series, [**Modular App Construction and Extensible Builders**](/blog/extensible-datatypes-part-1), we demonstrated an example use of the **extensible builder pattern**, which uses **extensible records** to support modular construction of an application context.

Similarly, in the second part of the series, [**Modular Interpreters and Extensible Visitors**](/blog/extensible-datatypes-part-2), we saw how the modular visitor pattern allows us to implement evaluation and to-Lisp conversion for each variant of a language expression enum using separate visitor providers.

## Discussion

Discuss on [Reddit](https://www.reddit.com/r/rust/comments/1lxv6zr/the_design_and_implementation_of_extensible/), [GitHub](https://github.com/orgs/contextgeneric/discussions/14) or [Discord](https://discord.gg/Hgk3rCw6pQ).

## Series Overview

[**Part 1: Modular App Construction and Extensible Builders**](/blog/extensible-datatypes-part-1) – In this introductory part, we present a high-level overview of the key features enabled by extensible data types. We then dive into a hands-on demonstration showing how extensible records can be used to build and compose modular builders for real-world applications.

[**Part 2: Modular Interpreters and Extensible Visitors**](/blog/extensible-datatypes-part-2) – This part continues the demonstration by introducing extensible variants. We use them to address the [**expression problem**](https://en.wikipedia.org/wiki/Expression_problem), implementing a set of reusable interpreter components for a small toy language.

**Part 3: Implementing Extensible Records** (this post) – Here, we walk through the internal mechanics behind extensible records. We show how CGP supports the modular builder pattern demonstrated in Part 1 through its underlying type and trait machinery.

[**Part 4: Implementing Extensible Variants**](/blog/extensible-datatypes-part-4) – This part mirrors Part 3, but for extensible variants. We examine how extensible variants are implemented, and compare the differences and similarities between extensible records and variants.

## Underlying Theory

The design of extensible data types in CGP is inspired by a rich body of research in the programming languages community. In particular, CGP draws heavily from the implementation techniques found in [**datatype-generic programming in Haskell**](https://wiki.haskell.org/index.php?title=Generics), as well as ideas presented in the paper [**Abstracting extensible data types: or, rows by any other name**](https://dl.acm.org/doi/10.1145/3290325). Concepts similar to CGP’s have also appeared in other languages under names like **row polymorphism** and **polymorphic variants**.

While it might be too academic — or simply too dry — for this post to delve into all the theoretical differences between CGP and these approaches, it’s worth highlighting one essential distinction: CGP’s seamless integration of extensible data types with the CGP system itself. This tight integration sets CGP apart by making the extensibility not just a design principle, but a native part of how you build and scale software in Rust.

### Constraint Propagation Problem

At the heart of CGP lies a powerful solution to a common challenge:

*how can we compose two generic functions that each rely on their own row constraints, and automatically propagate those constraints into the resulting composed function?*

To illustrate this problem, consider the following simple example:

```rust
pub fn first_name_to_string<Context>(context: &Context) -> String
where
    Context: HasField<Symbol!("first_name"), Value: Display>,
{
    context.get_field(PhantomData).to_string()
}

pub fn last_name_to_string<Context>(context: &Context) -> String
where
    Context: HasField<Symbol!("last_name"), Value: Display>,
{
    context.get_field(PhantomData).to_string()
}
```

In this example, we define two functions that extract the `first_name` and `last_name` fields from a generic `Context` type and convert them to strings. The trait `HasField` is used to represent a **row constraint**, where field names are expressed as type-level strings using `Symbol!`.

Now, suppose we want to combine the output of these two functions into a full name string. We can write a function that explicitly concatenates the two results, like so:

```rust
pub fn full_name_to_string<Context>(context: &Context) -> String
where
    Context: HasField<Symbol!("first_name"), Value: Display>
        + HasField<Symbol!("last_name"), Value: Display>,
{
    format!(
        "{} {}",
        first_name_to_string(context),
        last_name_to_string(context)
    )
}
```

This works, but requires us to manually specify all the field constraints in the function signature. This becomes increasingly cumbersome as the number of required constraints grows.

More critically, it becomes difficult to write generic higher-order functions that can accept any two such functions with their own constraints, and automatically compose them into a new function that correctly propagates those constraints. For instance, consider this naive attempt:

```rust
pub fn concate_outputs<Context>(
    fn_a: impl Fn(&Context) -> String,
    fn_b: impl Fn(&Context) -> String,
) -> impl Fn(&Context) -> String {
    move |context| format!("{} {}", fn_a(context), fn_b(context))
}
```

Here, `concate_outputs` takes two functions and returns a new one that calls both and concatenates their results. However, the function signature does not capture the constraints required by `fn_a` and `fn_b`. As a result, when we use this to build a composed function, we still have to restate all of the necessary constraints explicitly:

```rust
pub fn full_name_to_string<Context>(context: &Context) -> String
where
    Context: HasField<Symbol!("first_name"), Value: Display>
        + HasField<Symbol!("last_name"), Value: Display>,
{
    let composed = concate_outputs(first_name_to_string, last_name_to_string);
    composed(context)
}
```

This works within the body of a function, but it prevents us from defining the composed function as a top-level value. In other words, we cannot simply write something like:

```rust
// Invalid export
pub let full_name_to_string = concate_outputs(first_name_to_string, last_name_to_string);
```

and then export that directly from a module. The constraints must still be manually declared somewhere, limiting the expressiveness and reusability of our composition.

### Type-Level Composition

In many programming languages research, solving the problem of constraint-aware function composition typically requires advanced language features like **constraint kinds**. These features allow constraints to be expressed and manipulated at the type level, enabling functions with different requirements to be composed seamlessly. However, languages like Rust do not yet support these advanced capabilities. This limitation has been one of the major reasons why extensible data types have not seen broader adoption in the Rust and other mainstream languages.

The breakthrough that CGP introduces is the ability to perform this kind of composition in Rust today — without waiting for the language to evolve. The key insight is to **represent functions as types**, and then use CGP's system of traits and type-level programming to manage composition and constraint propagation.

To see how this works, let’s begin by transforming two basic functions into CGP providers. Instead of writing traditional functions, we define `FirstNameToString` and `LastNameToString` as types that implement the `ComputerRef` trait:

```rust
#[cgp_new_provider]
impl<Context, Code, Input> ComputerRef<Context, Code, Input> for FirstNameToString
where
    Context: HasField<Symbol!("first_name"), Value: Display>,
{
    type Output = String;

    fn compute_ref(context: &Context, _code: PhantomData<Code>, _input: &Input) -> String {
        context.get_field(PhantomData).to_string()
    }
}

#[cgp_new_provider]
impl<Context, Code, Input> ComputerRef<Context, Code, Input> for LastNameToString
where
    Context: HasField<Symbol!("last_name"), Value: Display>,
{
    type Output = String;

    fn compute_ref(context: &Context, _code: PhantomData<Code>, _input: &Input) -> String {
        context.get_field(PhantomData).to_string()
    }
}
```

In this setup, `FirstNameToString` and `LastNameToString` are no longer standalone functions, but rather types that can be plugged into the CGP system. Each type implements the `ComputerRef` trait and specifies its output as a `String`. For simplicity, we ignore the `Code` and `Input` parameters, as they are not essential for this example.

Once we have our functions defined as types, we can now build a new provider that composes them:

```rust
#[cgp_new_provider]
impl<Context, Code, Input, ProviderA, ProviderB> ComputerRef<Context, Code, Input>
    for ConcatOutputs<ProviderA, ProviderB>
where
    ProviderA: ComputerRef<Context, Code, Input, Output: Display>,
    ProviderB: ComputerRef<Context, Code, Input, Output: Display>,
{
    type Output = String;

    fn compute_ref(context: &Context, code: PhantomData<Code>, input: &Input) -> String {
        let output_a = ProviderA::compute_ref(context, code, input);
        let output_b = ProviderB::compute_ref(context, code, input);
        format!("{} {}", output_a, output_b)
    }
}
```

`ConcatOutputs` acts as a **higher-order provider**: it composes two other providers and returns a new one that computes both and combines their results. This is conceptually similar to a higher-order function that takes two closures and returns a new closure, except that it operates entirely at the type level. It requires that both inner providers implement `ComputerRef` and produce outputs that implement `Display`, ensuring that both results can be formatted into a string.

The real power of this approach becomes evident when we compose the two providers with a simple **type alias**:

```rust
pub type FullNameToString = ConcatOutputs<FirstNameToString, LastNameToString>;
```

With this definition, `FullNameToString` behaves like a single provider that computes the full name by combining the first and last name. What’s remarkable is that we don’t need to explicitly restate the underlying constraints from `FirstNameToString` and `LastNameToString`. Those constraints are **inferred and propagated** automatically, and will only be enforced at the point where `FullNameToString` is actually used.

This programming model effectively introduces **lazy type-level computation**. The logic for computing outputs is driven by trait implementations at the type level, but the actual evaluation only occurs when the provider is invoked. This allows CGP to perform complex, constraint-aware compositions without requiring language-level support.

## Base Implementation

### `HasField` Trait

Now that we have a clearer understanding of how CGP builds on extensible data types, it is time to take a closer look at the key field-related traits that CGP introduces to support this functionality. We begin with the simplest and most foundational trait: `HasField`.

The `HasField` trait has been part of CGP since its early versions, but it is still worth revisiting here to bridge the gap toward the more advanced constructs that follow. Its definition is straightforward:

```rust
pub trait HasField<Tag> {
    type Value;

    fn get_field(&self, _tag: PhantomData<Tag>) -> &Self::Value;
}
```

This trait provides **read-only access** to individual fields of a struct, using `Tag` as the field identifier. The tag indicates which field we want to access, and can take one of two forms depending on the struct: `Symbol!("first_name")` for named fields, or `Index<0>` for tuple-style fields. The associated type `Value` represents the type of the field being accessed, and the `get_field` method returns a reference to that field.

The use of `PhantomData<Tag>` as an argument may look unusual at first, but it plays a critical role in allowing Rust to infer which field is being requested. When multiple `HasField` implementations are available for a type, this allows the compiler to resolve the correct one.

Consider the following struct:

```rust
#[derive(HasField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

When the `HasField` macro is derived, it automatically generates implementations for both fields as follows.

```rust
impl HasField<Symbol!("first_name")> for Person {
    type Value = String;

    fn get_field(&self, _tag: PhantomData<Symbol!("first_name")>) -> &Self::Value {
        &self.first_name
    }
}

impl HasField<Symbol!("last_name")> for Person {
    type Value = String;

    fn get_field(&self, _tag: PhantomData<Symbol!("last_name")>) -> &Self::Value {
        &self.last_name
    }
}
```

This allows generic code to access either field in a type-safe way without requiring access to the concrete types.

With `HasField`, we can write code that reads from a subset of fields on a struct, enabling a flexible and reusable programming model. However, if we want to *construct* such subsets — as is required for the extensible builder pattern — we will first need to introduce a few additional building blocks.

### Partial Records

One of the core limitations of Rust’s struct system is that when constructing a value, you must provide values for *all* of its fields. This rigidity makes it difficult to build structs in a piecemeal fashion. To overcome this, CGP introduces the idea of **partial records** — a way to represent a struct with some fields intentionally left out.

Let’s consider the same `Person` struct used previously:

```rust
#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

When we derive the `BuildField` trait for this struct, CGP automatically generates a corresponding partial version of the struct called `PartialPerson`:

```rust
pub struct PartialPerson<F0: MapType, F1: MapType> {
    pub first_name: F0::Map<String>,
    pub last_name: F1::Map<String>,
}
```

Here, `F0` and `F1` are type parameters that must implement the `MapType` trait. These type parameters control whether each field is present or not in a given instance of the partial struct.

The `MapType` trait is defined as follows:

```rust
pub trait MapType {
    type Map<T>;
}
```

This trait uses a *generic associated type* (GAT) called `Map` to determine how the original field types should be transformed. In particular, `Map<T>` will either be `T` itself (if the field is present) or a placeholder type `()` (if the field is missing).

To support this, CGP provides two implementations of the `MapType` trait. The first is `IsPresent`, which maps a type to itself to indicate that a field is included:

```rust
pub struct IsPresent;

impl MapType for IsPresent {
    type Map<T> = T;
}
```

The second is `IsNothing`, which maps every type to `()` to indicate that the field is absent:

```rust
pub struct IsNothing;

impl MapType for IsNothing {
    type Map<T> = ();
}
```

To see how this works in practice, suppose we want to construct a partial `Person` value that only includes the `first_name` field. We can instantiate the type as `PartialPerson<IsPresent, IsNothing>`, where `F0 = IsPresent` and `F1 = IsNothing`. The resulting type expands to:

```rust
pub struct PartialPerson {
    pub first_name: String,
    pub last_name: (),
}
```

This means we can create a partial instance like this:

```rust
let partial_person = PartialPerson::<IsPresent, IsNothing> {
    first_name: "John".to_owned(),
    last_name: (),
};
```

### `HasBuilder` Trait

Once we have defined a partial record struct, we can introduce the `HasBuilder` trait. This trait allows us to initialize a partial record where *all fields are absent* by default:

```rust
pub trait HasBuilder {
    type Builder;

    fn builder() -> Self::Builder;
}
```

The `HasBuilder` trait defines an associated type called `Builder`, which represents the initial form of the partial record. The key requirement is that this `Builder` must have all of its fields empty, since the `builder()` method constructs it without requiring any input.

For the `Person` struct, we can implement the `HasBuilder` trait as follows:

```rust
impl HasBuilder for Person {
    type Builder = PartialPerson<IsNothing, IsNothing>;

    fn builder() -> Self::Builder {
        PartialPerson {
            first_name: (),
            last_name: (),
        }
    }
}
```

In this implementation, the initial `Builder` is simply a `PartialPerson` type where both field parameters use the `IsNothing` type mapper. To construct the empty builder, we initialize each field with its mapped type, which for `IsNothing` is always the unit type `()`. This gives us a clean and predictable starting point for incrementally building up a complete `Person` instance.

### `BuildField` Trait

With the initial builder in place, the next step is to define the `BuildField` trait, which enables us to incrementally populate fields in the partial record. This trait is defined as follows:

```rust
pub trait BuildField<Tag> {
    type Value;
    type Output;

    fn build_field(self, _tag: PhantomData<Tag>, value: Self::Value) -> Self::Output;
}
```

The `BuildField` trait is parameterized by a `Tag` type, which identifies the field being constructed, just like in the `HasField` trait. It also includes an associated type `Value`, representing the type of the field being added.

The trait additionally includes an associated type `Output`, which represents the new type of the builder after the field has been inserted. This `Output` type is essential because each insertion updates the builder’s type parameters, effectively transforming it into a new type that reflects the presence of additional fields.

The `build_field` method takes ownership of the current builder and consumes the new field value, returning an updated builder that includes the newly added field.

To see how this works in practice, consider the implementation of `BuildField` for the `first_name` field of the `PartialPerson` struct:

```rust
impl<F1: MapType> BuildField<Symbol!("first_name")> for PartialPerson<IsNothing, F1> {
    type Value = String;
    type Output = PartialPerson<IsPresent, F1>;

    fn build_field(self, _tag: PhantomData<Symbol!("first_name")>, value: Self::Value) -> Self::Output {
        PartialPerson {
            first_name: value,
            last_name: self.last_name,
        }
    }
}
```

In this implementation, we define `BuildField<Symbol!("first_name")>` for a `PartialPerson` where the `first_name` field is absent (`IsNothing`) and the `last_name` field is parameterized generically as `F1`. This means the implementation will work regardless of whether `last_name` is present or not. We specify the `Value` as `String`, which matches the type of `first_name`, and set the `Output` type to a new `PartialPerson` where the first parameter has been updated to `IsPresent`. The second parameter remains unchanged to preserve the existing state of `last_name`.

The method body constructs a new `PartialPerson` where the `first_name` field is now set to the given value, while the `last_name` field is carried over from the original builder.

Similarly, we can define `BuildField` for the `last_name` field:

```rust
impl<F0: MapType> BuildField<Symbol!("last_name")> for PartialPerson<F0, IsNothing> {
    type Value = String;
    type Output = PartialPerson<F0, IsPresent>;

    fn build_field(self, _tag: PhantomData<Symbol!("last_name")>, value: Self::Value) -> Self::Output {
        PartialPerson {
            first_name: self.first_name,
            last_name: value,
        }
    }
}
```

In this case, the generic parameter `F0` tracks the state of `first_name`, while `IsNothing` ensures that `last_name` is not yet present. The logic follows the same structure as the earlier implementation, simply updating the appropriate field.

With these implementations in place, we can now use the `HasBuilder` and `BuildField` traits together to construct a `Person` incrementally:

```rust
Person::builder() // PartialPerson<IsNothing, IsNothing>
    .build_field(PhantomData::<Symbol!("first_name")>, "John".to_string()) // PartialPerson<IsPresent, IsNothing>
    .build_field(PhantomData::<Symbol!("last_name")>, "Smith".to_string()) // PartialPerson<IsPresent, IsPresent>
```

Notably, the **order** in which fields are built does not matter. The type transformations ensure correctness regardless of sequencing, so the builder also works if we construct the `last_name` field first:

```rust
Person::builder() // PartialPerson<IsNothing, IsNothing>
    .build_field(PhantomData::<Symbol!("last_name")>, "Smith".to_string()) // PartialPerson<IsNothing, IsPresent>
    .build_field(PhantomData::<Symbol!("first_name")>, "John".to_string()) // PartialPerson<IsPresent, IsPresent>
```

This gives developers the freedom to build up records in any order while maintaining type safety.

### `FinalizeBuild` Trait

The previous example demonstrated how the `builder` and `build_field` methods can be used to construct values in the style of a fluent builder pattern. However, it is important to note that the result of the final `build_field` call is still a `PartialPerson<IsPresent, IsPresent>`, not a fully constructed `Person`.

This limitation arises because each `BuildField` implementation for `PartialPerson` is only responsible for inserting a single field. The presence or absence of other fields is abstracted away through generic type parameters, which means that the implementation cannot detect when the final field has been added. Consequently, there is no opportunity to directly return a `Person` value when the last required field is set.

While this might make the final construction step slightly more verbose, it is a deliberate trade-off. The generic nature of `BuildField` is what allows fields to be built in any order without having to implement every possible combination of partially constructed records — something that would otherwise result in an overwhelming combinatorial explosion of implementations.

To resolve this, CGP introduces the `FinalizeBuild` trait. This trait is used once the builder has been fully populated, converting the complete `PartialPerson` into a proper `Person` value:

```rust
pub trait FinalizeBuild {
    type Output;

    fn finalize_build(self) -> Self::Output;
}
```

The `FinalizeBuild` trait defines an associated `Output` type, representing the final constructed result. The `finalize_build` method takes ownership of the builder and transforms it into the desired output.

For `PartialPerson`, the trait is implemented as follows:

```rust
impl FinalizeBuild for PartialPerson<IsPresent, IsPresent> {
    type Output = Person;

    fn finalize_build(self) -> Self::Output {
        Person {
            first_name: self.first_name,
            last_name: self.last_name,
        }
    }
}
```

This implementation only applies when all fields are marked as present. At this point, the builder contains all the data necessary to construct a `Person`, so the conversion is a straightforward transfer of fields.

With this in place, the build process becomes complete by appending a call to `finalize_build`:

```rust
let person = Person::builder() // PartialPerson<IsNothing, IsNothing>
    .build_field(PhantomData::<Symbol!("first_name")>, "John".to_string()) // PartialPerson<IsPresent, IsNothing>
    .build_field(PhantomData::<Symbol!("last_name")>, "Smith".to_string()) // PartialPerson<IsPresent, IsPresent>
    .finalize_build(); // Person
```

Together, the partial record struct and the traits `HasBuilder`, `BuildField`, and `FinalizeBuild` form a solid and ergonomic foundation for supporting extensible records in CGP. All of these pieces are automatically generated by the `#[derive(BuildField)]` macro. We do not use multiple derive macros here, so as to ensure consistency and correctness, eliminating the possibility of compilation failures due to the user missing to derive one of these traits.

## Implementation of Record Merges

With the field builder traits in place, we can now explore how the earlier [struct building](/blog/extensible-datatypes-part-1#safe-struct-building) method `build_from` can be implemented to support merging a `Person` struct into a superset struct such as `Employee`.

Before we can implement `build_from`, we first need a few more supporting constructs to enable the merging operation.

### `IntoBuilder` Trait

In typical usage, partial records begin in an empty state and are gradually populated with field values until they can be finalized into a complete struct. However, we can also go in the **opposite** direction by converting a fully constructed struct *into* a partial record where all fields are present, and then progressively *remove* fields from it, one by one, until none remain.

This reverse direction is particularly important for the merging process, where we need to *move* fields out of a source struct and insert them into a target partial record, field by field.

To support this, we introduce the `IntoBuilder` trait:

```rust
pub trait IntoBuilder {
    type Builder;

    fn into_builder(self) -> Self::Builder;
}
```

This trait mirrors the structure of `HasBuilder`, with an associated `Builder` type that represents a partial record in which all fields are populated. The key difference is that `into_builder` consumes the original struct and produces its fully populated partial record equivalent.

The implementation of `IntoBuilder` for `Person` looks like this:

```rust
impl IntoBuilder for Person {
    type Builder = PartialPerson<IsPresent, IsPresent>;

    fn into_builder(self) -> Self::Builder {
        PartialPerson {
            first_name: self.first_name,
            last_name: self.last_name,
        }
    }
}
```

If you compare this to the implementation of the earlier `FinalizeBuild` trait, you’ll see that they are nearly identical in structure. The only difference is the direction of conversion — `IntoBuilder` transforms a `Person` into a `PartialPerson<IsPresent, IsPresent>`, while `FinalizeBuild` does the reverse.

Even though these interfaces are similar, we define `IntoBuilder` and `FinalizeBuild` as separate traits. This distinction is valuable because it makes the intent of each trait clear and prevents accidental misuse. Each trait captures a specific stage in the lifecycle of a partial record, whether we are constructing it from scratch, building it up field by field, or tearing it down for merging.

### `TakeField` Trait

Now that we have the `IntoBuilder` trait to help convert a struct into a fully populated partial record, we also need a way to extract individual fields from that partial record. This is where the `TakeField` trait comes in. It serves as the *opposite* of `BuildField`, allowing us to take a field value *out* of a partial record:

```rust
pub trait TakeField<Tag> {
    type Value;
    type Remainder;

    fn take_field(self, _tag: PhantomData<Tag>) -> (Self::Value, Self::Remainder);
}
```

The `Tag` and `Value` types in `TakeField` behave just like they do in `BuildField` and `HasField`. What is new here is the associated type `Remainder`, which represents the state of the partial record after the specified field has been removed. The `take_field` method consumes `self` and returns a tuple containing the extracted field value along with the updated remainder of the partial record.

As an example, here is the `TakeField` implementation for extracting the `first_name` field from a `PartialPerson`:

```rust
impl<F1> TakeField<Symbol!("first_name")> for PartialPerson<IsPresent, F1> {
    type Value = String;
    type Remainder = PartialPerson<IsNothing, F1>;

    fn take_field(self, _tag: PhantomData<Symbol!("first_name")>) -> (Self::Value, Self::Remainder) {
        let value = self.first_name;
        let remainder = PartialPerson {
            first_name: (),
            last_name: self.last_name,
        };

        (value, remainder)
    }
}
```

As shown, this implementation is defined for a `PartialPerson` where the first generic parameter is `IsPresent`, indicating that the `first_name` field is available to be taken. In the resulting `Remainder`, that parameter is updated to `IsNothing`, reflecting the removal of the field. The method itself returns the `first_name` value and a new partial record with `first_name` cleared and the rest of the fields left untouched.

This setup provides the building blocks needed to flexibly extract and transfer fields, which is essential for safely merging one struct into another.

### `HasFields` Trait

With `IntoBuilder` available, we can now begin transferring fields from a source struct into a target partial record by peeling them off one at a time. To enable this process generically, we need a mechanism to *enumerate* the fields of a struct so that generic code can discover which fields are available for transfer.

This is where the `HasFields` trait comes into play. It is defined as follows:

```rust
pub trait HasFields {
    type Fields;
}
```

The `HasFields` trait includes a single associated type called `Fields`, which holds a *type-level list* representing all the fields of the struct. For example, here is how `HasFields` would be implemented for the `Person` struct:

```rust
impl HasFields for Person {
    type Fields = Product![
        Field<Symbol!("first_name"), String>,
        Field<Symbol!("last_name"), String>,
    ];
}
```

Once the fields of a struct are made available as a type-level list, this list can be used to drive *type-level iteration* for field-wise operations such as merging. This lays the foundation for generically moving fields between records in a structured and type-safe way.

### `BuildFrom` Trait

The `build_from` method is defined within the `CanBuildFrom` trait, which uses a blanket implementation to support merging fields from one struct into another:

```rust
pub trait CanBuildFrom<Source> {
    type Output;

    fn build_from(self, source: Source) -> Self::Output;
}

impl<Builder, Source, Output> CanBuildFrom<Source> for Builder
where
    Source: HasFields + IntoBuilder,
    Source::Fields: FieldsBuilder<Source::Builder, Builder, Output = Output>,
{
    type Output = Output;

    fn build_from(self, source: Source) -> Output {
        Source::Fields::build_fields(source.into_builder(), self)
    }
}
```

In this setup, the `CanBuildFrom` trait is implemented for the partial record acting as the *target* of the merge, while the `Source` type is specified through a generic parameter. The trait defines an associated type `Output`, representing the result of merging the fields from `Source` into the current builder. The `build_from` method takes ownership of both the builder (`self`) and the source value, and returns a new builder that incorporates the fields from `Source`.

The blanket implementation of `CanBuildFrom` imposes several trait bounds. First, the `Source` type must implement both `HasFields` and `IntoBuilder`. These traits provide access to a type-level list of fields and the ability to convert `Source` into a partial record with all fields present. The field list obtained from `Source::Fields` must also implement the `FieldsBuilder` helper trait. This trait is responsible for driving the merge process field by field, taking as input the `Source::Builder` and the target `Builder` (i.e., `Self`), and producing an `Output`.

The implementation of `build_from` begins by converting the source into its partial form using `into_builder`, then delegates the merge logic to `FieldsBuilder::build_fields`, which handles transferring each field in sequence.

### `FieldsBuilder` Trait

The `FieldsBuilder` trait is a *private* helper used exclusively by the `CanBuildFrom` trait. It is defined as follows:

```rust
trait FieldsBuilder<Source, Target> {
    type Output;

    fn build_fields(source: Source, target: Target) -> Self::Output;
}
```

Unlike `CanBuildFrom`, the parameters for `FieldsBuilder` are slightly reordered. The `Self` type represents the list of fields from the source struct, while `Source` and `Target` refer to the partial records we are merging from and into. The goal of this trait is to drive type-level iteration across the list of fields, one by one, and move each field from the source to the target.

To accomplish this, we start with an implementation that matches on the head of the `Fields` list:

```rust
impl<Source, Target, RestFields, Tag, Value> FieldsBuilder<Source, Target>
    for Cons<Field<Tag, Value>, RestFields>
where
    Source: TakeField<Tag, Value = Value>,
    Target: BuildField<Tag, Value = Value>,
    RestFields: FieldsBuilder<Source::Remainder, Target::Output>,
{
    type Output = RestFields::Output;

    fn build_fields(source: Source, target: Target) -> Self::Output {
        let (value, next_source) = source.take_field(PhantomData);
        let next_target = target.build_field(PhantomData, value);

        RestFields::build_fields(next_source, next_target)
    }
}
```

Here, we pattern match the `Fields` type to `Cons<Field<Tag, Value>, RestFields>`, allowing us to extract the field name `Tag` and its type `Value`. Given this information, we require that the `Source` partial record implements `TakeField` and that the `Target` partial record implements `BuildField`, both using the same `Tag` and `Value`.

We then handle the remaining fields recursively by requiring `RestFields` to implement `FieldsBuilder`, using the `Remainder` type from `TakeField` as the next `Source`, and the `Output` type from `BuildField` as the next `Target`. This enables a seamless hand-off from one field to the next during the merging process.

The recursive chain is terminated by the base case, when there are no more fields left to process:

```rust
impl<Source, Target> FieldsBuilder<Source, Target> for Nil {
    type Output = Target;

    fn build_fields(_source: Source, target: Target) -> Self::Output {
        target
    }
}
```

In this final implementation, the `Source` partial record is now fully depleted — all fields have been taken out — so we simply return the `Target`, which now contains all the merged fields.

### Example Use of `BuildFrom`

The implementation of `FieldsBuilder` can appear intimidating at first, particularly for readers unfamiliar with type-level programming. To make the process more approachable, let’s walk through a concrete example using `BuildFrom` to illustrate what actually happens under the hood.

Consider a new struct named `Employee`, which contains the same fields as `Person`, along with an additional field called `employee_id`:

```rust
#[derive(BuildField)]
pub struct Employee {
    pub employee_id: u64,
    pub first_name: String,
    pub last_name: String,
}
```

We begin by constructing a `Person` value. After that, we can use `build_from` to merge its contents into a partially built `Employee`:

```rust
let person = Person {
    first_name: "John".to_owned(),
    last_name: "Smith".to_owned(),
};

let employee = Employee::builder() // PartialEmployee<IsNothing, IsNothing, IsNothing>
    .build_from(person) // PartialEmployee<IsNothing, IsPresent, IsPresent>
    .build_field(PhantomData::<Symbol!("employee_id")>, 1) // PartialEmployee<IsPresent, IsPresent, IsPresent>
    .finalize_build(); // Person
```

When we call `build_from`, several steps take place behind the scenes:

* The type `PartialEmployee<IsNothing, IsNothing, IsNothing>` is required to implement `CanBuildFrom<Person>`.
  * The `Person` type implements `HasFields::Fields` as:
    ```rust
    Product![
        Field<Symbol!("first_name"), String>,
        Field<Symbol!("last_name"), String>,
    ]
    ```
  * `Person` also implements `IntoBuilder`, producing `PartialPerson<IsPresent, IsPresent>` as its `Builder`.
* `Person::Fields` must then implement `FieldsBuilder<PartialPerson<IsPresent, IsPresent>, PartialEmployee<IsNothing, IsNothing, IsNothing>>`.
  * The first `Cons` in the list matches:
    * `Tag` is `Symbol!("first_name")`
    * `Value` is `String`
    * `RestFields` is `Cons<Field<Symbol!("last_name"), String>, Nil>`
  * `PartialPerson<IsPresent, IsPresent>` implements `TakeField<Symbol!("first_name"), Value = String>`, resulting in:
    * A `Remainder` of `PartialPerson<IsNothing, IsPresent>`
  * `PartialEmployee<IsNothing, IsNothing, IsNothing>` implements `BuildField<Symbol!("first_name"), Value = String>`, producing:
    * An `Output` of `PartialEmployee<IsNothing, IsPresent, IsNothing>`
* The remaining fields, `Cons<Field<Symbol!("last_name"), String>, Nil>`, must now implement `FieldsBuilder<PartialPerson<IsNothing, IsPresent>, PartialEmployee<IsNothing, IsPresent, IsNothing>>`.
  * This matches:
    * `Tag` is `Symbol!("last_name")`
    * `Value` is `String`
    * `RestFields` is `Nil`
  * `PartialPerson<IsNothing, IsPresent>` implements `TakeField<Symbol!("last_name"), Value = String>`, giving:
    * A `Remainder` of `PartialPerson<IsNothing, IsNothing>`
  * `PartialEmployee<IsNothing, IsPresent, IsNothing>` implements `BuildField<Symbol!("last_name"), Value = String>`, yielding:
    * An `Output` of `PartialEmployee<IsNothing, IsPresent, IsPresent>`
* Finally, the `Nil` case implements `FieldsBuilder<PartialPerson<IsNothing, IsNothing>, PartialEmployee<IsNothing, IsPresent, IsPresent>>`, which concludes by returning:
  * `PartialEmployee<IsNothing, IsPresent, IsPresent>` as the final `Output`.

Although these steps may seem complex at first glance, a closer look reveals that the process simply moves each field from the `PartialPerson` instance into the `PartialEmployee`, one at a time. What makes this look more complicated is not the logic itself, but the fact that it is encoded entirely at the type level using traits and generics, rather than as regular Rust code.

If any part of this explanation remains unclear, it might be helpful to paste this blog post — or just this sub-section — into your favorite LLM and ask it to explain the process in simpler terms. Hopefully, the explanation provided here is already clear enough for an LLM to understand, so that it can in turn help make this pattern more accessible to developers who are still learning the intricacies of type-level programming.

## Builder Dispatcher

With the `BuildFrom` trait in place, we can now explore how CGP implements generalized **builder dispatchers** that enable flexible and reusable ways to assemble struct fields from various sources.

### `BuildWithHandlers` Provider

In [earlier examples](/blog/extensible-datatypes-part-1/#builder-dispatcher), we used a utility called `BuildAndMergeOutputs` to combine outputs from multiple builder providers such as `BuildSqliteClient`, `BuildHttpClient`, and `BuildOpenAiClient`. Under the hood, `BuildAndMergeOutputs` is built upon a more fundamental dispatcher named `BuildWithHandlers`. The implementation of this dispatcher looks like the following:

```rust
#[cgp_provider]
impl<Context, Code, Input, Output, Builder, Handlers, Res> Computer<Context, Code, Input>
    for BuildWithHandlers<Output, Handlers>
where
    Output: HasBuilder<Builder = Builder>,
    PipeHandlers<Handlers>: Computer<Context, Code, Builder, Output = Res>,
    Res: FinalizeBuild<Output = Output>,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, _input: Input) -> Self::Output {
        PipeHandlers::compute(context, code, Output::builder()).finalize_build()
    }
}
```

The `BuildWithHandlers` struct is parameterized by an `Output` type, which is the final struct we want to construct, and a `Handlers` type, which represents a type-level list of builder handlers. These handlers will be used to incrementally populate the fields of the `Output` struct.

This provider implements the `Computer` trait for any combination of `Context`, `Code`, and `Input`, although the `Input` parameter is intentionally ignored. To begin, the dispatcher requires the `Output` type to implement `HasBuilder`, which gives access to an initially empty partial record through `Output::builder()`.

Once the empty builder is obtained, it is passed as input to `PipeHandlers<Handlers>`, a pipeline that applies each handler in the list to progressively build up the partial record. The result of this pipeline must implement `FinalizeBuild`, allowing it to be converted into the fully constructed `Output` struct.

And yes, if you're wondering whether this `PipeHandlers` is the same one used in [Hypershell](/blog/hypershell-release) to build shell-like command pipelines — the answer is absolutely yes. `BuildWithHandlers` operates by initializing an empty partial record, passing it through a chain of handlers using `PipeHandlers`, and then finalizing the result into a complete struct. It’s the same elegant piping mechanism, just applied to a different domain.

This reuse of core abstractions like `Pipe` and `Handler` across different systems is one of the most powerful aspects of CGP. These components weren’t designed just for piping shell commands — they were built to support general-purpose [function composition](https://en.wikipedia.org/wiki/Function_composition_%28computer_science%29), a core concept in functional programming.

### Example Use of `BuildWithHandlers`

The `BuildWithHandlers` trait serves as the foundational builder dispatcher in CGP. It is the low-level mechanism behind higher-level dispatchers like `BuildAndMergeOutputs`, and understanding it provides valuable insight into how CGP composes complex data construction pipelines.

Let’s walk through a concrete example to illustrate how `BuildWithHandlers` works in practice.

Suppose we have two `Computer` providers: one that constructs a `Person` and another that produces a `u64` value for the `employee_id` field. We can use `BuildWithHandlers` to compose these providers and construct an `Employee`.

We begin by implementing the two providers as simple functions using the `#[cgp_producer]` macro:

```rust
#[cgp_producer]
pub fn build_person() -> Person {
    Person {
        first_name: "John".to_owned(),
        last_name: "Smith".to_owned(),
    }
}

#[cgp_producer]
pub fn build_employee_id() -> u64 {
    1
}
```

The `#[cgp_producer]` macro generates provider implementations (named `BuildPerson` and `BuildEmployeeId`) that wrap these functions. These implementations conform to the `Computer` trait by automatically calling the associated function. This macro allows pure functions to be used as providers with minimal boilerplate, especially useful when you don't need to depend on the generic `Context` or `Code` parameters.

Now, with both providers defined, we can compose them using `BuildWithHandlers` like so:

```rust
let employee = BuildWithHandlers::<
    Employee,
    Product![
        BuildAndMerge<BuildPerson>,
        BuildAndSetField<Symbol!("employee_id"), BuildEmployeeId>
    ],
>::compute(&(), PhantomData::<()>, ());
```

In this example, we tell `BuildWithHandlers` that the final struct we want to build is `Employee`. We then provide a list of two builder handlers:

1. `BuildAndMerge<BuildPerson>` wraps the `BuildPerson` provider. It first calls the provider to build a `Person`, then uses `BuildFrom` to merge the resulting fields into the `PartialEmployee` builder.
2. `BuildAndSetField<Symbol!("employee_id"), BuildEmployeeId>` wraps the `BuildEmployeeId` provider. It calls the provider to get the `u64` value and then uses `BuildField` to assign that value to the `employee_id` field.

We invoke `compute` on the specialized `BuildWithHandlers`, using unit types `()` as dummy arguments for `Context`, `Code`, and `Input`. In real-world applications, these types would typically carry contextual information such as configurations or runtime.

This example demonstrates the flexibility of `BuildWithHandlers`. You’re not limited to merging entire record subsets — you can also work with providers that produce individual field values, then insert them into specific fields using composable builder logic.

### `BuildAndMerge`

The `BuildAndMerge` adapter is relatively simple and is defined as follows:

```rust
#[cgp_provider]
impl<Context, Code, Builder, Provider, Output, Res> Computer<Context, Code, Builder>
    for BuildAndMerge<Provider>
where
    Provider: for<'a> Computer<Context, Code, &'a Builder, Output = Res>,
    Builder: CanBuildFrom<Res, Output = Output>,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, builder: Builder) -> Self::Output {
        let output = Provider::compute(context, code, &builder);
        builder.build_from(output)
    }
}
```

`BuildAndMerge` wraps an inner provider (such as `BuildPerson`) and expects the current `Builder` — a partial record for the target struct — as input. The inner provider is invoked with a **reference** to this builder as its inner input, allowing it to inspect any fields that may already be set. This feature enables more advanced scenarios where intermediate results influence later ones.

After the inner provider returns a value, `BuildAndMerge` uses the `BuildFrom` trait to merge the result into the existing builder. The merged builder is then returned as output.

In simpler cases like `BuildPerson`, the input builder is ignored, so the trait bounds are trivially satisfied. But the mechanism still works the same way: take a value and merge its fields into the partial record.

### `BuildAndSetField`

The `BuildAndSetField` adapter works similarly but focuses on setting a single field rather than merging a full record:

```rust
#[cgp_provider]
impl<Context, Code, Tag, Value, Provider, Output, Builder> Computer<Context, Code, Builder>
    for BuildAndSetField<Tag, Provider>
where
    Provider: for<'a> Computer<Context, Code, &'a Builder, Output = Value>,
    Builder: BuildField<Tag, Value = Value, Output = Output>,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, builder: Builder) -> Self::Output {
        let value = Provider::compute(context, code, &builder);
        builder.build_field(PhantomData::<Tag>, value)
    }
}
```

Instead of calling `BuildFrom`, this adapter uses the `BuildField` trait to assign a value to a specific field. The field to be set is identified by `Tag`, and its value comes from the wrapped provider.

This adapter is ideal for cases where you have an individual value (such as `u64`) and want to insert it into a specific field (like `employee_id`).

### `MapFields` Trait

In the previous example, we saw how to use `BuildWithHandlers` with a list of individual providers — each wrapped in `BuildAndMerge` or `BuildAndSetField` — to build a composite struct. This same pattern is what powers higher-level dispatchers like `BuildAndMergeOutputs`.

Since the pattern of wrapping a list of handlers is so common, CGP provides the `MapFields` trait to simplify the process. Here’s how it’s defined:

```rust
pub trait MapFields<Mapper> {
    type Map;
}

impl<Mapper, Current, Rest> MapFields<Mapper> for Cons<Current, Rest>
where
    Mapper: MapType,
    Rest: MapFields<Mapper>,
{
    type Map = Cons<Mapper::Map<Current>, Rest::Map>;
}

impl<Mapper> MapFields<Mapper> for Nil {
    type Map = Nil;
}
```

The `MapFields` trait enables type-level mapping over a type-level list — very similar to how `.iter().map()` works on value-level lists in Rust. You pass in a `Mapper` that implements the `MapType` trait, and it is applied to each element in the list. This is the same `MapType` trait we’ve seen used earlier in other utilities, like `IsPresent` and `IsNothing`.

### `BuildAndMergeOutputs`

Now that we have the pieces in place, we can implement `BuildAndMergeOutputs` as a composition of `BuildWithHandlers` and `BuildAndMerge`.

We start by defining a simple type mapper called `ToBuildAndMergeHandler`, which takes any `Handler` type and maps it to a `BuildAndMerge<Handler>`:

```rust
pub struct ToBuildAndMergeHandler;

impl MapType for ToBuildAndMergeHandler {
    type Map<Handler> = BuildAndMerge<Handler>;
}
```

With that, we can define `BuildAndMergeOutputs` simply as a *type alias*. It takes an `Output` type and a list of handler types, and internally applies `ToBuildAndMergeHandler` to each handler in the list. The result is passed to `BuildWithHandlers`:

```rust
pub type BuildAndMergeOutputs<Output, Handlers> =
    BuildWithHandlers<Output, <Handlers as MapFields<ToBuildAndMergeHandler>>::Map>;
```

This implementation showcases the power of type-level composition in CGP. By combining smaller, reusable components like `BuildWithHandlers`, `BuildAndMerge`, and `MapFields`, we can construct higher-level abstractions like `BuildAndMergeOutputs` with minimal boilerplate and high flexibility.

This modular, composable design stands in contrast to traditional macro-based approaches. With macros, it’s much harder to verify that combining two smaller macros results in correct behavior — especially as the complexity grows. In contrast, with CGP's type-level constructs, each piece remains type-checked and composable under well-defined `where` constraints. These constraints act as a safeguard, ensuring that the combined logic remains sound and predictable.

Moreover, this approach enables us to easily define other dispatcher variants, such as those that use `BuildAndSetField` instead of `BuildAndMerge`, without having to rewrite or duplicate core logic.

### Hiding Constraints with `delegate_components!`

While it's possible to define `BuildAndMergeOutputs` as a simple type alias, doing so introduces ergonomic issues when it’s used with a *generic* `Handlers` type provided by the caller.

Consider a situation where we want to implement a provider that wraps around `BuildAndMergeOutputs` in order to perform validation on the result before returning it. We might write something like this:

```rust
#[cgp_new_provider]
impl<Context, Code, Input, Output, Handlers> Computer<Context, Code, Input>
    for BuildAndValidateOutput<Output, Handlers>
where
    BuildAndMergeOutputs<Output, Handlers>: Computer<Context, Code, Input>,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: Input) -> Output {
        ...
    }
}
```

Here, `BuildAndValidateOutput` *statically* depends on `BuildAndMergeOutputs` by requiring that it implements `Computer`. However, this code won’t compile as-is. The problem is that `BuildAndMergeOutputs` is defined as a type alias that internally relies on a hidden constraint: `Handlers: MapFields<ToBuildAndMergeHandler>`. Since this constraint is not visible at the site of usage, the compiler now demands that we explicitly provide proof that `Handlers` satisfies this requirement.

While we could add the missing constraint to the `where` clause of `BuildAndValidateOutput`, this reduces the ergonomics and composability of the abstraction. It requires callers to understand the internal structure of `BuildAndMergeOutputs`, which goes against one of the core strengths of CGP — *hiding internal constraints* so they don't leak into user code.

Fortunately, we can solve this by turning `BuildAndMergeOutputs` into a regular provider struct, and using the `delegate_components!` macro to **delegate** the `Computer` implementation to `BuildWithHandlers`, while keeping the necessary constraints encapsulated.

Here’s how:

```rust
delegate_components! {
    <Output, Handlers: MapFields<ToBuildAndMergeHandler>>
    new BuildAndMergeOutputs<Output, Handlers> {
        ComputerComponent:
            BuildWithHandlers<Output, Handlers::Map>
    }
}
```

This macro creates a new `BuildAndMergeOutputs` provider that wraps `BuildWithHandlers`, while adding the required constraint on `Handlers` behind the scenes. Internally, the macro expands into a `DelegateComponent` implementation like the following:

```rust
impl<Output, Handlers> DelegateComponent<ComputerComponent>
    for BuildAndMergeOutputs<Output, Handlers>
where
    Handlers: MapFields<ToBuildAndMergeHandler>,
{
    type Delegate = BuildWithHandlers<Output, Handlers::Map>;
}
```

With this setup, `BuildAndMergeOutputs` can now be used like any other CGP provider — without needing to manually restate its internal type constraints. This keeps client code clean and focused, and allows CGP abstractions to remain composable and extensible.

The key benefit of this pattern is that it avoids boilerplate while preserving type safety. Whenever a provider's implementation is simply a thin wrapper around another provider with some added constraints, it's much more convenient to use `DelegateComponent` via `delegate_components!` than to implement the provider trait manually.

### Type-Level Metaprogramming

The technique we just explored — wrapping providers and using `delegate_components!` — can be seen as a form of **metaprogramming** in CGP. Here, we’re leveraging **type-level programming** not just within CGP’s core component abstractions like `DelegateComponent`, but also as a tool for *programmatically defining component wiring* through the use of generic parameters and trait constraints.

This highlights a deeper design philosophy behind CGP: rather than inventing a new meta-language or macro system, CGP embraces Rust’s existing type system and trait machinery as the foundation for composability and abstraction. Type-level programming in CGP isn’t an escape hatch — it is the underlying mechanism that *makes escape hatches unnecessary*.

Some readers may find the phrase "type-level programming" intimidating, especially if they associate it with obscure or overly abstract code. But consider the alternatives: in many dynamic languages like Ruby, Python, or JavaScript, metaprogramming is accomplished through bespoke syntax, reflection, or runtime patching — approaches that can be powerful but often come with poor tooling, fragile semantics, and hard-to-debug behavior.

In contrast, type-level programming offers a principled and well-typed foundation for metaprogramming. Constraints are checked at compile time, tooling support remains robust, and the abstractions remain composable. Instead of inventing ad hoc metaprogramming constructs, CGP relies on the well-established theory and practice of type-level computation.

By doing so, CGP achieves a rare combination of power and predictability. Developers who embrace this pattern gain access to highly expressive abstractions, while staying within the familiar boundaries of Rust's type system.

## Conclusion

By this point, I hope you have a clearer understanding of how CGP supports extensible records. If you are interested in exploring the implementation further, take a look at the [GitHub repository](https://github.com/contextgeneric/cgp), especially the `cgp-field` and `cgp-dispatch` crates, which contain the full source code.

With partial records and field traits such as `HasField`, `BuildField`, and `TakeField`, CGP enables powerful generic operations like `BuildFrom`, which allows one struct to be merged into another seamlessly. These building blocks form the foundation for more advanced compositional patterns.

To implement the extensible builder pattern, we introduced the `BuildWithHandlers` dispatcher. This component composes multiple builder handlers and merges their outputs using a clean and predictable build pipeline, constructed with `PipeHandlers`. The simplicity arises from a system where modularity is built into the core design, rather than added as an afterthought.

On top of this, we implemented the high-level `BuildAndMergeOutputs` dispatcher by defining a conditional delegation to `BuildWithHandlers`, after first wrapping each handler using the `BuildAndMerge` adapter. This design preserves composability while allowing for customized construction logic.

### Future Extensions

Because extensible records and builders are implemented modularly, it is easy to extend the system further without rewriting the core. For instance, the current builder pattern requires the source struct to match the target struct exactly, but we may want to allow certain fields to be **dropped** rather than merged.

We might also want to support **overriding** existing fields in a partial record when the same field appears in multiple sources. Additionally, it would be useful to finalize a partial record by filling in any missing fields with **default values**.

These enhancements are already within reach, and we plan to support them in future versions of CGP. Even if they are not included directly in the library, the existing abstractions make it easy for others to implement them independently.

Thanks to the expressive power of Rust’s type system and the composability of type-level programming, these extensions can be implemented in a way that is both straightforward and correct by construction. They remain completely optional and do not introduce additional complexity to the core logic presented here.

This level of flexibility would be difficult to achieve with more ad hoc approaches, such as macros or code generation, which often require the entire logic and all possible extensions to be baked into a single monolithic system. That leads to unnecessary coupling and limits customizability.

If you're still unsure how all of this comes together, a future blog post will walk through the implementation of these extensions in detail to show exactly how they work.

### Next Part

In the final [Part 4 of this series, **Implementing Extensible Variants**](/blog/extensible-datatypes-part-4), we will follow a similar path to explore how CGP implements extensible variants. Keep in mind the concepts we covered for extensible records — you may be surprised to discover just how much of the same logic carries over, despite the differences between records and variants.
