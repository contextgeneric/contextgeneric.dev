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
provider implementations to be defined, bypassing Rust's trait system original restriction
of forbidding overlapping implementations.

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