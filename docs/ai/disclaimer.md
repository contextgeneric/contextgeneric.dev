---
sidebar_position: 3
sidebar_label: AI Disclaimer
description: 'How the CGP project uses AI: agents write most of the documentation and tooling, the author designs CGP and writes the core library, and how each part is checked and reviewed.'
---

# How AI Is Used in the CGP Project

[Context-Generic Programming (CGP)](/docs/) is a language extension for Rust, with pluggable trait
implementations at compile-time. The project uses AI agents to write documentation, develop tools,
maintain tests, and assist with library implementation. The author designs every CGP construct and
interface and writes the core library almost entirely by hand.

How much of the work agents do differs by part of the project, and the sections below run from the
most agent-written part to the least. Each section says who does what, how the work is checked, and
where those checks have limits.

## Why the levels differ

CGP gives agents more writing responsibility when the result can be checked against existing code
and does not become part of your compiled program. Documentation, tools, and tests fit that
arrangement. The core library requires more direct authorship because applications depend on its
code, and its design sets the behavior that all other work must follow.

Verification also differs by part. A claim about a macro can be checked against its source and
expansion snapshots, and a tool's output can be compared with test fixtures. Design is different: it
requires the author's judgment about which behavior the library should provide, and passing tests
alone cannot settle that.

## Documentation and reference pages

AI agents write most of CGP's reference pages, explanations, guides, and error documentation from
the public [CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base). That
repository is written for agents, and it records the material behind the website and the rules
agents must follow. Readers can inspect both the documentation and the process that produces it,
which [How CGP's Documentation Is Written](/docs/ai/knowledge-base/how-cgp-is-documented) describes
in full.

The knowledge base is in an early phase and under active development. Many of its documents still
need further review, by the author and by agents, and some contain errors or inconsistencies that no
review has found yet. The aim is to improve it step by step toward full accuracy, meaning agreement
with the author's intent as well as with the source, and the pages written from it inherit its
current state.

**The source code is authoritative.** Agents must check claims about accepted syntax, generated
names, and macro expansions against the code, tests, or expansion snapshots. When documentation and
code disagree, the documentation is corrected. The rules also require a change in behavior to update
its documentation in the same change, and pages that describe current behavior are corrected in
place.

The author directs this work. He designs CGP, writes the rules agents follow, and chooses what gets
documented, and the rules tell an agent to ask rather than guess when a choice has more than one
reasonable answer. The public history records AI contributions through `Co-Authored-By` commit
trailers naming the model, so readers can see which changes agents made. That history is a record of
contributions, not a guarantee that every statement is correct.

**Human review varies by page.** The author reads these pages in full before publication: the front
page, the Concepts explanation pages, the reference index, this disclosure, the page describing how
the documentation is written, and every blog post. On each Comparisons page, which describes another
community's tool beside CGP, he reads in full the two sections that judge that tool: what each
approach costs, and where the other tool is the better choice. Every other page, including the
individual construct reference pages and the remaining sections of each comparison page, follows a
writing guide and receives spot checks; the author does not read each one line by line.

Source checks and human review catch different mistakes, and neither catches all of them. Some
checks are mechanical: expansion snapshots fail when a macro's output changes, and code shown on a
page is compiled in a test crate when the page is written or revised. Agreement between prose and
code depends on agents following the rules and on review. Checking a macro expansion against the
source can reveal a technical mistake, while reading an explanation can reveal a misleading
argument. Either can miss an error, and the project remains responsible for correcting it.

## Blog posts and tutorials

The author drafts blog posts and tutorials, then an agent revises the prose and checks the code and
claims against the knowledge base. The author supplies the argument, the examples, and the judgment
about what to teach. This reverses the arrangement for reference pages, where agents write the
initial text. Revision improves the prose and checks the code; it does not guarantee that the
argument is correct.

Older site text went through broader LLM refinement than this, as
[the post about this website](/blog/2026/02/21/new-website) describes. This section describes the
current process.

## Tooling and tests

Agents write most of [cargo-cgp](/docs/cargo-cgp) and maintain much of CGP's test suite. Neither
becomes part of your compiled program: `cargo-cgp` runs on a project, and the tests exercise the
library. Their results can be checked against test fixtures and the library's behavior.

Agents also help extend test coverage across the syntax forms the procedural macros accept. That
work is extensive and repetitive, and agent assistance makes coverage practical beyond what the
author could maintain alone. Passing tests are evidence for the cases they cover, but an incorrect
expectation or a missing case can still leave a bug undetected.

`cargo-cgp` is an early release and can contain bugs. `cargo cgp check` leads with the root cause
for the classes it recognizes, and the tool does not yet reshape every class.

## The CGP library

The author designs every CGP construct and interface, and writes the core library almost entirely
by hand. This is the code applications import and compile, so he takes direct responsibility for
its design and implementation.

AI assists with correctness checks, code-quality review, and procedural-macro implementation, and
the author directs and reviews that work. The [cgp
repository](https://github.com/contextgeneric/cgp) records shared work through `Co-Authored-By`
trailers, including contributions to the macros.

The line runs between design ownership and implementation assistance. The author decides what CGP
provides and how its interfaces fit together; agents help implement and check some of those
decisions. The library therefore includes AI contributions, and its checks cannot guarantee that
every implementation is correct.

## Contributions from others

Contributions from outside the project follow their own policy, which is deliberately light; a more
detailed one will follow. An AI-assisted contribution to any CGP repository must use the knowledge
base, so the agent works from the current syntax and updates the documentation the change affects.
An AI-assisted pull request or issue must also include the prompts used to verify the issue or apply
the fix, since the CGP maintainers may run similar prompts themselves rather than merge a pull
request directly. [Contributing to CGP with an
Agent](/docs/ai/knowledge-base/contributing-with-an-agent) gives the policy in full.

Finding and reporting problems in the knowledge base is itself a useful contribution while it is
still being reviewed. File them on its
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues).

## Responsibility

The project is responsible for everything published under CGP's name, including errors in
agent-written work. As CGP's author, I would rather you told me about an error than worked around
it. Report a problem in the knowledge base on its
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues), and any other error in
[GitHub Discussions](https://github.com/orgs/contextgeneric/discussions) or on the repository it
concerns.

## How pages are marked

New or substantially rewritten pages carry a short note at their foot, linking to the section here
that describes how they were made. Older pages receive a note only after their provenance has been
established, so the absence of a note does not mean a page was written without AI assistance.

---

*An AI agent wrote and revised this page using the
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base). See
[Documentation and reference pages](#documentation-and-reference-pages).*
