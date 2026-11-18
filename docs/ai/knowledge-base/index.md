---
title: 'The CGP Knowledge Base'
sidebar_label: 'Overview'
sidebar_position: 1
description: 'The public, agent-written record behind CGP''s documentation and agent skill: what it contains, who reads it, and how it relates to the source code.'
---

# The CGP Knowledge Base

[Context-Generic Programming (CGP)](/docs/) is a Rust library for writing several implementations
of a trait and choosing which one each type uses. Most of its documentation, both on this site and
in its [agent skill](/docs/ai/skills), draws on a public repository: the
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base). It records what each of
CGP's constructs means, how its macros and tools are built, and how the project presents itself. AI
agents write and maintain it, and agents are its main readers.

## What it is for

An agent working with CGP needs to understand its syntax, but neither its training nor the macro
source reliably provides that understanding. The source shows token-stream transformations rather
than the traits and impls they produce, while much of the published material shows syntax the
library no longer accepts. The knowledge base records what that syntax means so an agent can read
the explanation instead of reconstructing it or guessing.

That explanation includes the syntax each construct accepts and the code its macro expands to.
Alongside it, the knowledge base documents the internals of the macros and of
[cargo-cgp](/docs/tooling/cargo-cgp), the classes of compile error CGP produces, worked examples, and
comparisons with related ideas from other languages.

For everyday work, the agent skill condenses this record to fit in an agent's context alongside your
code and teaches it to read and write CGP. When the skill leaves a question unanswered, the agent
can turn to the complete record in the knowledge base. It might need to understand a macro's corner
case or implementation, for example, or find out why a diagnostic is worded a certain way.

## Who reads it

Because the knowledge base is written for agents, its documents assume the skill's vocabulary and
favor completeness over a gentle introduction. They also record unfinished work and known defects
alongside current behavior. If you're learning CGP yourself, start with the
[tutorials](/docs/tutorials/hello) and the [Concepts](/docs/concepts) pages.

You may still find the knowledge base useful when you want the complete record of a construct,
want to check how a page on this site was produced, or are deciding whether to contribute.

## How it relates to the code

CGP's source code is the final authority. When a document disagrees with the code, the document is
wrong and is corrected. The knowledge base explains what the source means, the agent skill
condenses that explanation, and the pages on this site adapt it for people. Corrections follow the
same path, starting in the source or the knowledge base and then reaching the skill and the site.

## Its current state

Agents check the knowledge base against the source code, but the checks can miss a stale sentence
or an inconsistency. Some documents therefore still contain errors that review has yet to uncover.
Treat each document as the best available record, checked against the source but without a
guarantee that it is free of errors.

Reporting and fixing these errors helps the knowledge base reflect both the source and the
author's intent for CGP more accurately. If you find an error, file it on the
knowledge base's [issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues), as
[Contributing to CGP with an Agent](contributing-with-an-agent.md#report-problems-in-the-knowledge-base)
describes.

## What it contains

Each top-level section of the knowledge base has a `README.md` that lists its documents:

- **`cgp/`:** the CGP library, with a reference document for every construct, concept overviews,
  guides for choosing between constructs, a catalog of compile errors, and the macros' internals.
- **`cargo-cgp/`:** the error toolchain's usage, internals, open issues, and error codes.
- **`examples/`:** worked examples, each developing one use case, which supply most of the code on
  this site.
- **`related-work/`:** comparisons with the ideas CGP resembles, such as type classes, dependency
  injection, and ML modules.
- **`communication-strategy/`:** how CGP is presented in public, including the policy behind the
  [AI disclaimer](/docs/ai/disclaimer).
- **`website/`:** a record of each page on this site, and a guide for each kind of page.
- **`releases/`:** the version history, including when each construct was renamed or removed.
- **`projects/`:** the libraries built with CGP, such as Hypershell and cgp-serde.

The top-level files help an agent find its way before it opens a section:

- **[README.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/README.md):** how the
  knowledge base is organized, and what an agent that only reads it needs to know.
- **[summary.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/summary.md):** every
  document, each with a one-line summary.
- **[AGENTS.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/AGENTS.md):** the
  rules for writing and maintaining the documents.
- **[sibling-projects.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/sibling-projects.md):**
  the CGP repositories and where to find each one.

## Where to go next

Choose the page that answers your question:

- **How can I tell whether this documentation is reliable?** Read
  [How CGP's Documentation Is Written](how-cgp-is-documented.md).
- **How do I get my agent to use the knowledge base?** Read
  [Using the Knowledge Base with Your Agent](using-it-with-your-agent.md).
- **How do I contribute to CGP with an agent's help?** Read
  [Contributing to CGP with an Agent](contributing-with-an-agent.md).

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
