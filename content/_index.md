+++

title = "Context-Generic Programming"

+++

# Introduction

Context-generic programming (CGP) is a new programming paradigm for Rust that
allows strongly-typed components to be implemented and composed in a modular,
generic, and type-safe way.

## Modular Component System

CGP makes use of Rust's trait system to define generic component _interfaces_
that decouple code that _consumes_ an interface from code that _implements_ an interface.
This is done by having _provider traits_ that are used for implementing a
component interface, in addition to _consumer traits_ which are used for
consuming a component interface.

The separation of provider traits from consumer traits allows multiple context-generic
provider implementations to be defined, bypassing Rust's trait system's original restriction
that forbids overlapping implementations.

## Expressive Ways to Write Code

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
with _zero dependencies_. This allows such programs to be instantiated with
specialized-dependencies in no-std environments, such as on embedded systems,
operating system kernels, or Wasm sandboxes.

## Zero-Cost Abstraction

Since all CGP features work only at compile-time, it provides the same
_zero-cost abstraction_ advantage as Rust. Applications do not have to sacrifice
any runtime overhead for using CGP in the code base.

# Hello World Example

We will demonstrate various concepts of CGP with a simple hello world example.

## Greeter Component

First, we would import `cgp` and define a greeter component as follows:

```rust
use cgp::prelude::*;

#[derive_component(GreeterComponent, Greeter<Context>)]
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
    Context: HasField<symbol!("name"), Field: Display>,
{
    fn greet(context: &Context) {
        println!(
            "Hello, {}!",
            context.get_field(PhantomData::<symbol!("name")>)
        );
    }
}
```

The provider `GreetHello` is implemented as a struct, and implements
the provider trait `Greeter`. It is implemented as a
_context-generic provider_ that can work with any `Context` type,
but with additional constraints (or dependencies) imposed on the
context.

In this example case, the constraint
`HasField<symbol!("name"), Field: Display>` means that `GreetHello`
expects `Context` to be a struct with a field named `name`, with
the field type being any type that implements `Display`.

The trait `HasField` is a CGP getter trait for accessing fields in a
struct. The `symbol!` macro is used to convert any string literal
into types, so that they can be used as type argument. The
associated type `Field` is implemented as the type of the field in
the struct.

The `HasField` trait provides a `get_field` method,
which can be used to access a field value. The type
`PhantomData::<symbol!("name")>` is passed to `get_field` to infer
which field we want to read, in case if there are more than one
field in scope.

Notice that with the separation of provider trait from consumer trait,
multiple providers like `GreetHello` can _all_ have generic implementation
over any `Context`, without causing any issue of overlapping implementation
imposed by Rust's trait system.

Additionally, the provider `GreetHello` can require additional
constraints from `Context`, without those constraints bein present
in the trait bound of `CanGreet`. This concept is sometimes known as
_dependency injection_, as extra dependencies are "injected" into
the provider through the context.

Compared to other languages, CGP can not only inject methods into
a provider, but also _types_, as we seen with the `Field` associated
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
automatically implement `Person: HasField<symbol!("name"), Field = String>`,
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
`HasField<symbol!("name"), Field: Display>`.

Hopefully by the end of this tutorial, you have gotten a sense of how
it is like to program in CGP.
There are a lot more to cover on how such wiring is done behind the scene
by CGP, and what else we can do with CGP.
You can continue and find out more by reading the book
[Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).

# Current Status

As of end of 2024, CGP is still in _early-stage_ development, with many
rough edges in terms of documentation, tooling, debugging techniques,
community support, and ecosystem.

As a result, you are advised to proceed _at your own risk_ on using CGP in
any serious project. Note that the current risk of CGP is _not_ technical,
but rather the limited support you may get when encoutering any challenge
or difficulty in learning or using CGP.

Currently, the target audience for CGP are primarily early adopters and
[contributors](#contribution), preferrably with strong background in
_functional programming_.

# Getting Started

The best way to get started is to start reading the book
[Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).
You can also learn about how CGP works by looking at real world projects
such as [Hermes SDK](https://github.com/informalsystems/hermes-sdk/).

Also check out the [Resources](/resources) page to find out more resources
for learning CGP.

# Contribution

We are looking for any contributor who can help promote CGP to the wider
Rust ecosystem. The core concepts and paradigms are stable enough for
use in production, but we need contribution on improving documentation
and tooling.

You can also help promote CGP by writing tutorials, give feedback,
ask questions, and share about CGP on social media.

# Acknowledgement

CGP is invented by [Soares Chen](https://maybevoid.com/), with learnings and
inspirations taken from many related programming languages and paradigms,
particularly Haskell.

The development of CGP would not have been possible without strong support
from my employer, [Informal Systems](https://informal.systems/). In particular,
CGP was first introduced and evolved from the
[Hermes SDK](https://github.com/informalsystems/hermes-sdk/) project,
which uses CGP to build a highly modular relayer for inter-blockchain communication.
(p.s. we are also hiring [Rust engineers](https://informalsystems.bamboohr.com/careers/57)
to work on Hermes SDK and CGP!)