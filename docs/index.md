---
title: 'Introduction: pluggable trait implementations'
sidebar_label: 'Introduction'
sidebar_position: 1
description: 'What Context-Generic Programming is, the four words every page of its documentation uses, and where to go for each question you bring.'
---

# Introduction

Context-Generic Programming (CGP) is a language extension for Rust, with pluggable trait
implementations at compile-time. It lets you write several implementations of one interface and
have each application choose the one it uses. That may sound like a new language to learn, but CGP
is an ordinary library on stable Rust: its macros generate plain traits and implementations, the
compiler turns each choice into a direct call, and you can adopt it one trait at a time.

A small example shows what that choice looks like. Suppose an application fetches stored files from
Amazon S3, a second deployment of it fetches them from Google Cloud Storage, and its tests read them
from a local folder. All three go through the same interface, so the code that asks for a file stays
the same. The only difference between them is one line in each application type, which names the
implementation that application uses.

Four words describe the parts of that arrangement, and the rest of this documentation uses them
throughout:

- **[Context](/docs/reference/glossary#context):** a type that stands for one set of choices, such
  as an application, and holds the data its implementations need.
- **[Component](/docs/reference/glossary#component):** an interface whose implementation a context
  chooses.
- **[Provider](/docs/reference/glossary#provider):** a named implementation of a component.
- **[Wiring](/docs/reference/glossary#wiring):** the table on a context that selects a provider for
  each component, resolved at compile time.

Because the trait that callers use stays an ordinary Rust trait, you can still implement it
directly, and bring in providers and wiring only where interchangeable implementations or reuse
make them worth having. [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits)
shows how the two sides fit together.

## Start here

If you want to write code, these pages take you from installing CGP to wiring and checking
components of your own, in this order:

1. [Quickstart](/docs/quickstart): install CGP and run a first program in a few minutes.
2. [Hello World](/docs/tutorials/hello): a program like the Quickstart's, then the idea that makes
   it worth using.
3. [Area calculation](/docs/tutorials/area-calculation): a series that grows ordinary Rust functions
   into components and providers, and ends with how to check wiring and read its errors.

## Find what you need

If you arrived with a particular question, each line below names it and the part of the
documentation that answers it:

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

The [blog](/blog) is a record of releases, design discussions, and talks. Its older posts describe
the library as it was when they were published, so the pages above are the better guide to CGP as
it is now.

---

*An AI agent revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
