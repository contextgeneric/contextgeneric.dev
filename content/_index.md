+++

title = "Context-Generic Programming"

insert_anchor_links = "heading"

+++

# Announcement

Welcome to Context-Generic Programming! If you are new here, please read the
[announcement blog post](/blog/early-preview-announcement/) about the launch of
the project.

# Introduction

Context-generic programming (CGP) is a new programming paradigm for Rust that
allows strongly-typed components to be implemented and composed in a modular,
generic, and type-safe way. In this section, we will walk through some of the
advantages CGP provides.

## Modular Component System

CGP makes use of Rust's trait system to define generic component _interfaces_
that decouple code that _consumes_ an interface from code that _implements_ an interface.
This is done by having _provider traits_ that are used for implementing a
component interface, in addition to _consumer traits_ which are used for
consuming a component interface.

The separation of provider traits from consumer traits allows multiple context-generic
provider implementations to be defined, bypassing Rust's trait system's original restriction
that forbids overlapping implementations.

## Highly Expressive Code

With CGP, one can easily write _abstract programs_ that is generic over
a context, together with all its associated types and methods. CGP allows such
generic code to be written without needing to explicitly specify a long list
generic parameters in the type signatures.
CGP also provides powerful _macros_ for defining component interfaces, as well
as providing simple ways to wire up component implementations to be used with
a concrete context.

CGP allows Rust code to be written with the same level of expressiveness,
if not more, as other popular programming paradigms, including object-oriented programming
and dynamic-typed programming.

## Type-Safe Composition

CGP makes use of Rust's strong type system to help ensure that all wiring
of components is _type-safe_, catching any missing dependencies as compile-time
errors. CGP works fully within safe Rust, and does not make use of
any dynamic-typing techniques, e.g. `dyn` traits, `Any`, or reflection.
As a result, developers can ensure that no CGP-specific errors can happen
during application runtime.

## No-Std Friendly

CGP makes it possible to build _fully-abstract programs_ that can be defined
with _zero concrete dependencies_ (aside from other abstract CGP components).
What this means is that dependencies including I/O, runtime, cryptographic
operations, encoding schemes, can all be abstracted away from the core logic
of the application.

This allows the application core logic to be instantiated with
specialized dependencies in no-std environments, such as on embedded systems,
operating system kernels, sandboxed environments like Wasm, and symbolic
execution environments like Kani.

## Zero-Cost Abstraction

Since all CGP features work only at compile-time, it provides the same
_zero-cost abstraction_ advantage as Rust. Applications do not have to sacrifice
any runtime overhead for using CGP in the code base.

# Current Status

As of end of 2024, CGP is still in _early-stage_ development, with many
rough edges in terms of documentation, tooling, debugging techniques,
community support, and ecosystem.

As a result, you are advised to proceed _at your own risk_ on using CGP in
any serious project. Note that the current risk of CGP is _not_ technical,
but rather the limited support you may get when encoutering any challenge
or difficulty in learning or using CGP.

Currently, the target audience for CGP are primarily early adopters and
[contributors](#contribution).

# Hello World Example

We will demonstrate various concepts of CGP with a simple hello world example.

## Greeter Component

First, we would import `cgp` and define a greeter component as follows:

```rust
use cgp::prelude::*;

#[cgp_component {
    name: GreeterComponent,
    provider: Greeter,
    context: Context,
}]
pub trait CanGreet {
    fn greet(&self);
}
```

The `cgp` crate provides all common constructs inside the `prelude` module,
which should be imported in many cases. The first CGP construct we use is
the `derive_component` macro, which generates additional constructs for
the greeter component. The macro target, `CanGreet`, is a _consumer trait_
that is used similar to regular Rust traits, but is not for implementation.

The first macro argument, `GreeterComponent`, is used as the _name_ of
the greeter component we defined. The second argument is used
to define a _provider trait_ called `Greeter`, which is used for implementing
the greet component. `Greeter` has similar structure as the `CanGreet`,
but with the implicit `Self` type replaced with the generic type `Context`.

## Hello Greeter

With the greeter component defined, we would implement a hello greeter provider
as follows:

```rust
pub struct GreetHello;

impl<Context> Greeter<Context> for GreetHello
where
    Context: HasField<symbol!("name"), Value: Display>,
{
    fn greet(context: &Context) {
        println!(
            "Hello, {}!",
            context.get_field(PhantomData::<symbol!("name")>)
        );
    }
}
```

The provider `GreetHello` is defined as a struct, and implements
the provider trait `Greeter`. It is implemented as a
_context-generic provider_ that can work with any `Context` type,
but with additional constraints (or dependencies) imposed on the
context.

In this example case, the constraint
`HasField<symbol!("name"), Value: Display>` means that `GreetHello`
expects `Context` to be a struct with a field named `name`, with
the field type being any type that implements `Display`.

The trait `HasField` is a CGP getter trait for accessing fields in a
struct. The `symbol!` macro is used to convert any string literal
into types, so that they can be used as type argument. The
associated type `Value` is implemented as the type of the field in
the struct.

The `HasField` trait provides a `get_field` method,
which can be used to access a field value. The type
`PhantomData::<symbol!("name")>` is passed to `get_field` to help infer
which field we want to read, in case if there are more than one
field in scope.

Notice that with the separation of provider trait from consumer trait,
multiple providers like `GreetHello` can _all_ have generic implementation
over any `Context`, without causing any issue of overlapping implementation
that is usually imposed by Rust's trait system.

Additionally, the provider `GreetHello` can require additional
constraints from `Context`, without those constraints bein present
in the trait bound of `CanGreet`. This concept is sometimes known as
_dependency injection_, as extra dependencies are "injected" into
the provider through the context.

Compared to other languages, CGP can not only inject methods into
a provider, but also _types_, as we seen with the `Value` associated
type in `HasField`.

## Person Context

Next, we will define a concrete context `Person`, and wire it up to
use `GreetHello` to implement `CanGreet`:

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
        GreeterComponent: GreetHello,
    }
}
```

The `Person` context is defined to be a struct containing a `name` field,
which is of type `String`. The CGP macro `derive(HasField)` is used to
automatically implement `Person: HasField<symbol!("name"), Value = String>`,
so that it can be used by `GreetHello`.

Additionally, we also define an empty struct `PersonComponents`, which
is used to wire up all the providers for `Person`. We implement the
CGP trait `HasComponents` for `Person`, which sets `PersonComponents`
as its _aggregated provider_.

We use the CGP macro `delegate_components` to wire up the delegation of
providers for `PersonComponent`. The macro allows multiple components
to be listed in the body, in the form of `ComponentName: Provider`.
In this example, we only have one entry, which is to use `GreetHello`
as the provider for `GreeterComponent`. Notice that this is where we
use the component name `GreeterComponent`, which was defined earlier
by `derive_component`.

With the expressive mapping of components to provider inside
`delegate_components!`, we can easily switch the implementation of
`Greeter` to another provider by making just one line of code change.

Note that CGP allows component wiring to be done _lazily_. This means
that any error such as unsatisfied dependencies will only be resolved
when we try to _use_ the provider.

## Calling Greet

Now that the wiring has been done, we can try to construct a `Person`
and then call `greet` on it:

```rust
let person = Person {
    name: "Alice".into(),
};

// prints "Hello, Alice!"
person.greet();
```

If we try to build and run the above code, we will see that the code
compiles successfully, and the line "Hello, Alice!" is greeted on the
terminal.

The method `greet` is called from the consumer trait `CanGreet`, which
is implemented by `Person` via `PersonComponents`, which implements
`Greeter` via delegation of `GreeterComponent` to `GreetHello`,
which implements `Greeter` given that `Person` implements
`HasField<symbol!("name"), Value: Display>`.
That is a lot of indirection going on!

Hopefully by the end of this tutorial, you have gotten a sense of how
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