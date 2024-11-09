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