---
title: 'Introduction: pluggable trait implementations'
sidebar_label: 'Introduction'
sidebar_position: 1
description: 'What Context-Generic Programming is, the four words every page of its documentation uses, and where to go for each question you bring.'
---

# Introduction

Context-Generic Programming (CGP) is a language extension for Rust, with pluggable trait
implementations at compile-time. You write several implementations of one interface, and each
application chooses the one it uses. CGP is a library on stable Rust that you adopt one trait at a
time: its macros generate ordinary traits and implementations, and the compiler resolves each choice
into a direct call.

For example, an application can fetch stored files from Amazon S3, a second deployment of it from
Google Cloud Storage, and its tests from a local folder. All three use the same interface, so the
code that fetches a file stays the same, and each application type states which implementation it
uses.

Every page in this documentation uses four words for the parts of that arrangement:

- **[Context](/docs/reference/glossary#context):** a type that stands for one set of choices, such
  as an application, and holds the data its implementations need.
- **[Component](/docs/reference/glossary#component):** an interface whose implementation a context
  chooses.
- **[Provider](/docs/reference/glossary#provider):** a named implementation of a component.
- **[Wiring](/docs/reference/glossary#wiring):** the table on a context that selects a provider for
  each component, resolved at compile time.

The trait that callers use stays an ordinary Rust trait, so you can still implement it directly and
add providers and wiring only where interchangeable implementations or reuse justify them.
[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) shows how the split
works.

## Start here

These take you from installing CGP to wiring and checking your own components:

1. [Quickstart](/docs/quickstart): install CGP and run a first program in a few minutes.
2. [Hello World](/docs/tutorials/hello): a program like the Quickstart's, then the idea that makes
   it worth using.
3. [Area calculation](/docs/tutorials/area-calculation): a series that grows ordinary Rust functions
   into components and providers, and ends with how to check wiring and read its errors.

## Find what you need

Each line names a question and the part of the documentation that answers it:

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

The [blog](/blog) records releases, design discussions, and talks; older posts describe the library
as it was when they were published.

---

*An AI agent revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
