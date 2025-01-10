+++

title = "Context-Generic Programming"

insert_anchor_links = "heading"

+++

# Announcement

We’re excited to announce the release of [v0.3.0](https://github.com/contextgeneric/cgp/releases/tag/v0.3.0) of the [`cgp`](https://docs.rs/cgp/0.3.0/cgp/) crate, along with several [new](https://patterns.contextgeneric.dev/error-handling.html) [chapters](https://patterns.contextgeneric.dev/field-accessors.html) of the [CGP Patterns](https://patterns.contextgeneric.dev/) book!

Read the blog post for more details: [CGP Updates: v0.3.0 Release and New Chapters](/blog/v0-3-0-release/).

# Introduction

Context-generic programming (CGP) is a new programming paradigm for Rust that allows strongly-typed components to be implemented and composed in a modular, generic, and type-safe way.

On this homepage, we provide a quick overview and highlight the key features of CGP. For a deeper dive into the concepts and patterns of CGP, explore our comprehensive book, [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).

# Current Status

As of the start of 2025, CGP remains in its _early stages_ of development. While promising, it still has several rough edges, particularly in areas such as documentation, tooling, debugging techniques, community support, and ecosystem maturity.

As such, adopting CGP for serious projects comes with inherent challenges, and users are advised to proceed _at their own risk_. The primary risk is not technical but stems from the limited support available when encountering difficulties in learning or applying CGP.

At this stage, CGP is best suited for early adopters and potential [contributors](#contribution) who are willing to experiment and help shape its future.

# Hello World Example

We will demonstrate various concepts of CGP with a simple hello world example.

## Greeter Component

To begin, we import the `cgp` crate and define a greeter component as follows:

```rust
use cgp::prelude::*;

#[cgp_component {
    name: GreeterComponent,
    provider: Greeter,
}]
pub trait CanGreet {
    fn greet(&self);
}
```

The `cgp` crate provides common constructs through its `prelude` module, which should be imported in most cases. The first CGP construct we use here is the `#[cgp_component]` macro. This macro generates additional CGP constructs for the greeter component.

The target of this macro, `CanGreet`, is a _consumer trait_ used similarly to regular Rust traits. However, unlike traditional traits, we won't implement anything directly on this trait.

The `name` argument, `GreeterComponent`, specifies the name of the greeter component. The `provider` argument, `Greeter`, designates a _provider trait_ for the component. The `Greeter` provider is used to define the actual implementations for the greeter component. It has a similar structure to `CanGreet`, but with the implicit `Self` type replaced by a generic `Context` type.

## A Name Dependency

Next, let's introduce dependencies needed to implement an example provider for `Greeter`. We'll start by declaring an abstract `Name` type:

```rust
cgp_type!( Name )
```

Here, the `cgp_type!` macro defines a CGP component for the abstract type `Name`. This macro is a concise alternative to using `#[cgp_component]`. It also derives additional implementations useful later. For now, it is enough to know that `cgp_type!` generates a `HasNameType` consumer trait, which includes an _associated type_ called `Name`.

Now, we'll define an _accessor trait_ to retrieve the name value from a context:

```rust
#[cgp_auto_getter]
pub trait HasName: HasNameType {
    fn name(&self) -> &Self::Name;
}
```

The `HasName` trait inherits from `HasNameType` to access the abstract type `Self::Name`. It includes the method `name`, which returns a reference to a value of type `Self::Name`.

The `#[cgp_auto_getter]` attribute macro applied to `HasName` automatically generates a blanket implementation. This enables any context containing a field named `name` of type `Self::Name` to automatically implement the `HasName` trait.

## Hello Greeter

The traits `CanGreet`, `HasNameType`, and `HasName` can be implemented independently across different modules or crates. However, we can import them into a single location and then implement a `Greeter` provider that uses `HasName` in its implementation:

```rust
pub struct GreetHello;

impl<Context> Greeter<Context> for GreetHello
where
    Context: HasName,
    Context::Name: Display,
{
    fn greet(context: &Context) {
        println!("Hello, {}!", context.name());
    }
}
```

Here, we define `GreetHello` as a struct, which will be used to implement the `Greeter` provider trait. Unlike the consumer trait, where `Self` refers to the implementing type, the `Greeter` provider trait uses an explicit generic type `Context`, which fulfills the role of `Self` from `CanGreet`. The `GreetHello` type will serve as the `Self` type for `Greeter`, but we don't reference `self` in the provider trait implementation.

The implementation comes with two additional constraints:

- `Context: HasName` ensures the context implements `HasName`.
- `Context::Name: Display` allows the provider to work with any abstract `Name` type, as long as it implements `Display`.

Notice that these constraints are specified only in the `impl` block, not in the trait bounds for `CanGreet` or `Greeter`. This design allows us to use _dependency injection_ for both values and _types_ through Rust’s trait system.

In the `greet` method, we use `context: &Context` as a parameter instead of the `&self` argument used in `CanGreet::greet`. We then call `context.name()` to retrieve the name value from the context and print a greeting with `println!`.

The `GreetHello` provider implementation is _generic_ over any `Context` and `Context::Name` types, as long as they satisfy the corresponding constraints for `HasName` and `Display`. By separating the provider trait from the consumer trait, we can have multiple provider implementations like `GreetHello` that are all generic over any `Context`, without causing the overlapping implementation issues typically imposed by Rust's trait system.

## Person Context

Next, we define a concrete context, `Person`, and wire it up to use `GreetHello` for implementing CanGreet:

```rust
#[derive(HasField)]
pub struct Person {
    pub name: String,
}

pub struct PersonComponents;

impl HasComponents for Person {
    type Components = PersonComponents;
}

delegate_components! {
    PersonComponents {
        NameTypeComponent: UseType<String>,
        GreeterComponent: GreetHello,
    }
}
```

The `Person` context is defined as a struct containing a `name` field of type `String`. We use the `#[derive(HasField)]` macro to automatically derive `HasField` implementations for every field in `Person`. This works together with the blanket implementation generated by `#[cgp_auto_getter]` for `HasName`, allowing `HasName` to be automatically implemented for `Person` without requiring any additional code.

Next, we declare a struct, `PersonComponents`, which is used to wire up the provider components for `Person`. We then implement `HasComponents` for `Person`, using `PersonComponents` to indicate that `Person` will utilize the providers specified in `PersonComponents`.

We use the `delegate_components!` macro to wire up `PersonComponents` with the necessary components. The first mapping, `NameTypeComponent: UseType<String>`, is a shorthand to associate the abstract `Name` type with `String`. The second mapping, `GreeterComponent: GreetHello`, indicates that we want to use `GreetHello` as the implementation of the `CanGreet` consumer trait.

The expressive use of `delegate_components!` makes it easy to rewire the components for `Person`. For instance, if we want to use a custom `FullName` struct for the name type, we can rewire `NameTypeComponent` to `UseType<FullName>`. Similarly, if there’s an alternative `Greeter` provider, `GreetLastName`, that implements `Greeter` with additional constraints, we can simply rewire `GreeterComponent` to use `GreetLastName` and add any necessary additional wiring to meet those constraints.

It’s important to note that CGP allows component wiring to be done _lazily_, meaning any errors (such as unsatisfied constraints) will only be detected when a consumer trait is actually used.

## Calling Greet

Now that the wiring is set up, we can construct a `Person` instance and call `greet` on it:

```rust
fn main() {
    let person = Person {
        name: "Alice".into(),
    };

    // prints "Hello, Alice!"
    person.greet();
}
```

This is made possible by a series of blanket implementations generated by CGP. Here's how the magic works:

- We can call `greet` because `CanGreet` is implemented for `Person`.
- `PersonComponents` contains the wiring that uses `GreetHello` as the provider for `GreeterComponent`.
- `GreetHello` implements `Greeter<Person>`.
- `Person` implements `HasName` via the `HasField` implementation.
- `Person::Name` is `String`, which implements `Display`.

There’s quite a bit of indirection happening behind the scenes!

By the end of this tutorial, you should have a high-level understanding of how programming in CGP works. There's much more to explore regarding how CGP handles the wiring behind the scenes, as well as the many features and capabilities CGP offers. To dive deeper, check out our book [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).

# Key Features

This section highlights some of the key advantages that Context-Generic Programming (CGP) offers.

## Modular Component System

CGP leverages Rust's powerful trait system to define generic component _interfaces_ that decouple the code that _consumes_ an interface from the code that _implements_ it. This is achieved by introducing:

- **Provider traits**, which define the implementation of a component interface.
- **Consumer traits**, which specify how a component interface is consumed.

By separating provider traits from consumer traits, CGP enables multiple context-generic provider implementations to coexist. This approach circumvents Rust's usual limitation on overlapping or orphaned trait implementations, offering greater flexibility and modularity.

## Highly Expressive Code

CGP empowers developers to write _abstract programs_ that are generic over a context, including all its associated types and methods. This capability eliminates the need to explicitly specify an extensive list of generic parameters in type signatures, streamlining code structure and readability.

Additionally, CGP offers powerful _macros_ for defining component interfaces and simplifies the process of wiring component implementations for use with a specific context.

With CGP, Rust code can achieve a level of expressiveness comparable to, if not exceeding, that of other popular programming paradigms, such as object-oriented programming and dynamically typed programming.

## Type-Safe Composition

CGP leverages Rust's robust type system to guarantee that all component wiring is _type-safe_, ensuring that any missing dependencies are caught at compile time. It operates entirely within safe Rust, avoiding dynamic typing techniques such as `dyn traits`, `Any`, or runtime reflection.

This strict adherence to type safety ensures that no CGP-specific errors can occur during application runtime, providing developers with greater confidence in their code's reliability.

## No-Std Friendly

CGP enables the creation of _fully abstract programs_ that can be defined without relying on any concrete dependencies — except for other abstract CGP components. This abstraction extends to dependencies such as I/O, runtime, cryptographic operations, and encoding schemes, allowing these concerns to be separated from the core application logic.

As a result, the core logic of an application can be seamlessly instantiated with specialized dependencies, making it compatible with no-std environments. These include embedded systems, operating system kernels, sandboxed environments like WebAssembly, and symbolic execution platforms such as Kani.

## Zero-Cost Abstraction

CGP operates entirely at compile-time, leveraging Rust's type system to ensure correctness without introducing runtime overhead. This approach upholds Rust's hallmark of _zero-cost abstraction_, enabling developers to use CGP's features without sacrificing runtime performance.

# Problems Solved

Here are some common problems in Rust that CGP helps to address.

## Error Handling

Rather than being tied to a specific error crate like `anyhow` or `eyre`, CGP's `HasErrorType` and `CanRaiseError` traits allow the decoupling of core application logic from error handling. This enables concrete applications to choose their preferred error library and select the error-handling strategy that best suits their needs, such as deciding whether or not to include stack traces in errors.

For more detailed information on error handling, refer to the [error handling chapter](https://patterns.contextgeneric.dev/error-handling.html) in our book

## Async Runtime

Rather than committing to a specific runtime crate like `tokio` or `async-std`, CGP enables the application core logic to rely on an abstract runtime context that provides only the features required by the application.

Unlike monolithic runtime traits, an abstract runtime context in CGP does _not_ require a comprehensive or upfront design of all possible runtime features any application might need. This flexibility allows easy switching between concrete runtime implementations, depending on the specific runtime features the application utilizes.

## Overlapping Implementations

A common frustration among Rust programmers is the restriction on overlapping trait implementations. A typical workaround is to use newtype wrappers, but this can become cumbersome when dealing with multiple composite types that need to be extended.

Rust requires a crate to own either the type or the trait for a trait implementation, which often places a significant burden on the author of a new type to implement all the common traits their users might need. This can lead to bloated type definitions, with excessive trait implementations such as `Eq`, `Clone`, `TryFrom`, `Hash`, and `Serialize`. Despite careful design, libraries may still face requests from users to implement less common traits, which can only be implemented by the crate that owns the type.

With the introduction of _provider traits_, CGP removes these restrictions on overlapping implementations. Both the owner and non-owners of a type can define custom implementations for that type. When multiple provider implementations are available, users can choose one and wire it up easily using CGP constructs.

CGP also favors the use of _abstract types_ over newtype wrappers. For instance, a type like `f64` can be directly used for both `Context::Distance` and `Context::Weight`, with the associated types still treated as distinct within the abstract code. CGP also enables specialized provider implementations, even if the crate does not own the primitive type (e.g., `f64`) or the provider trait.

## Dynamic Dispatch

A common approach for newcomers to support polymorphism in Rust is to use dynamic dispatch with `dyn Trait` objects. However, this severely limits the functionality to a restricted subset of _dyn-compatible_ (object-safe) features in Rust. Often, this limitation spreads throughout the entire codebase, requiring non-trivial workarounds for non-dyn-compatible constructs, such as `Clone`.

Even when dynamic dispatch is not used, many Rust programmers rely on ad-hoc polymorphism, defining enums to represent all potential variants of types in the application. This results in numerous `match` expressions scattered across the codebase, making it difficult to decouple logic for each branch. Additionally, adding new variants to the enum becomes challenging, as every branch must be updated, even when the new variant is only used in a small portion of the code.

CGP provides several solutions to address the dynamic dispatch problem by delegating the "assembly" of the variant collection to the concrete context. The core application logic can be written generically over the context and the associated type representing the abstract enum. CGP also facilitates powerful datatype-generic patterns that allow providers for each variant to be implemented separately and combined to work with enums that contain any combination of variants.

## Monolithic Traits

Even without CGP, Rust's trait system provides powerful mechanisms for building abstractions that would be difficult to achieve in other mainstream languages. One common best practice is to write abstract code that is generic over a context type, but this often involves an implicit trait bound tied directly to the generic context.

Unlike CGP, traits in this pattern are typically designed as monolithic, encompassing all the dependencies that the core application might need. Without CGP, an abstract caller must also include all trait bounds required by the generic functions it invokes. As a result, any additional generic trait bounds tend to propagate throughout the codebase, leading developers to combine all these trait bounds into one monolithic trait for convenience.

Monolithic traits can quickly become bottlenecks that prevent large projects from scaling. It's not uncommon for such traits to become bloated with dozens or even hundreds of methods and types. This overgrowth makes it increasingly difficult to introduce new implementations or modify existing ones. Additionally, with Rust's current practices, breaking down or decoupling these monolithic traits into smaller, more manageable traits can be challenging.

CGP offers significant improvements over this traditional pattern, making it possible to write abstract Rust code without the risk of creating unwieldy, monolithic traits. CGP enables the decomposition of large traits into many small, focused traits, each ideally consisting of just a single method or type. This is made possible by the dependency injection pattern used in CGP, which allows implementations to introduce only the minimal trait bounds they need directly within the implementation, rather than bundling everything into a single, monolithic structure.

# Getting Started

To get started with CGP, the best approach is to dive into our book, [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/). It provides a comprehensive guide to understanding and working with CGP.

You can also explore real-world applications and projects that use CGP, such as [Hermes SDK](https://github.com/informalsystems/hermes-sdk/), to gain a deeper understanding of its practical uses.

Additionally, be sure to check out the [Resources](/resources) page for more materials and learning tools to help you get up to speed with CGP.

# Contribution

We welcome contributors who are passionate about promoting CGP within the Rust ecosystem. Whether you're a beginner or an experienced Rust developer, there are numerous ways you can contribute to the project.

In this section, we'll explore different ways you can get involved and help grow the CGP community. Your contributions, regardless of your level of expertise, are valuable and appreciated!

## Read The Documentation

We encourage you to explore the documentation available on this website, including the [CGP Patterns](https://patterns.contextgeneric.dev) book. Your feedback is invaluable to us—if you encounter anything confusing or unclear, please let us know so we can improve the content and make it more accessible to everyone.

## Participate in Discussions

Join the conversation on platforms like [GitHub](https://github.com/orgs/contextgeneric/discussions) or [Reddit](https://www.reddit.com/r/cgp/). Whether you have questions about CGP or ideas for new topics or content, these forums are great places to share your thoughts and engage with the community.

## Spread on Social Media

Help raise awareness of CGP by sharing it on social media. Follow our official BlueSky account [@contextgeneric.dev](https://bsky.app/profile/contextgeneric.dev) to stay updated on CGP’s development and latest news.

## Write About It

If you find CGP interesting, consider writing your own blog posts or tutorials to share your learning journey. Sharing your insights can help others learn CGP in different ways, and even if the topic is already covered on the official site, your perspective might make it clearer to others.

## Contribute to Design

CGP currently lacks a logo, and our website uses a simple [Zola theme](https://juice.huhu.io/). If you have design experience and want to [contribute](https://github.com/contextgeneric/contextgeneric.dev), we would greatly appreciate your help in enhancing the website's design.

Additionally, we have a limited personal budget for professional design work. If you know a designer who could assist us, please feel free to recommend them.

# Acknowledgement

CGP was created by [Soares Chen](https://maybevoid.com/), with inspiration drawn from various programming languages and paradigms, particularly Haskell typeclasses.

The development of CGP would not have been possible without the strong support of my employer, [Informal Systems](https://informal.systems/). CGP was initially introduced and refined as part of the [Hermes SDK](https://github.com/informalsystems/hermes-sdk/) project, which leverages CGP to build a highly modular relayer for inter-blockchain communication.

(p.s. We are hiring [Rust engineers](https://informalsystems.bamboohr.com/careers/57) to work on Hermes SDK and CGP!)
