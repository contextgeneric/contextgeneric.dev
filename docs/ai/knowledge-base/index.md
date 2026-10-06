---
title: 'The CGP Knowledge Base'
sidebar_label: 'Overview'
sidebar_position: 1
description: 'The public, agent-written record that CGP''s documentation and agent skill are written from: what it holds, who it is written for, and how it relates to the source code.'
---

# The CGP Knowledge Base

[Context-Generic Programming (CGP)](/docs/) is a language extension for Rust, with pluggable trait
implementations at compile-time. The
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base) is a public repository of
documents that record what CGP's constructs mean, how its macros and tools are built, and how the
project presents itself. AI agents write and maintain it, and agents are its main readers. The
reference pages on this site, most of its other documentation, and the
[agent skill](/docs/ai/skills) are written from it.

## What it is for

The knowledge base gives an agent a written record of CGP to read, so the agent does not have to
reconstruct CGP from the macro source or recall it from training. It records the semantics of each
construct, including the syntax forms it accepts and the code each macro expands to. It also holds
the internals of the macros and of [cargo-cgp](/docs/cargo-cgp), the classes of compile error CGP
produces, worked examples, and comparisons with related ideas from other languages.

The [agent skill](/docs/ai/skills) is the part of this record sized for everyday work: it fits in an
assistant's context and teaches it to read and write CGP code. The knowledge base is the exhaustive
record behind it, for questions the skill does not answer, such as a corner case of one macro, how a
macro is implemented, or why a diagnostic reads as it does.

## Who reads it

The knowledge base is written for agents, not for people learning CGP. Its documents assume the
agent skill's vocabulary, favor completeness over a gentle introduction, and record unfinished work
and known defects beside current behavior. To learn CGP, start with the
[tutorials](/docs/tutorials/hello) and the [Concepts](/docs/concepts) pages.

You can still read it directly. It is useful when you want the complete record of a construct, when
you want to check how a page on this site was produced, or when you are deciding whether to
contribute.

## Its current state

The knowledge base is in an early phase and under active development. Many of its documents still
need further review, by the author and by agents, and some contain errors or inconsistencies that no
review has found yet. Read any one document as the current best record rather than a finished one.

The aim is to improve the knowledge base step by step until it is fully accurate, meaning that it
agrees with the author's intent for CGP as well as with the source code. Finding a problem in it is
a useful contribution: file it on the knowledge base's
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues), as
[Contributing to CGP with an Agent](contributing-with-an-agent.md) describes.

## How it relates to the code

CGP's source code is the final authority. When a document in the knowledge base disagrees with the
code, the document is wrong and is corrected. The knowledge base records what the source means, the
agent skill condenses that record, and the pages on this site are written from the record in a form
meant for people. A correction therefore starts in the source or the knowledge base and moves
outward to the skill and the site.

## What it contains

The knowledge base is divided into top-level sections, each with a `README.md` that catalogs its
documents:

- **`cgp/`:** The CGP library, with a reference document for every construct, concept overviews,
  guides for choosing between constructs, a catalog of compile errors, and the macros' internals.
- **`cargo-cgp/`:** The error toolchain's usage, internals, open issues, and error codes.
- **`examples/`:** Worked examples, each developing one use case, which serve as the verified source
  of most code on this site.
- **`related-work/`:** Comparisons with ideas CGP resembles, such as type classes, dependency
  injection, and ML modules.
- **`communication-strategy/`:** How CGP is presented in public, including the policy behind the
  [AI disclaimer](/docs/ai/disclaimer).
- **`website/`:** A record of each page on this site and a guide for each kind of page.
- **`releases/`:** The version history, including when each construct was renamed or removed.
- **`projects/`:** Libraries built with CGP, such as Hypershell and cgp-serde.
- **`skills/`:** Writing skills that the knowledge base's rules ask an agent to load.

These files at the top level orient an agent before it opens any section:

- **[README.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/README.md):** how the
  knowledge base is organized, and what an agent that only reads it needs to know.
- **[summary.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/summary.md):** every
  document, with a one-line summary of each.
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
