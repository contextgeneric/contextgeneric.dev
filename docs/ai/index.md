---
title: 'AI-Assisted Development'
sidebar_label: 'Overview'
sidebar_position: 0
description: 'How to work with CGP through a coding agent, using its agent skill and knowledge base, and how the CGP project itself uses AI to write its documentation, tools, and tests.'
---

# AI-Assisted Development

[Context-Generic Programming (CGP)](/docs/) is a language extension for Rust, with pluggable trait
implementations at compile-time. This section covers CGP and AI coding agents in two directions:
help for you when you work on CGP code with an agent, and an account of how the CGP project uses
agents itself. The two are separate subjects, and the pages below keep them apart.

## Working with CGP through an agent

CGP publishes material that teaches a coding agent its vocabulary, its recommended syntax, and how
to read its compile errors. That can reduce the work of learning CGP, writing its wiring, and
reading its diagnostics, but it does not replace your own review: an agent can still produce wrong
code, and you remain the one who has to understand it.

Choose the page for what you want to do:

- **Have your agent read and write CGP code.** Start with [CGP's agent skill](/docs/ai/skills),
  which your agent loads and which covers everyday work.
- **Get a deeper answer than the skill gives.** Read
  [Using the Knowledge Base with Your Agent](/docs/ai/knowledge-base/using-it-with-your-agent) to
  point your agent at the full record behind the skill.
- **Contribute to CGP with an agent's help.** Read
  [Contributing to CGP with an Agent](/docs/ai/knowledge-base/contributing-with-an-agent) for the
  policy for AI-assisted contributions, how to report problems in the knowledge base, the setup, and
  the rules your agent follows.

To learn CGP yourself, start with the [tutorials](/docs/tutorials/hello) and the
[Concepts](/docs/concepts) pages instead. The skill and the knowledge base are written for agents.

## How the CGP project uses AI

AI agents write most of CGP's documentation and most of `cargo-cgp`, maintain much of CGP's test
suite, and assist with parts of the library's implementation. The author designs every CGP construct
and writes the core library almost entirely by hand. These pages explain the arrangement and how it
is checked:

- **Which parts of the project are written by AI, and who reviews them?** Read [How AI Is Used in
  the CGP Project](/docs/ai/disclaimer). New pages written with AI assistance link to it from a note
  at their foot.
- **How is agent-written documentation kept accurate?** Read
  [How CGP's Documentation Is Written](/docs/ai/knowledge-base/how-cgp-is-documented).
- **What is the knowledge base the documentation comes from?** Read
  [The CGP Knowledge Base](/docs/ai/knowledge-base).

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
