---
title: 'Introduction: pluggable trait implementations'
sidebar_label: 'Introduction'
sidebar_position: 1
description: 'Learn what Context-Generic Programming is, get to know its core terms, and find the documentation you need.'
---

# Introduction

Context-Generic Programming (CGP) is a language extension for Rust, with pluggable trait
implementations at compile-time. In plain Rust, a type has at most one implementation of each
trait. CGP lets you write several implementations of the same trait and decide, for each type that
uses it, which one applies.

None of this needs a new language or a new compiler. CGP is a library on stable Rust whose macros
expand to ordinary traits and implementations, and the compiler resolves every choice into a direct
call. You can bring it into an existing project one trait at a time.

The documentation uses these terms to describe how the pieces fit together:

- **[Context](/docs/reference/glossary#context):** a type, such as an application type, that
  holds the data its implementations need and identifies which implementations to use.
- **[Component](/docs/reference/glossary#component):** an interface whose implementation a context
  chooses.
- **[Provider](/docs/reference/glossary#provider):** a named implementation of a component.
- **[Wiring](/docs/reference/glossary#wiring):** the table on a context that selects a provider for
  each component, resolved at compile time.

## Start here

To start writing code, follow these pages from installation through wiring and checking your own
components:

1. [Quickstart](/docs/quickstart): install CGP and run a first program in a few minutes.
2. [Hello World](/docs/tutorials/hello): understand a program like the Quickstart's, then see the
   same function work on a second context.
3. [Area calculation](/docs/tutorials/area-calculation): turn ordinary Rust functions into
   components and providers, then learn to check their wiring and understand errors.

## Find what you need

If you have a particular question, use these links to find the relevant documentation:

- **What can CGP do, feature by feature?** The [Overview](/docs/overview).
- **Why does CGP work this way?** [Bypassing coherence](/docs/concepts/coherence) and
  [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits), then the rest of the
  [Concepts](/docs/concepts).
- **Does CGP fit my project?** [Modularity Hierarchy](/docs/concepts/modularity-hierarchy), and the
  [costs and maturity](/#costs) on the front page.
- **How does it compare with what I know?** The [Comparisons](/docs/comparisons), from dependency
  injection and type classes to ML modules and dynamic dispatch.
- **What does a construct do, or a term mean?** The [Reference](/docs/reference) and the
  [Glossary](/docs/reference/glossary).
- **How do I read a CGP compile error?** [cargo-cgp](/docs/cargo-cgp), CGP's error toolchain.
- **What has been built with it?** The [Projects](/docs/projects).
- **How do I use CGP with a coding agent?** The [AI section](/docs/ai), with CGP's agent skill.
- **Where are the crates, talks, and community?** [Resources](/docs/resources).
- **How can I help?** [Contribute](/docs/contribute).

For releases, design discussions, and talks, visit the [blog](/blog). Older posts describe the
library at the time of publication, so use the documentation above for guidance on current CGP.

---

*An AI agent revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
