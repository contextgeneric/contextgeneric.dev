---
title: Agent Skills
description: The agent skills that teach a coding agent to read, write, and debug Context-Generic Programming code, and the skill pages this site publishes.
sidebar_position: 1
---

# Agent Skills

A coding agent working on a [Context-Generic Programming (CGP)](/docs/) project meets Rust code
whose meaning is not all in the source. CGP's macros generate the traits and impls that wiring
connects, and what the agent learned in training may show syntax the library no longer accepts. An
agent skill gives the agent that missing knowledge: it is a set of Markdown instructions that the
agent loads when a task needs them. This section publishes CGP's skills from the
[cgp-skills repository](https://github.com/contextgeneric/cgp-skills), so you can read what your
agent reads.

## The `/cgp` skill

The [`/cgp` skill](/docs/ai/skills/cgp/SKILL) teaches an agent to read, write, debug, and explain
CGP code. Its main page introduces consumer and provider traits, wiring, and impl-side dependencies,
shows what the macros expand to, and recommends `cargo-cgp` for readable compiler errors. From there
it sends the agent to the reference that owns the task, from
[components](/docs/ai/skills/cgp/references/components) and
[wiring](/docs/ai/skills/cgp/references/wiring) to
[namespaces](/docs/ai/skills/cgp/references/namespaces); the sidebar lists them all. The skill
supports `cgp` v0.8.0 and every later release. When your project's `cgp` is newer than the releases
the skill describes, it has the agent check for a newer skill.

The skill teaches the agent, but it does not make the agent's code correct. Review what the agent
writes as you would any other change, and read the skill when you want to see the instructions the
agent follows.

To install it, give your agent the repository's URL and ask it to install the `/cgp` skill, or
follow the steps for your harness on [Installing the CGP Skill](/docs/ai/skills/installation).

## Beyond the skill

The skill is written for an agent that needs dense rules and worked examples for one task at a
time, so it is a poor way to learn CGP yourself. For that, start with the
[tutorials](/docs/tutorials/hello) and the [Concepts](/docs/concepts) pages instead.

Your agent can also outgrow the skill, when a question needs a macro's corner cases or how it is
implemented. The skill is written from the CGP knowledge base, which holds that detail, and
[Using the Knowledge Base with Your Agent](/docs/ai/knowledge-base/using-it-with-your-agent) shows
how to point your agent at it.

## What is published here

The skill pages on this site show the repository at one revision, without changes, so the
repository may already hold newer ones. Install the skill from the repository, as the installation
page describes, and report a mistake in a skill there. Each skill has its own directory in this
section, so more skills can join the `/cgp` skill later.

---

*An AI agent wrote and revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
