---
sidebar_position: 2
---

# Overview

This page provides a quick overview and highlight the key features of CGP. For a deeper dive into the concepts and patterns of CGP, explore our comprehensive book, [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/).

# Key Features

This section highlights some of the key advantages that Context-Generic Programming (CGP) offers.

## Modular Component System

CGP leverages Rust's powerful trait system to define generic component _interfaces_ that decouple the code that _consumes_ an interface from the code that _implements_ it. This is achieved by introducing:

- **Provider traits**, which define the implementation of a component interface.
- **Consumer traits**, which specify how a component interface is consumed.

By separating provider traits from consumer traits, CGP enables multiple context-generic provider implementations to coexist. This approach circumvents Rust's usual limitation on overlapping or orphaned trait implementations, offering greater flexibility and modularity.

## Highly Expressive Macros

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
