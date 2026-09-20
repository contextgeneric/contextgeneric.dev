---
sidebar_position: 2
sidebar_label: AI Disclaimer
---

# How AI Is Used in the CGP Project

CGP uses AI agents to write documentation, develop tools, maintain tests, and assist with library
implementation. The author designs every CGP construct and interface and writes the core library
almost entirely by hand. This page explains who does what, how the work is checked, and where those
checks have limits.

## Why the levels differ

CGP gives agents more writing responsibility when the result can be checked against existing code
and does not become part of your compiled program. Documentation, tools, and tests fit that
arrangement. The core library requires more direct authorship because applications depend on its
code and its design establishes the behavior that other work must follow.

Verification also differs by artifact. A claim about a macro can be checked against its source and
expansion snapshots. A tool's output can be compared with test fixtures. Design requires the author's
judgment about which behavior the library should provide; passing tests alone cannot settle it.

## Documentation and reference pages

AI agents write most of CGP's reference pages, explanations, guides, and error documentation from the
public [CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base). That repository
records the material behind the website and the rules agents must follow. Readers can inspect both
the documentation and the process used to produce it.

**The source code is authoritative.** Agents must check claims about accepted syntax, generated names,
and macro expansions against the code, tests, or expansion snapshots. When documentation and code
disagree, the documentation needs correction. The rules also require documentation updates alongside
behavior changes and direct corrections to pages that describe current behavior.

The public history records AI contributions through `Co-Authored-By` commit trailers naming the
model. Together with the writing rules, that history lets readers inspect how the documentation was
produced. It provides a record of contributions, not a guarantee that every statement is correct.

**Human review varies by page.** The author sets the writing rules and priorities and reads the front
page, Concepts explanation pages, reference index, this disclosure, and every blog post in full
before publication. On each Comparisons page, which describes another community's tool beside CGP,
the author reads in full the two sections that judge that tool: what each approach costs, and where
the other tool is the better choice. Individual construct reference pages, and the remaining sections
of a comparison page, follow a writing guide and receive spot checks; the author does not read each
one line by line.

Source verification and human review serve different purposes. Checking a macro expansion against
the source can reveal a technical mistake, while reading an explanation can reveal a misleading
argument. Both can miss errors, and the project remains responsible for correcting them.

## Blog posts and tutorials

The author drafts blog posts and tutorials, then an agent revises the prose and checks the code and
claims against the knowledge base. The author supplies the argument, examples, and judgment about
what to teach. This differs from reference pages, where agents write the initial text.

Revision does not guarantee that an argument is correct. This describes the current writing process;
older site text received broader LLM refinement, as
[the post about this website](/blog/2026/02/21/new-website) describes.

## Tooling and tests

Agents write most of [cargo-cgp](https://github.com/contextgeneric/cargo-cgp) and maintain much of
CGP's test suite. These tools run on a project rather than becoming library dependencies compiled
into its application. Their output can be checked against fixtures and the library's behavior.

Agents also help expand test coverage across the syntax forms the procedural macros accept. That
work is extensive and repetitive, so assistance makes coverage practical beyond what the author
could maintain alone. Passing tests provide evidence for the cases they cover; an incorrect
expectation or a missing case can still leave a bug undetected.

`cargo-cgp` is an early pre-release and can contain bugs. It rewrites the error classes it recognizes
and passes other diagnostics through in the compiler's wording.

## The CGP library

The author designs every CGP construct and interface, and writes the core library almost entirely
by hand. This is the code applications import and compile, so he takes direct responsibility for
its design and implementation.

AI assists with correctness checks, code-quality review, and procedural-macro implementation. The
author directs and reviews that implementation work. The
[cgp repository](https://github.com/contextgeneric/cgp) records shared work through
`Co-Authored-By` trailers, including contributions to the macros.

The distinction is between design ownership and implementation assistance. The author decides what
CGP provides and how its interfaces fit together; agents help implement and check some of those
decisions. The library is not entirely free of AI contributions, and its checks cannot guarantee
that every implementation is correct.

## Responsibility

The project is responsible for everything published under CGP's name, including errors in
agent-written work. As CGP's author, I would rather you told me about an error than worked around it.

## How pages are marked

New or substantially rewritten pages carry a short note linking to the section here that describes
how they were made. Older pages receive a note only after their provenance has been established.
The absence of a note does not mean a page was written without AI assistance.

---

*An AI agent wrote and revised this page using the
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base). See
[Documentation and reference pages](#documentation-and-reference-pages).*
