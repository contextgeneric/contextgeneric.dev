---
sidebar_position: 1
---

# Introduction

Context-Generic Programming (CGP) is a language extension for Rust, with pluggable trait
implementations at compile-time. You can define several implementations of an interface and choose
which one each application uses. CGP is a library on stable Rust: its macros generate ordinary traits
and implementations, and the compiler resolves the choices into direct calls.

For example, an application can send email through a mail server while a test records messages in
memory. Both use the same interface, and the code that calls it stays the same. Each application type
chooses its implementation separately.

CGP calls the type that holds these choices a **context**, and a named implementation a **provider**.
In the email example, the context represents the application and holds any state its providers need.
Wiring selects a provider for each component, the interface whose implementation you want to choose.
The [consumer and provider traits](/docs/concepts/consumer-and-provider-traits) explanation shows
how this separation works.

You can adopt CGP one component at a time. A consumer trait remains an ordinary Rust trait that you
can implement directly. Providers and wiring become useful when you need interchangeable
implementations or want to reuse an implementation across contexts. The
[Overview](/docs/overview) covers those features, along with abstract types, extensible data, and
composable handlers.

## Current Status

CGP is in active development, with a young ecosystem and limited community support. Learning its
patterns takes time, and compiler errors can be difficult to interpret. If you adopt it for a
mission-critical project, expect to investigate problems yourself when the available documentation
and support do not cover your case.

[cargo-cgp](/docs/cargo-cgp) helps diagnose wiring errors by naming their root cause and showing the
dependency chain. The tool is an early pre-release and rewrites the error classes it recognizes;
other diagnostics retain the compiler's wording. CGP also publishes an
[agent skill](/docs/ai/skills) to help coding assistants read, write, and debug CGP code. You still
need to review the code an assistant produces.

Start with a small part of your project whose implementation choices already vary. For a trait
with one implementation, a plain trait may be enough. CGP particularly welcomes
[early adopters and contributors](/docs/contribute) who want to experiment and help improve the
library, tools, and documentation.

## Getting Started

Start with the [Hello World tutorial](/docs/tutorials/hello) for a short working example. Continue
with the [area-calculation series](/docs/tutorials/area-calculation) to learn how ordinary Rust
functions develop into components and providers.

Use the documentation according to the question you need to answer:

- [Overview](/docs/overview): what CGP offers and where it helps.
- [Concepts](/docs/concepts): how its ideas work and fit together.
- [Reference](/docs/reference): the syntax and behavior of individual constructs.
- [Glossary](/docs/reference/glossary): short definitions of the terms used across this documentation.
- [Resources](/docs/resources): libraries, projects, talks, and further reading.

The [Context-Generic Programming Patterns book](https://patterns.contextgeneric.dev/) develops CGP
from first principles. It is useful for understanding the underlying patterns, but its examples do
not consistently match the current library. Use the maintained tutorials and reference when writing
new code. The [blog](/blog) records releases, design discussions, and talks; older posts describe
the library as it was when they were published.

---

*An AI agent revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
