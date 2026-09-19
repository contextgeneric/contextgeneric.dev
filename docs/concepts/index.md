---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Concepts

CGP lets a type choose reusable trait implementations through compile-time wiring. These pages
explain why that design works, what you can build with it, and when its costs outweigh the benefits.
They assume familiarity with Rust traits and generic bounds; you can read them without running code.

The [tutorials](/docs/tutorials/hello) introduce CGP by building working programs. The
[reference](/docs/reference/) specifies individual constructs and their syntax. Concepts connects
those constructs to the problems they solve, with examples, tradeoffs, and links to both.

## Where to start

Choose an entry point for the question you want answered:

- **Why does CGP exist?** [Bypassing coherence](./coherence.md) explains how Rust keeps trait
  implementations unambiguous and how CGP allows reusable alternatives within those rules.
- **How does CGP work?** [Consumer and provider traits](./consumer-and-provider-traits.md) explains
  the split between the trait callers use and the trait providers implement, then traces a method call.
- **Should I use CGP?** [Modularity Hierarchy](./modularity-hierarchy.md) compares ordinary traits,
  blanket implementations, and wired components so you can choose what your problem requires.

If traits and generic bounds are unfamiliar, start with the [tutorials](/docs/tutorials/hello).
They introduce the ideas through working code and provide the background these explanations assume.

## The ideas, grouped

The sidebar follows a suggested reading order. The groups below let you find related ideas directly.

### Traits and implementation choices

These pages explain the problem CGP addresses and the mechanism it uses:

- [Bypassing coherence](./coherence.md): Why Rust restricts overlapping implementations and how
  separate provider types let alternative implementations coexist.
- [Consumer and provider traits](./consumer-and-provider-traits.md): How the trait split and wiring
  connect a method call to a provider.
- [Modularity Hierarchy](./modularity-hierarchy.md): How to choose how much of CGP a program needs.

### Dependencies

A provider can state requirements that its callers do not need to carry. These pages explain the
forms those requirements take:

- [Impl-side dependencies](./impl-side-dependencies.md): Traits, values, and types required by an
  implementation
  without appearing in its public interface.
- [Implicit arguments](./implicit-arguments.md): Values read from context fields.
- [Abstract types](./abstract-types.md): Types selected by a context and shared by generic code.

### Composition and wiring

These pages explain how to combine implementations, share wiring, and verify the result:

- [Checking your wiring](./check-traits.md): Why wiring errors can remain undetected until use, and
  how compile-time checks report missing requirements near the wiring.
- [Higher-order providers](./higher-order-providers.md): Providers that take other providers as parameters.
- [Aggregate providers](./aggregate-providers.md): Groups of provider choices reused across contexts.
- [Namespaces](./namespaces.md): Shared wiring organized by paths and inherited by contexts.

### Applications of the design

These pages apply the same component model to errors, computations, and data:

- [Modular error handling](./modular-error-handling.md): Choosing an error type and the providers
  that construct errors and attach detail.
- [Handlers](./handlers.md): Computations represented as composable components.
- [Monadic handlers](./monadic-handlers.md): Composing computations through a monad.
- [Extensible records](./extensible-records.md): Generic operations over struct fields.
- [Extensible variants](./extensible-variants.md): Generic operations over enum variants.
- [Dispatching](./dispatching.md): Routing variants to their handlers.
- [Type-level DSLs](./type-level-dsls.md): Representing a small language as types resolved by the compiler.
- [Recovering `Send` bounds](./send-bounds.md): Requiring sendable futures when spawning async CGP tasks.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
