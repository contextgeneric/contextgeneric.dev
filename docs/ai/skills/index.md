---
title: Agent Skills
description: The agent skills that teach a coding agent to read, write, and debug Context-Generic Programming code, and the skill pages this site publishes.
sidebar_position: 1
---

# Agent Skills

A coding agent working on a [Context-Generic Programming (CGP)](/docs/) project needs more than
the source code to understand it. CGP's macros generate the traits and impls that its wiring
connects, while the agent's training may show syntax the library no longer accepts. An agent skill
supplies that missing knowledge through Markdown instructions that the agent loads when a task
needs them. This section publishes CGP's skills from the
[cgp-skills repository](https://github.com/contextgeneric/cgp-skills), so you can read what your
agent reads.

## The `/cgp` skill

The [`/cgp` skill](/docs/ai/skills/cgp/SKILL) teaches an agent to read, write, debug, and explain
CGP code. Its main page introduces consumer and provider traits, wiring, and impl-side dependencies,
shows what the macros expand to, and recommends `cargo-cgp` for readable compiler errors. It then
directs the agent to references for the task at hand, covering topics from
[components](/docs/ai/skills/cgp/references/components) and
[wiring](/docs/ai/skills/cgp/references/wiring) to
[namespaces](/docs/ai/skills/cgp/references/namespaces). The sidebar lists all the references.

The skill supports `cgp` v0.8.0 and every later release. If your project uses a newer `cgp` release
than the skill describes, the instructions tell the agent to check for a newer version of the
skill.

These instructions help the agent work with CGP, but they do not guarantee correct code. Review
what the agent writes as you would any other change. You can also read the skill to see the
instructions the agent follows.

To install it, give your agent the repository's URL and ask it to install the `/cgp` skill, or
follow the steps for your agent in [Installing the CGP Skill](/docs/ai/skills/installation).

## Beyond the skill

The skill gives an agent dense rules and worked examples for one task at a time. That format is
not well suited to learning CGP yourself, so start with the [tutorials](/docs/tutorials/hello) and
the [Concepts](/docs/concepts) pages instead.

Your agent may also need more detail than the skill provides, such as a macro's corner cases or
how it is implemented. That detail lives in the CGP knowledge base, which provides the material
for the skill.
[Using the Knowledge Base with Your Agent](/docs/ai/knowledge-base/using-it-with-your-agent) shows
how to point your agent at it.

## What is published here

The skill pages on this site reproduce one revision of the repository without changes. Because
the repository may already have a newer version, install the skill from there as described on the
installation page. If you find a mistake in a skill, report it in the repository too. Each skill
has its own directory in this section, so more skills can be added alongside `/cgp` later.

---

*An AI agent wrote and revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
