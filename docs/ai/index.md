---
title: 'AI-Assisted Development'
sidebar_label: 'Overview'
sidebar_position: 0
description: 'How a coding agent can help you work with CGP through its agent skill and knowledge base, and how the CGP project itself uses AI to write its documentation, tools, and tests.'
---

# AI-Assisted Development

This section is about coding agents and [Context-Generic Programming (CGP)](/docs/), a Rust library
that lets you write several implementations of a trait and choose which one each type uses. It
answers two separate questions. One is how an agent can help with your own CGP code, for which the
agent has to know CGP better than its training does. The other is how much of the documentation you
are reading an agent wrote, and how that work is checked. The two halves of this page keep them
apart.

## Working with CGP through an agent

Much of what has been published about CGP shows syntax the library no longer accepts, and an agent
reading your code cannot see what CGP's macros generate from it. CGP publishes two things that fill
those gaps. The agent skill teaches an agent CGP's vocabulary, its current syntax, and how to read
its compile errors, and the knowledge base holds the complete record the skill is written from.
Together they can make learning CGP, writing its wiring, and reading its diagnostics less work. Your
agent can still write wrong code, though, so review what it produces as you would any other change.

Choose the page for what you want to do:

- **Have your agent read and write CGP code.** Start with [CGP's agent skill](/docs/ai/skills),
  then follow [Installing the CGP Skill](/docs/ai/skills/installation) for the agent you use.
- **Get an answer the skill does not carry,** such as a macro's corner case or how it is
  implemented. [Using the Knowledge Base with Your Agent](/docs/ai/knowledge-base/using-it-with-your-agent)
  shows how to point your agent at the full record.
- **Change CGP itself with an agent's help.** [Contributing to CGP with an
  Agent](/docs/ai/knowledge-base/contributing-with-an-agent) gives the contribution policy, the
  setup, and the rules your agent follows.

The skill and the knowledge base are both written for agents rather than people, so to learn CGP
yourself, start with the [tutorials](/docs/tutorials/hello) and the [Concepts](/docs/concepts) pages
instead.

## How the CGP project uses AI

AI agents write most of CGP's documentation and most of `cargo-cgp`, maintain much of its test
suite, and help implement parts of the library. The author designs every CGP construct and writes
the core library almost entirely by hand. Three pages explain this arrangement and how it is
checked:

- **Which parts of the project do agents write, and who reviews them?** Read [How AI Is Used in the
  CGP Project](/docs/ai/disclaimer). Pages written with an agent's help link to it from a note at
  their foot.
- **How is agent-written documentation kept accurate?** Read [How CGP's Documentation Is
  Written](/docs/ai/knowledge-base/how-cgp-is-documented).
- **What is the record that documentation comes from?** Read
  [The CGP Knowledge Base](/docs/ai/knowledge-base).

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
