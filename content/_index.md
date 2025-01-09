+++

title = "Context-Generic Programming"

insert_anchor_links = "heading"

+++

# Announcement

Welcome to Context-Generic Programming! If you are new here, please read the
[announcement blog post](/blog/early-preview-announcement/) about the launch of
the project.

# Introduction

Context-Generic Programming (CGP) is a new programming paradigm in Rust that enables the creation of strongly-typed components, which can be implemented and composed in a modular, generic, and type-safe manner. This approach brings several advantages to building robust and maintainable systems.

On this homepage, we provide a quick overview and highlight the key features of CGP. For a deeper dive into the concepts and patterns of CGP, explore our comprehensive book, [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).

# Current Status

As of the start of 2025, CGP remains in its _early stages_ of development. While promising, it still has several rough edges, particularly in areas such as documentation, tooling, debugging techniques, community support, and ecosystem maturity.

As such, adopting CGP for serious projects comes with inherent challenges, and users are advised to proceed _at their own risk_. The primary risk is not technical but stems from the limited support available when encountering difficulties in learning or applying CGP.

At this stage, CGP is best suited for early adopters and potential [contributors](#contribution) who are willing to experiment and help shape its future.

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

# Problems Solved

Here are some example common problems in Rust that CGP helps to solve.

## Error Handling

Instead of choosing a specific error crate like `anyhow` or `eyre`, the
CGP traits `HasErrorType` and `CanRaiseError` can be used to decouple
the application core logic from error handling.
Concrete applications can freely choose specific error library, as well as
suitable strategies such as whether to include stack traces inside the error.

## Async Runtime

Instead of choosing a specific runtime crate like `tokio` or `async-std`,
CGP allows application core logic to depend on an abstract runtime context
that provide features that only the application requires.

Compared to monolithic runtime traits, an abstract runtime context in
CGP does _not_ require comprehensive or up front design of all possible
runtime features application may need. As a result, CGP makes it easy
to switch between concrete runtime implementations, depending on which
runtime feature the application actually uses.

## Overlapping Implementations

A common frustration among Rust programmers is the restrictions on
potentially overlapping implementations of traits.
A common workaround for the limitation is to use newtype wrappers.
However, the wrapping can become complicated, when there are multiple
composite types that need to be extended.

As Rust requires a crate to own either the type or the trait for a
trait implementation, this often places significant burden on the
author that defines a new type to implement all possible common traits
their users may need. This often leads to type definitions accompanied
by overly bloated implementations of traits such as `Eq`, `Clone`,
`TryFrom`, `Hash`, and `Serialize`. But even with great care, the library
could still get requests from users to implement one of the less common
traits that only the owner of the type can implement.

With the introduction of _provider traits_, CGP removes the restrictions
on overlapping implementations altogether. Both owner and non-owners
of a type can define a custom implementation for the type. When multiple
provider implementations are available, users can choose one of them, and
easily wire up the provider using CGP constructs.

CGP also prefer the use of _abstract types_ over newtype wrappers. For
example, a type like `f64` can be used directly to for both
`Context::Distance` and `Context::Weight`, with the associated types
still treated as different types inside the abstract code. CGP also
makes it possible for specialized providers to be implemented, even
if the crate do not own the primitive type `f64` or the provider trait.

## Dynamic Dispatch

A common attempt for newcomers to support polymorphism in Rust code is
to use dynamic dispatch in the for of `dyn Trait` objects. However, this
severely limits what can be done in the code to a limited subset of
_object-safe_ features in Rust. Very often, this limitation can be
infectious to the entire code base, and require non-trivial workaround
on non-object-safe constructs such as `Clone`.

Even when dynamic dispatch is not used, many Rust programmers also resort
to ad-hoc polymorphism, by defining enums to represent all possible variants
of types that may be used in the application. This leads to many `match`
expressions scattered across the code base, making it challenging to
decouple the code for each branch. Furthermore, this approach makes it
very difficult to add new variants to the enum, as all branches have to
be updated, even when the variant is only used in a small part of the code.

CGP provides multiple ways to solve the dynamic dispatch problem, by leaving
the "assembly" of the collection of variants to the concrete context.
Meanwhile, the core application logic can be written to be generic over
the context, together with the assocaited type that represents the abstract enum.
CGP also enables powerful data-generic pattern that allows providers of
each variant to be implemented separately, and then be combined to work
with enums that contain any combination of the variants.

## Monolithic Traits

Even without CGP, Rust' trait system already provides powerful ways for
programmers to build abstractions that would otherwise not be possible
in other mainstream languages. One of the best practices is similar
to CGP, which is to write abstract code that is generic over a context
type, except that there is an implicit trait bound that is always
tied to the generic context.

Unlike CGP, the trait in this pattern is often designed as a monolithic
trait that contains _all_ dependencies that the core application may need.
This is because without CGP, an abstract caller would have to also include
all the trait bounds that are specified by the generic functions it calls.
This means that any extra generic trait bound would easily propagate to
the entire code base. And when that happens, developers would just combine
all trait bounds into one monolithic trait for convenience sake.

Monolithic traits can easily become the bottleneck that prevents large projects
from scaling further. It is not uncommon for monolithic traits to be bloated
with dozens, if not hundreds, of methods and types. When that happens, it
becomes increasingly difficult to introduce new implementations to such
monolithic trait.
With the current practices in Rust, it is also challenging to decouple or
break down such monolithic trait to multiple smaller traits.

CGP offers significant improvement over this original design pattern,
and makes it possible to write abstract Rust code without the risk of
introducing a giant monolithic trait.
CGP makes it possible to to break monolithic traits down to many
small traits, which in fact, could and _should_ be as small as _one_
method or type per trait. This is made possible thanks to the
dependency injection pattern used by CGP, which allows implementations
only introduce the minimal trait bounds they need directly within the
body of the implementation.

# Getting Started

The best way to get started is to start reading the book
[Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).
You can also learn about how CGP works by looking at real world projects
such as [Hermes SDK](https://github.com/informalsystems/hermes-sdk/).

Also check out the [Resources](/resources) page to find out more resources
for learning CGP.

# Contribution

We are looking for any contributor who can help promote CGP to the wider
Rust ecosystem. Regardless of your level of understanding in CGP and Rust,
there are many ways you can help contribute to the project. This section
covers some of the ways you can contribute to the CGP community.

## Read The Documentation

You can read the documentation linked on this website, such as the
[CGP book](https://patterns.contextgeneric.dev), and give feedback on how
the content can be improved. If there is anything that is confusing or difficult
to understand, do let us know so that we can improve upon it.

## Participate in Discussions

You can participate in online discussion forums on
[GitHub](https://github.com/orgs/contextgeneric/discussions) or
[Reddit](https://www.reddit.com/r/cgp/). If you have any questions about CGP,
you can ask about them at the forum. If there is any specific topic or content
that you would like to read about, you can also share the ideas at the forum.

## Spread on Social Media

You can help raise the awareness of CGP by talking about it on social media.
We have an official BlueSky account
[@contextgeneric.dev](https://bsky.app/profile/contextgeneric.dev), so do
follow us to keep up to date on the development of CGP.

## Write About It

If you find CGP interesting, it would help a lot if you can write your own blog posts
and share your progress in learning CGP. You could also write your own tutorial series
to help others learn CGP. Since everyone has different ways of learning things, it is
always good to have different ways to explain CGP, even if it is something that has
already been explained on this official website.

## Help in Design

We do not yet have a logo for CGP, and the website is using a simple
[Zola theme](https://juice.huhu.io/). If you are experienced in design and would
like to contribute, it would be awesome if you can help improve the design of the website.

We also have a limited (personal) budget to pay for any professional design work. So if you
know of anyone who may be suitable for such work, we would like to hear your recommendation.

# Acknowledgement

CGP is invented by [Soares Chen](https://maybevoid.com/), with learnings and
inspirations taken from many related programming languages and paradigms,
particularly Haskell typeclasses.

The development of CGP would not have been possible without strong support
from my employer, [Informal Systems](https://informal.systems/). In particular,
CGP was first introduced and evolved from the
[Hermes SDK](https://github.com/informalsystems/hermes-sdk/) project,
which uses CGP to build a highly modular relayer for inter-blockchain communication.
(p.s. we are also hiring [Rust engineers](https://informalsystems.bamboohr.com/careers/57)
to work on Hermes SDK and CGP!)