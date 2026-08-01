---
sidebar_position: 2
sidebar_label: AI Disclaimer
---

# How AI Is Used in the CGP Project

CGP is built with substantial help from AI coding agents, and how much help varies a great deal from
one part of the project to another. This page says which parts, and why the line falls where it does,
so that you can decide for yourself how much weight to give each.

The short version is that **the CGP library is designed and written by hand, while most of the
documentation around it is written by agents from a public knowledge base that is checked against the
library's source.** Everything else sits somewhere between those two, and the sections below take each
part in turn, starting where AI does the most.

## Why the levels differ

Two questions decide how much of any given artifact an agent writes, and they point the same way.

**Does it become part of your program?** Code you add as a dependency and compile into your own project
is code you inherit, and you cannot easily audit it at the moment you need it to be right. Code that
merely runs *on* your project — a command-line tool, a test suite — is judged on its results, and you
can stop using it without touching anything you wrote.

**Can it be checked against something that is already true?** A sentence about what a macro generates
can be checked against the macro. A test can be checked by running it. A *design* cannot be checked
against anything, because it is the thing everything else is checked against.

So the closer something sits to the code you compile, the more of it is written by hand; and the more
cheaply it can be verified against a ground truth, the more of it an agent writes.

## Documentation and reference pages

Most of the written material about CGP — the construct reference, the explanations, the guides, and the
error catalog — is written by AI agents rather than by hand.

What makes that workable is not the agents but the arrangement around them, and that arrangement is
public, so it can be inspected rather than taken on trust. All of this material is developed first in
the [CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base), a separate repository
that holds the project's documentation along with the rules an agent must follow to write it. The
website pages are then derived from what is recorded there.

**The knowledge base answers to the source code.** Its governing rule is that the source in each
repository is the single source of truth, ranking above any document and above the agent skill built
from it. So a claim about what a macro expands to, which identifier it generates, or which syntax it
accepts is not written from an agent's recollection — it is read out of the code, out of the test
suite, or out of the expansion snapshots that pin what each macro emits. When several descriptions of
one behavior disagree, the disagreement is treated as a defect and the code wins.

Two further rules keep it from drifting. Documentation changes in the same change as the behavior it
describes, rather than in a follow-up, on the principle that a document describing behavior the library
no longer has is worse than no document at all, because the next reader will trust it. And each
document describes only how things work now, so a page is corrected in place rather than accumulating
notes about what used to be true.

**The history is public too, and it is specific.** Every commit to the knowledge base records which
model contributed to it in a `Co-Authored-By` trailer. If you want to know how a particular sentence
about `#[cgp_component]` came to exist, you can read the rule that produced it and the commit that
landed it. That is a stronger claim than any assurance this page could make, which is why the knowledge
base is public in the first place.

**What the human author does is set the rules and read what matters.** He decides what gets written and
in what order, sets the conventions the agents work under, reads in full the pages that carry the
project's argument — the front page, the explanation pages, the reference index, this page, and every
blog post — and is answerable for all of it.

**The honest limit:** that review is not uniform, and it would be easy to imply otherwise. The roughly
seventy per-construct reference pages are not each read line by line. They are a mechanical port of
documents already written and verified against the source, into a fixed template, so the risk they
carry is a wrong expansion rather than a wrong argument — and a wrong expansion is caught by checking
the source, which is what the knowledge base's rules exist to enforce. Review where judgement matters,
verification against the source everywhere: that is the actual arrangement, and it is worth knowing
precisely rather than approximately.

## Blog posts and tutorials

These work the other way round. **The author writes the draft; an agent revises it against the
knowledge base.** The argument, the judgement about what is worth saying, the concessions, and the
voice all originate with him, and the agent's job is to improve the prose and to check the code and
claims against what the knowledge base records.

The ordering is the whole difference, and it is worth stating because "AI-assisted" otherwise covers
both this and the section above. A blog post is the author's argument, improved; a reference page is an
agent's writing, governed. Those are not the same thing.

**The honest limit:** a revised draft is still the author's claim, and revision does not make a wrong
argument right. Note also that this describes how posts and tutorials are written now — the site's
older text was refined with LLM assistance more broadly, as
[the post about this website](/blog/2026/02/21/new-website) describes.

## Tooling and tests

[`cargo-cgp`](https://github.com/contextgeneric/cargo-cgp), the toolchain that makes CGP's compiler
errors readable, is mostly written by agents, and CGP's test suite is largely maintained by them.

One rule gates both: **neither becomes part of your program.** A cargo subcommand runs *on* a project
and is never a dependency of it, so you are exposed to its behavior rather than to its source — and its
behavior is pinned by a snapshot test suite that either matches or does not. A test is the same case in
a sharper form, because a test is self-verifying against the real library: a wrong test fails loudly
rather than misleading quietly.

The test suite also has a positive reason rather than merely a permissive one. Exhaustively exercising
every syntax form a procedural macro accepts is large, mechanical, high-value work that one person
writing in their own time will never finish, and the library is better tested for having an agent do
it.

**The honest limit:** `cargo-cgp` is an early pre-release, and a bug in a tool is still a bug. It
reshapes the error classes it recognizes and passes the rest through as the compiler wrote them.

## The CGP library

**The design of every CGP construct and interface is the author's, and the library is written almost
entirely by hand.** This is the code you import and compile, and it is where the ground truth runs out:
there is nothing to check a design against, because the design is what everything else is checked
against.

AI is used here too, and it would be misleading to imply otherwise. It is used to verify correctness,
to review code quality, and to work through the implementation of the procedural macros. The
[`cgp` repository's](https://github.com/contextgeneric/cgp) commit history records this in the same
`Co-Authored-By` trailers the knowledge base uses, and some of those commits touch the macros — so the
distinction that matters is not whether an agent has ever contributed a line, but which decisions are
whose.

That distinction is this: **what CGP *is*** — every construct, every interface, and every decision
about how the pieces fit together — **is the author's work**, while **how a given macro is
implemented** is work he directs, reviews, and frequently shares. The first is what you are adopting
when you adopt CGP. The second is an implementation detail, held to the same standard as any other code
in the library.

## Responsibility

None of the above transfers responsibility. Everything published under CGP's name is the project's, and
an error in it is the project's error rather than an agent's — and as CGP's author, I would rather you
told me about one than worked around it.

## How pages are marked

Pages written with AI assistance carry a short note at the foot, linking back to the section here that
describes how they were made. That note goes on pages written from now on. Older pages do not carry one
and have not been retrofitted with one, because attaching a provenance note to a page without first
establishing what is actually true of it would be a guess rather than a disclosure.

---

*This page was written by an AI agent from the [CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base) and reviewed by the author — see [Documentation and reference pages](#documentation-and-reference-pages).*
