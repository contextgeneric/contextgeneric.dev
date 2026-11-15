---
title: 'The CGP Knowledge Base'
sidebar_label: 'Overview'
sidebar_position: 1
description: 'The public, agent-written record that CGP''s documentation and agent skill are written from: what it holds, who it is written for, and how it relates to the source code.'
---

# The CGP Knowledge Base

[Context-Generic Programming (CGP)](/docs/) is a Rust library for writing several implementations
of a trait and choosing which one each type uses. Most of its documentation, on this site and in its
[agent skill](/docs/ai/skills), is written from one public repository: the
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base). It records what each of
CGP's constructs means, how its macros and tools are built, and how the project presents itself. AI
agents write and maintain it, and agents are its main readers.

## What it is for

An agent working with CGP has to know what a piece of syntax means, and neither the macro source nor
its training tells it reliably. The macro source shows token-stream transformations rather than the
traits and impls they produce, and much of what has been published about CGP shows syntax the
library no longer accepts. The knowledge base writes that meaning down once, so an agent can read it
instead of reconstructing it or guessing. It records the syntax each construct accepts and the code
its macro expands to, the internals of the macros and of [cargo-cgp](/docs/tooling/cargo-cgp), the
classes of compile error CGP produces, worked examples, and comparisons with related ideas from
other languages.

The agent skill is the part of this record sized for everyday work: it fits in an agent's context
beside your code and teaches the agent to read and write CGP. The knowledge base is the complete
record behind it, for the questions the skill does not answer, such as a corner case of one macro,
how a macro is implemented, or why a diagnostic reads as it does.

## Who reads it

The knowledge base is written for agents, not for people learning CGP. Its documents assume the
skill's vocabulary, prefer completeness to a gentle introduction, and record unfinished work and
known defects beside current behavior. To learn CGP yourself, start with the
[tutorials](/docs/tutorials/hello) and the [Concepts](/docs/concepts) pages.

You can still read it yourself when you want the complete record of a construct, when you want to
check how a page on this site was produced, or when you are deciding whether to contribute.

## How it relates to the code

CGP's source code is the final authority. When a document disagrees with the code, the document is
wrong and is corrected. The knowledge base records what the source means, the agent skill condenses
that record, and the pages on this site rewrite it for people. A correction therefore starts in the
source or the knowledge base and moves outward to the skill and the site.

## Its current state

Agents check the knowledge base against the source code, but the checks can miss a stale sentence
or an inconsistency, so some documents still contain errors that no review has found yet. Read each
document as the best record available: checked against the source, but not guaranteed to be free of
errors.

Each error reported and fixed brings the knowledge base closer to full accuracy, meaning agreement
with the author's intent for CGP as well as with the source. If you find one, file it on the
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

Four files at the top level orient an agent before it opens any section:

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
