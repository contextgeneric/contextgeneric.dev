---
title: 'AI-Assisted Development'
sidebar_label: 'Overview'
sidebar_position: 0
description: 'How a coding agent can help you work with CGP through its agent skill and knowledge base, and how the CGP project itself uses AI to write its documentation, tools, and tests.'
---

# AI-Assisted Development

If you use a coding agent to work with [Context-Generic Programming (CGP)](/docs/), a Rust library
that lets you write several implementations of a trait and choose which one each type uses, the
agent needs more information about CGP than its training provides. This section explains how to give
your agent that information. It also explains how the CGP project itself uses agents, including how
much of the documentation they write and how that work is checked.

## Working with CGP through an agent

Much of what has been published about CGP shows syntax the library no longer accepts. An agent
reading your code also cannot see what CGP's macros generate from it. To fill these gaps, CGP
provides an agent skill that teaches the agent CGP's vocabulary, its current syntax, and how to read
its compile errors. The skill draws on a knowledge base that holds the complete record behind it.

Together, the skill and knowledge base can make it easier to learn CGP, write its wiring, and read
its diagnostics. Your agent can still write wrong code, though, so review what it produces as you
would any other change.

Choose the page for what you want to do:

- **Have your agent read and write CGP code.** Start with [CGP's agent skill](/docs/ai/skills),
  then follow [Installing the CGP Skill](/docs/ai/skills/installation) for the agent you use.
- **Find details the skill does not cover,** such as a macro's corner case or how it is
  implemented. [Using the Knowledge Base with Your Agent](/docs/ai/knowledge-base/using-it-with-your-agent)
  shows how to point your agent at the full record.
- **Change CGP itself with an agent's help.** [Contributing to CGP with an
  Agent](/docs/ai/knowledge-base/contributing-with-an-agent) gives the contribution policy, the
  setup, and the rules your agent follows.

The skill and knowledge base are written for agents. If you want to learn CGP yourself, start with
the [tutorials](/docs/tutorials/hello) and the [Concepts](/docs/concepts) pages.

## How the CGP project uses AI

AI agents write most of CGP's documentation and most of `cargo-cgp`, maintain much of its test
suite, and help implement parts of the library. The author designs every CGP construct and writes
the core library almost entirely by hand. The following pages explain who does this work and how
it is checked:

- **Which parts of the project do agents write, and who reviews them?** Read [How AI Is Used in the
  CGP Project](/docs/ai/disclaimer). Pages written with an agent's help link to it from a note at
  the bottom of each page.
- **How is agent-written documentation kept accurate?** Read [How CGP's Documentation Is
  Written](/docs/ai/knowledge-base/how-cgp-is-documented).
- **What is the record that documentation comes from?** Read
  [The CGP Knowledge Base](/docs/ai/knowledge-base).

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
