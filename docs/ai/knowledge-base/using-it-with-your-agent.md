---
title: 'Using the Knowledge Base with Your Agent'
sidebar_label: 'Using It with Your Agent'
sidebar_position: 3
description: 'Direct a coding agent to the CGP knowledge base when a question about CGP goes beyond the agent skill, from corner cases and unfamiliar errors to implementation details and comparisons.'
---

# Using the Knowledge Base with Your Agent

[Context-Generic Programming (CGP)](/docs/) is a Rust library for writing several implementations of
a trait and choosing which one each type uses. Your agent can learn its everyday use from the
[agent skill](/docs/ai/skills), but some questions go beyond what the skill covers. For those, you
can point the agent at the public
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base), the full record the
skill is written from.

The knowledge base is written for agents, so this page explains how to direct your agent to it.
It assumes you already use a coding agent and know how to give it files or a repository to read.

## When to use it

Start with the agent skill for everyday CGP code. It teaches an agent the vocabulary, the
recommended syntax, how to wire components together, and how to read compile errors. It is also
sized to fit in the agent's context alongside your code. For tasks such as writing a component,
wiring a context, or fixing a missing dependency, the skill is usually enough.

Use the knowledge base when a question needs the complete record:

- **A corner case:** an unusual syntax form, how a macro treats lifetimes, or an interaction between
  two constructs.
- **An unfamiliar error:** a compile error whose cause the skill and
  [cargo-cgp](/docs/tooling/cargo-cgp) do not make clear.
- **The implementation:** how a macro produces its expansion, for learning or for auditing the
  source.
- **The generated code:** what the macros add to your own crate, which
  [`cargo cgp expand`](/docs/tooling/cargo-cgp/expand) prints and the knowledge base explains.
- **A comparison:** how CGP relates to an idea from another language you already know.

The skill already links to knowledge-base documents for the details it leaves out, so an agent
with the skill may fetch some of them on its own. You can also direct it to the documents that
address your question.

## Give the agent access

An agent can read the knowledge base from a local clone or fetch files from GitHub. A clone is
faster for the agent and lets it search across documents:

```sh
git clone https://github.com/contextgeneric/cgp-knowledge-base.git
```

To audit CGP's source, also clone the [`cgp` repository](https://github.com/contextgeneric/cgp) into
the same parent directory. The knowledge base uses this layout when referring to sibling
repositories, which are listed in
[sibling-projects.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/sibling-projects.md).

## Tell it what to read

An agent makes the best use of the knowledge base when you tell it where to start and how to treat
the rules it finds there. Include these instructions in your prompt:

1. **Load the CGP skill** so the agent knows the vocabulary the documents use. If your agent does
   not load skills, attach the skill's
   [SKILL.md](https://raw.githubusercontent.com/contextgeneric/cgp-skills/main/cgp/SKILL.md)
   instead, as [Installing the CGP Skill](/docs/ai/skills/installation#agents-without-skill-support)
   describes.
2. **Read the README first.** The
   [README](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/README.md) explains the
   organization and has a section for agents that only read the knowledge base.
3. **Read the section that covers the question.** Each top-level directory has a `README.md` that
   catalogs its documents. Name the section yourself when you know it, so the agent does not need
   the full index: `cgp/reference/` for a construct, `cgp/errors/` for a compile error,
   `cgp/implementation/` for how a macro is built, and `related-work/` for a comparison.

The knowledge base's `AGENTS.md` and the writing guides in its sections describe how its own
documents are written and maintained. As the README explains, those rules apply when an agent
edits the knowledge base, but an agent that only reads it does not need to follow them.

Ask the agent to read selectively, because each document takes up space in its context. The full
document index,
[summary.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/summary.md), is roughly
a hundred kilobytes, or tens of thousands of tokens. Have the agent open it only when it cannot tell
which section covers the question.

## Review the answer

Review the agent's answer before relying on it. The knowledge base gives an agent a checked record
to work from, but the agent can still misread a document. A document can also lag behind the code or
contain an error that reviewers have not found, as the knowledge base's
[current state](index.md#its-current-state) explains. Ask the agent to name the documents it used so
you can check each claim at its source.

When an answer disagrees with the code, the code is right. If the disagreement comes from the
document rather than the agent's reading of it, file it on the knowledge base's
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues), together with the
prompt that surfaced it, so the document can be corrected.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
