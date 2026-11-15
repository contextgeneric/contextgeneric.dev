---
title: 'Contributing to CGP with an Agent'
sidebar_label: 'Contributing with an Agent'
sidebar_position: 4
description: 'Meet CGP''s policy for AI-assisted contributions, report problems in the knowledge base, set up the repositories for a coding agent, and follow the rules each repository gives it.'
---

# Contributing to CGP with an Agent

This page is for contributors who want to change [Context-Generic Programming (CGP)](/docs/)
itself, whether the library, the [cargo-cgp](/docs/tooling/cargo-cgp) toolchain, the agent skill,
or the documentation, with a coding agent's help. It covers the policy for AI-assisted
contributions, reporting problems in the knowledge base, the local setup the repositories expect,
and the rules your agent follows while it works. It assumes you already use an agent and have
contributed to a Rust project through GitHub before.

## Contribution policy

**An AI-assisted contribution to any CGP repository must use the
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base).** If an agent writes or
changes code or documentation in your contribution, direct it to read the knowledge base, which is
written for agents, and to follow the `AGENTS.md` file of each repository it touches. The
contribution then carries the documentation and skill updates the change requires, and it uses the
vocabulary and the current syntax the knowledge base records.

The policy exists because an agent working from its training alone reproduces CGP's outdated
syntax and invents semantics the knowledge base already records.
[How CGP's Documentation Is Written](how-cgp-is-documented.md) explains the problem and the process
that answers it.

**An AI-assisted pull request or issue must include the prompts used to produce it:** the prompt
that verified the issue, or the prompt that applied the fix. The CGP maintainers may run similar
prompts themselves rather than merge a pull request directly, so the prompt can be as useful to them
as the change.

This is a deliberately light policy, and a more detailed one will follow. Until then, it asks for
nothing beyond these two requirements. You remain responsible for reviewing what your agent produces
before you submit it.

## Report problems in the knowledge base

Finding problems in the knowledge base is one of the most useful contributions, and it needs no
local setup. The checks behind the knowledge base can miss errors, so some of its documents still
contain them, as [its current state](index.md#its-current-state) describes. An error you find, or an
inconsistency between two documents or between a document and the code, saves the project a review
it would otherwise have to do itself.

File each problem on the knowledge base's
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues). Say which document is
wrong, what it says, and what the source or another document shows instead. If an agent found or
checked the problem, include the prompt you gave it, as the policy above requires.

## Set up the repositories

Clone the CGP repositories next to each other under one parent directory. Their documents and rules
refer to each other by relative paths such as `../cgp`, and some tests depend on that layout. Clone
the repositories your change touches, together with the knowledge base and the skill:

```sh
mkdir context-generic && cd context-generic
git clone https://github.com/contextgeneric/cgp.git
git clone https://github.com/contextgeneric/cgp-knowledge-base.git
git clone https://github.com/contextgeneric/cgp-skills.git
# as needed:
git clone https://github.com/contextgeneric/cargo-cgp.git
git clone https://github.com/contextgeneric/contextgeneric.dev.git
```

The layout matters in these places:

- **The skill.** The CGP skill lives in `cgp-skills`, and a change to a construct updates it there
  in the same contribution, so clone it beside `cgp`.
- **The `cargo-cgp` tests.** Its UI test suite compiles fixtures against a `cgp` checkout at
  `../cgp`. Work on its compiler driver may also consult the Rust compiler's source, which its
  `AGENTS.md` expects at `../external/rust`.
- **The rest of the family.** The full list of repositories, including the projects built with CGP,
  is in the knowledge base's
  [sibling-projects.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/sibling-projects.md).

## Follow the repository rules

Each repository has an `AGENTS.md` at its root, and it is the first file your agent should read:
[`cgp`](https://github.com/contextgeneric/cgp/blob/main/AGENTS.md),
[`cargo-cgp`](https://github.com/contextgeneric/cargo-cgp/blob/main/AGENTS.md),
[`cgp-skills`](https://github.com/contextgeneric/cgp-skills/blob/main/AGENTS.md), [the knowledge
base](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/AGENTS.md), [the
website](https://github.com/contextgeneric/contextgeneric.dev/blob/main/AGENTS.md), and the example
and project repositories
[`cgp-examples`](https://github.com/contextgeneric/cgp-examples/blob/main/AGENTS.md),
[`hypershell`](https://github.com/contextgeneric/hypershell/blob/main/AGENTS.md), and
[`cgp-serde`](https://github.com/contextgeneric/cgp-serde/blob/main/AGENTS.md). Each one gives
the rules that repository follows and names the guide to read before each kind of task, so your
agent reads the detailed rules only for the work in front of it. The code repositories' files also
point to the checks that build and test them.

One obligation runs through all of them: **a change carries its documentation in the same
contribution.** The code is the authority, and the knowledge base, the skill, and the site are
views of it that must not drift. Each kind of change carries its own updates:

- **A change to a construct** (its syntax, its expansion, its defaults, or its errors) updates the
  knowledge base's reference and implementation documents, the expansion snapshots, and the matching
  part of the agent skill.
- **A change to `cargo-cgp`'s behavior** updates the knowledge base's `cargo-cgp/` documents and the
  UI fixtures that pin the output.
- **A change to a page on this site** updates the knowledge base's record of that page.
- **A correction to a document** starts in the knowledge base and then reaches whatever was written
  from it: the matching part of the skill, and the page on this site ported from it.

A contribution that spans repositories needs a pull request in each. Link them to each other so
they can be reviewed together.

The rules also ask your agent to load two skills before it works: the CGP skill, from
[`cgp-skills`](https://github.com/contextgeneric/cgp-skills), and the `point-first-writing` skill,
which they name for any prose the agent writes and which is not part of the CGP repositories. Make
both available to your agent in the way it loads skills, as
[Installing the CGP Skill](/docs/ai/skills/installation) describes for the CGP skill.

The rules also shape how the agent works with you. It stops and asks you when a choice has more than
one defensible answer, such as an unclear intended behavior, rather than guessing. And it creates
commits only when you ask for them, so you decide when the work is ready to commit and submit.

## Use an established workflow

The `cgp` and `cargo-cgp` repositories each name a complete workflow for their most common kind of
change, written in the knowledge base, which an agent can follow from start to finish:

- **Reviewing a macro.** The `cgp` repository's [macro review
  workflow](https://github.com/contextgeneric/cgp/blob/main/AGENTS.md#macro-review-workflow) hardens
  one macro at a time: fixing bugs and corner cases, closing test gaps, checking spans and hygiene,
  and updating every view of the macro together.
- **Improving an error message.** The `cargo-cgp` repository's [error-message
  loop](https://github.com/contextgeneric/cargo-cgp/blob/main/AGENTS.md#improving-error-messages-against-real-world-cgp-code)
  reproduces a confusing diagnostic as a small test fixture before changing the tool, then confirms
  that the fixture's output improves.

## Where to ask

Ask questions about a contribution in [GitHub
Discussions](https://github.com/orgs/contextgeneric/discussions) or on
[Discord](https://discord.gg/Hgk3rCw6pQ). The author designs CGP's constructs and interfaces, so
raise a change to a construct's design there before starting it. The [Contribute](/docs/contribute)
page lists other ways to help CGP.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
