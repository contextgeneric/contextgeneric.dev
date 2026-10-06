---
title: 'Using the Knowledge Base with Your Agent'
sidebar_label: 'Using It with Your Agent'
sidebar_position: 3
description: 'Direct a coding agent to the CGP knowledge base when a question about CGP needs more than the agent skill carries: a corner case, an unfamiliar error, the implementation, or a comparison.'
---

# Using the Knowledge Base with Your Agent

[Context-Generic Programming (CGP)](/docs/) is a language extension for Rust, with pluggable trait
implementations at compile-time. This page shows how to direct a coding agent to the [CGP knowledge
base](https://github.com/contextgeneric/cgp-knowledge-base) when you want a deeper answer than the
[agent skill](/docs/ai/skills) gives. The knowledge base is written for agents rather than people,
so the page is about directing your agent to it. It assumes you already use an agent and know how to
give it files or a repository to read.

## When to use it

Start with the agent skill for everyday CGP code. It teaches an agent the vocabulary, the
recommended syntax, the wiring, and how to read compile errors, and it is sized to fit in the
agent's context next to your code. Writing a component, wiring a context, or fixing a missing
dependency usually needs nothing more.

Use the knowledge base when a question needs the complete record:

- **A corner case:** an unusual syntax form, how a macro treats lifetimes, or an interaction between
  two constructs.
- **An unfamiliar error:** a compile error whose cause the skill and
  [cargo-cgp](/docs/cargo-cgp) do not make clear.
- **The implementation:** how a macro produces its expansion, for learning or for auditing the
  source.
- **The generated code:** what the macros add to your own crate, which
  [`cargo cgp expand`](/docs/cargo-cgp/expand) prints and the knowledge base explains.
- **A comparison:** how CGP relates to an idea from another language you already know.

The skill already links to knowledge-base documents for details it leaves out, so an agent with the
skill may fetch some of them on its own. The steps below make that reading deliberate.

## Give the agent access

An agent can read the knowledge base from a local clone or fetch files from GitHub. A clone is
faster for the agent and lets it search across documents:

```sh
git clone https://github.com/contextgeneric/cgp-knowledge-base.git
```

To audit CGP's source, also clone the [`cgp` repository](https://github.com/contextgeneric/cgp) into
the same parent directory. The knowledge base refers to its sibling repositories by that layout, and
[sibling-projects.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/sibling-projects.md)
lists them.

## Tell it what to read

An agent makes the best use of the knowledge base when you tell it where to start and how to treat
the rules it finds there. Include three instructions in your prompt:

1. **Load the CGP skill** so the agent knows the vocabulary the documents use. If your agent does
   not load skills, attach the skill's
   [SKILL.md](https://raw.githubusercontent.com/contextgeneric/cgp-skills/refs/heads/main/cgp/SKILL.md)
   instead, as the [skill page](/docs/ai/skills) describes.
2. **Read the README first.** The
   [README](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/README.md) explains the
   organization and has a section for agents that only read the base.
3. **Read the section that owns the question.** Each top-level directory has a `README.md` that
   catalogs its documents. Name the section yourself when you know it, so the agent does not need
   the full index: `cgp/reference/` for a construct, `cgp/errors/` for a compile error,
   `cgp/implementation/` for how a macro is built, and `related-work/` for a comparison.

The `AGENTS.md` files describe how the knowledge base's own documents are written and maintained.
They bind an agent that edits the base, not one that only reads it, and the README tells the agent
so.

Ask the agent to read selectively, because reading costs context. The index of every document,
[summary.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/summary.md), is roughly
a hundred kilobytes, or tens of thousands of tokens. Have the agent open it only when it cannot tell
which section owns the question.

## Review the answer

Review the agent's answer before relying on it. The knowledge base gives an agent a checked record
to work from, but the agent can still misread a document, and a document can lag behind the code.
The knowledge base is also in an early phase, and many of its documents still need further review,
as [its current state](index.md#its-current-state) describes.
Ask the agent to name the documents it used, so you can check each claim at its source.

When an answer disagrees with the code, the code is right. If the disagreement comes from the
document rather than the agent's reading of it, file it on the knowledge base's
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues), together with the
prompt that surfaced it, so the document can be corrected.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
