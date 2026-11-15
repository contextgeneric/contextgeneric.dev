---
title: 'How AI Is Used in the CGP Project'
sidebar_position: 3
sidebar_label: AI Disclaimer
description: 'How the CGP project uses AI: agents write most of the documentation and tooling, the author designs CGP and writes the core library, and how each part is checked and reviewed.'
---

# How AI Is Used in the CGP Project

AI agents do much of the work on [Context-Generic Programming (CGP)](/docs/). They write most of its
documentation and most of its error toolchain, maintain much of its test suite, and help implement
parts of the library. The author designs every CGP construct and interface, and writes the core
library almost entirely by hand. This page explains where that line falls and why, then goes
through each part of the project, from the most agent-written to the least, saying who does the
work, how it is checked, and where the checks stop.

## Why the levels differ

Two questions decide how much of a part agents write. The first is whether the work becomes part of
your compiled program. Library code does, so you have to trust it, while documentation, tools, and
tests do not: a tool runs on your project, and a test exercises the library. The second is whether
the work can be checked against something that already exists. A claim about a macro can be checked
against the macro's source and expansion snapshots, and a tool's output against its test fixtures.
Design has nothing to be checked against, because it decides what the library should do in the first
place, and passing tests cannot settle that.

Agents therefore do the most writing where the result stays out of your program and can be checked,
and the least in the core library, whose design sets the behavior everything else must follow. These
criteria decide who writes each part; they do not make any part correct.

## Documentation and reference pages

AI agents write most of CGP's reference pages, explanations, guides, and error documentation. They
write them from the [CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base), a
public repository, also written by agents, that records how CGP works and the rules agents must
follow. The pages on this site rewrite that record for people.

**The source code is authoritative.** Agents must check claims about accepted syntax, generated
names, and macro expansions against the code, the tests, or the expansion snapshots, and when a
document disagrees with the code, the document is corrected. A change to how CGP behaves must update
its documentation in the same change. Some of these checks are mechanical: expansion snapshots fail
when a macro's output changes, and the code shown on a page is compiled in a test crate when the
page is written or revised. Agreement between prose and code depends on agents following the rules
and on review, and both can miss a stale sentence. So some documents still contain errors that no
review has found yet, and the pages written from them share those errors.

The author directs this work, designing CGP, writing the rules agents follow, and choosing what gets
documented, and the rules tell an agent to ask rather than guess when a choice has more than one
reasonable answer. Each commit records AI contributions with a `Co-Authored-By` trailer naming the
model, so you can see which changes agents made. That record shows who contributed, not that every
statement is correct. [How CGP's Documentation Is Written](/docs/ai/knowledge-base/how-cgp-is-documented)
gives the full account of the process.

**Human review varies by page.** The author reads these pages in full before publication: the front
page, the Concepts explanation pages, the reference index, this page, the page describing how the
documentation is written, and every blog post. On each Comparisons page, which describes another
community's tool beside CGP, the author reads in full the two sections that judge that tool: what
each approach costs, and where the other tool is the better choice. Every other page, including the
individual construct reference pages and the rest of each comparison page, follows a writing guide
and receives spot checks, without the author reading each one line by line.

Source checks and human review catch different mistakes. Checking an expansion against the source
can reveal a technical error, while reading an explanation can reveal a misleading argument. Neither
catches every error, and the project remains responsible for correcting the ones that get through.

## Blog posts and tutorials

The author drafts blog posts and tutorials, supplying their argument, their examples, and the
judgment about what to teach. An agent then revises the prose and checks the code and claims against
the knowledge base. This reverses the arrangement for reference pages, where an agent writes the
first draft. The revision improves the prose and checks the code, but it does not make the argument
correct.

Older text on this site went through broader LLM refinement, as
[the post about this website](/blog/2026/02/21/new-website) describes. This section describes the
current process.

## Tooling and tests

Agents write most of [cargo-cgp](/docs/tooling/cargo-cgp), CGP's error toolchain, and maintain much
of CGP's test suite. Neither becomes part of your compiled program: `cargo-cgp` runs on your
project, and the tests exercise the library. Both can be checked, `cargo-cgp`'s output against its
test fixtures and the tests against the library's behavior.

Agents also extend the tests across the many syntax forms the procedural macros accept. That work is
broad and repetitive, and agents make coverage practical beyond what the author could maintain
alone. Passing tests are evidence only for the cases they cover, though, and a wrong expectation or
a missing case can still let a bug through.

`cargo-cgp` is an early release and can contain bugs. `cargo cgp check` leads with the root cause
for the classes it recognizes, and the tool does not yet reshape every class.

## The CGP library

The author designs every CGP construct and interface, and writes the core library almost entirely
by hand. This is the code your application imports and compiles, so the author takes direct
responsibility for its design and implementation.

Agents help with correctness checks, code-quality review, and the implementation of the procedural
macros, and the author directs and reviews that work. The
[cgp repository](https://github.com/contextgeneric/cgp) records it with `Co-Authored-By` trailers,
including on the macros. The line therefore falls between design and implementation: the author
decides what CGP provides and how its interfaces fit together, and agents help implement and check
some of those decisions. The library does contain AI contributions, and its checks cannot guarantee
that every implementation is correct.

## Contributions from others

Contributions from outside the project follow a deliberately light policy, and a more detailed one
will follow. It has two requirements. An AI-assisted contribution to any CGP repository must use the
knowledge base, so the agent works from the current syntax and updates the documentation its change
affects. And an AI-assisted pull request or issue must include the prompts used to verify the issue
or apply the fix, since the CGP maintainers may run similar prompts themselves rather than merge a
pull request directly. [Contributing to CGP with an
Agent](/docs/ai/knowledge-base/contributing-with-an-agent) gives the policy in full.

Finding a problem in the knowledge base is a useful contribution in its own right, because the
checks can miss one. File it on the knowledge base's
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues).

## Responsibility

The project is responsible for everything published under CGP's name, including errors in
agent-written work. As CGP's author, I would rather you told me about an error than worked around
it. Report a problem in the knowledge base on its
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues), and any other error in
[GitHub Discussions](https://github.com/orgs/contextgeneric/discussions) or on the repository it
concerns.

## How pages are marked

A page that is new or substantially rewritten carries a short note at its foot, linking the section
of this page that describes how it was made. The front page is the one exception and carries no
note. An older page gets a note only once its provenance has been established, so a page without
one may still have been written with AI assistance.

---

*An AI agent wrote and revised this page using the
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base). See
[Documentation and reference pages](#documentation-and-reference-pages).*
