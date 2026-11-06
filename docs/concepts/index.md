---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Concepts

CGP connects reusable trait implementations to the types that use them through compile-time wiring.
Understanding that connection explains how providers obtain dependencies, how contexts choose shared
types and values, and how implementations compose into larger operations. These pages develop those
ideas alongside their costs. You can read them without running code, with familiarity with Rust
traits and generic bounds as a starting point.

If you want to learn by building a program, the [tutorials](/docs/tutorials/hello) introduce the
same ideas through working code. Once you need the syntax or precise behavior of a construct, the
[reference](/docs/reference/) supplies that detail.

## Where to start

Choose an entry point for the question you want answered:

- **Why does CGP exist?** [Bypassing coherence](./coherence.md) explains how Rust keeps trait
  implementations unambiguous and how CGP allows reusable alternatives within those rules.
- **How does CGP work?** [Consumer and provider traits](./consumer-and-provider-traits.md) explains
  the split between the trait callers use and the trait providers implement, then traces a method call.
- **Should I use CGP?** [Modularity Hierarchy](./modularity-hierarchy.md) compares ordinary traits,
  blanket implementations, and wired components so you can choose what your problem requires.

If you need more background on traits or generic bounds, the [tutorials](/docs/tutorials/hello)
introduce them in working programs before returning to these explanations.

## The ideas, grouped

The sidebar offers a reading order from the trait model to its applications. The groups below
bring related pages together, so you can also follow a particular question across the section.

### Traits and implementation choices

The trait pages begin with Rust's rules for choosing implementations, then explain how CGP builds
on them and where that additional structure is useful:

- [Bypassing coherence](./coherence.md): Why Rust restricts overlapping implementations and how
  separate provider types let alternative implementations coexist.
- [Consumer and provider traits](./consumer-and-provider-traits.md): How the trait split and wiring
  connect a method call to a provider.
- [Modularity Hierarchy](./modularity-hierarchy.md): How to choose how much of CGP a program needs.

### Dependencies

Once a caller depends on a consumer trait, the chosen provider can have further requirements of
its own. The dependency pages explain how those requirements reach the context as traits, values,
and types:

- [Impl-side dependencies](./impl-side-dependencies.md): Traits, values, and types required by an
  implementation without appearing in its public interface.
- [Implicit arguments](./implicit-arguments.md): Values read from context fields.
- [Abstract types](./abstract-types.md): Types selected by a context and shared by generic code.

### Composition and wiring

As implementations grow, wiring needs to express their relationships and remain practical to
maintain. These pages explain composition, shared tables, and checks on the resulting choices:

- [Checking your wiring](./check-traits.md): Why wiring errors can remain undetected until use, and
  how compile-time checks report missing requirements near the wiring.
- [Higher-order providers](./higher-order-providers.md): Providers that take other providers as parameters.
- [Aggregate providers](./aggregate-providers.md): Groups of provider choices reused across contexts.
- [Namespaces](./namespaces.md): Shared wiring organized by paths and inherited by contexts.

### Applications of the design

The same separation between interfaces, providers, and wiring supports more specialized designs.
These pages follow it into errors, computations, and generic operations on data:

- [Modular error handling](./modular-error-handling.md): Choosing an error type and the providers
  that construct errors and attach detail.
- [Handlers](./handlers.md): Computations represented as composable components.
- [Monadic handlers](./monadic-handlers.md): Composing computations through a monad.
- [Extensible records](./extensible-records.md): Generic operations over struct fields.
- [Extensible variants](./extensible-variants.md): Generic operations over enum variants.
- [Dispatching](./dispatching.md): Routing variants to their handlers.
- [Type-level DSLs](./type-level-dsls.md): Representing a small language as types resolved by the compiler.
- [Recovering `Send` bounds](./send-bounds.md): Requiring sendable futures when spawning async CGP tasks.

If you came here because a page used a term you did not recognize, the
[glossary](/docs/reference/glossary) defines the vocabulary used across this documentation and points
at the page that explains each term.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
