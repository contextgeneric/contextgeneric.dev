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

First, we would import `cgp` and define a greeter component as follows:

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

The `cgp` crate provides all common constructs inside the `prelude` module,
which should be imported in many cases. The first CGP construct we use is
the `cgp_component` macro, which generates additional CGP constructs for
the greeter component. The macro target, `CanGreet`, is a _consumer trait_
that is used similar to regular Rust traits, except that we will not implement anything directly on this trait.

The first `name` argument, `GreeterComponent`, is used as the _name_ of
the greeter component we defined. The second argument, `provider`, is used
as the name of a _provider trait_ called `Greeter`, which we would use for writing implementations for the greeter component. `Greeter` has similar structure as the `CanGreet`,
but with the implicit `Self` type replaced with the generic type `Context`.

## A Name Dependency

Next, we would introduce some dependencies that we may need when implementing a provider for `Greeter`. We first declare an abstract `Name` type as follows:

```rust
cgp_type!( Name )
```

Here, we use the `cgp_type!` macro to define a CGP component for the abstract type `Name`. `cgp_type!` produces the same effect as using `#[cgp_component]`, but offers a shorter syntax and derives some additional implementations that we can use later. For the purpose of this demo, it is sufficient to know that `cgp_type!` generates a `HasNameType` consumer trait, which contains an _associated type_ `Name`.

Next, we would define an _accessor trait_ for getting the name value from a context:

```rust
#[cgp_auto_getter]
pub trait HasName: HasNameType {
    fn name(&self) -> &Self::Name;
}
```

The `HasName` trait includes `HasNameType` as its super trait, in order to access the abstract type `Self::Name`. The trait contains a `name` method, which returns a reference to a name value of type `Self::Name`.

We use the `#[cgp_auto_getter]` attribute macro on the `HasName` trait, which generates a blanket implementation that enables any context that contains a field `name: Self::Name` to automatically implement the `HasName` trait.

## Hello Greeter

The traits `CanGreet`, `HasNameType`, and `HasName` can all be implemented independently over different modules or crates. But we can import them all in one place and implement a `Greeter` provider that uses `HasName` inside its implementation:

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

We first define `GreetHello` as a struct, which we then use to implement the `Greeter` provider trait. Compared to the consumer trait, the original `Self` type inside `CanGreet` is replaced with an _explicit_ generic type `Context`, which would place the same role as `Self` from `CanGreet`. On the other hand, the `GreetHello` type is used as the `Self` type in `Greeter`, but we won't be referencing `self` values anywhere inside a provider trait implementation.

The provider implementation is followed by two additional constraints: the contraint `Context: HasName` requires the context to additionally implement `HasName`, and the constraint `Context::Name: Display` allows the provider to work with any abstract `Name` type, as long as it implements `Display`. Notice that the two constraints are not specified anywhere at the trait bounds for `CanGreet` or `Greeter`. By including them only at the impl-side, we are effectively enabling _dependency injection_ of values and also _types_ through Rust's trait system.

Inside the `greet` method, we accept a parameter `context: &Context`, in contrast to the original `&self` argument inside `CanGreet::greet`. We can then call `context.name()` to get the name value from the context, and then print out a greeting using `println!`.

The provider implementation of `GreetHello` is defined to be _generic_ over any `Context` and `Context::Name` types, as long as the corresponding constraints for `HasName` and `Display` are satisfied. With the separation of provider trait from consumer trait, multiple providers like `GreetHello` can _all_ have generic implementation over any `Context`, without causing any issue of overlapping implementation that is usually imposed by Rust's trait system.

## Person Context

Next, we will define a concrete context `Person`, and wire it up to use `GreetHello` to implement `CanGreet`:

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

The `Person` context is defined to be a struct containing a `name` field, which is of type `String`. The macro `#[derive(HasField)]` is used here to derive `HasField` instances for every field that `Person` has. This would work together with the blanket implemented generated by `#[cgp_auto_getter]` for `HasName`, allowing `HasName` to be automatically implemented for `Person` with no additional code.

Next, we declare a struct `PersonComponents`, which is used for the wiring of provider components for `Person`. We implement `HasComponents` for `Person` using `PersonComponents`, to indicate that we want `Person` to use the providers wired by `PersonComponents`.

We then use the macro `delegate_components!` to wire up `PersonComponents` with the required components. The first mapping, `NameTypeComponent: UseType<String>`, is a shorthand for us to implement the abstract `Name` type as `String`. The second mapping, `GreeterComponent: GreetHello`, indicates that we want to use `GreetHello` to implement `CanGreet`.

The expressive use of `delegate_components!` allows us to easily rewire the components that we want to use for `Person`. Let's say we want to use a custom `FullName` struct as the name type, we can just rewire the `NameTypeComponent` with `UseType<FullName>`. Similarly, suppose if there is an alternative `Greeter` provider, `GreetLastName`, that implements `Greeter` with additional constraints, we can simply rewire `GreeterComponents` to use `GreetLastName`, and then add additional wirings to satisfy the additional constraints.

Note that CGP allows component wiring to be done _lazily_. This means that any error such as unsatisfied constraints will only be resolved when we try to _use_ a consumer trait.

## Calling Greet

Now that the wiring has been done, we can try to construct a `Person` and then call `greet` on it:

```rust
fn main() {
    let person = Person {
        name: "Alice".into(),
    };

    // prints "Hello, Alice!"
    person.greet();
}
```

If we try to build and run the above code, we will see that the code
compiles successfully, and the line `"Hello, Alice!"` is greeted on the
terminal.

This is made possible by a chain of blanket implementations generated by CGP: We can call `greet` because `CanGreet` is implemented by `Person`, because `PersonComponents` contains the wiring of `GreetHello` as the provider for `GreeterComponent`, because `GreetHello` implements `Greeter<Person>`, because `Person` implements `HasName` via `HasField`, and because `Person::Name` is `String` which implements `Display`. That is a lot of indirection going on!

Hopefully by the end of this tutorial, you have gotten a high level sense of how
it is like to program in CGP.
There are a lot more to cover on how such wiring is done behind the scene
by CGP, and what else we can do with CGP.
You can continue and find out more by reading the book
[Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).

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