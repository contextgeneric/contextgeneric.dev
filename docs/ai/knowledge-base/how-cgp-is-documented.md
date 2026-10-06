---
title: 'How CGP''s Documentation Is Written'
sidebar_label: 'How the Documentation Is Written'
sidebar_position: 2
description: 'AI agents write CGP''s documentation from a public knowledge base that is checked against the library''s source. How that differs from one-shot prompting, and where its checks stop.'
---

# How CGP's Documentation Is Written

[Context-Generic Programming (CGP)](/docs/) is a language extension for Rust, with pluggable trait
implementations at compile-time. AI agents write most of its documentation from the public
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base), a record written for
agents and checked against the library's source. This page describes that process, the problem it
answers, and where its checks stop. The [AI disclaimer](/docs/ai/disclaimer) summarizes who writes
and reviews each part of the project; this page gives the full account of the documentation process.

## The problem with one-shot prompts

An agent asked to write about CGP in a single prompt, with no written record to work from, falls
back on what it learned in training, and for CGP that is a poor source. CGP's syntax has changed
across releases, and much of what has been published about it, including many posts on this site's
blog, shows constructs the current release no longer accepts, such as `#[cgp_context]` and
`cgp_preset!`. An agent writing from training reproduces that syntax, and it fills the gaps in what
it remembers with plausible guesses about what the macros generate.

Giving the agent the source code does not solve this alone. CGP is implemented mostly as procedural
macros, so the source shows token-stream transformations rather than the traits and impls they
produce. Knowing that `#[cgp_component]` generates a consumer trait, a provider trait, and the
blanket impls that connect them requires mentally running the macro. Every agent that reads the
source has to repeat that reconstruction, and two agents can reconstruct it differently.

## A written record, kept true

The knowledge base records each of those reconstructions once, in prose, so that later agents read
the conclusion instead of deriving it again. Each document owns one subject: a construct, a
cross-cutting concept, a class of compile errors, a worked example, the internals of one macro, or a
page of this site. A document that mentions another subject links to the document that owns it
rather than explaining it a second time, so two explanations cannot drift apart.

The record is only useful while it is true, so these rules govern every document:

- **The source outranks every document.** When a document and the code disagree, the code is right
  and the document has a defect. Agents check claims about accepted syntax, generated names, and
  macro expansions against the source, the tests, and the expansion snapshots, not against memory or
  an earlier draft of the document.
- **A change carries its documentation.** A change to how a construct behaves updates the matching
  documents in the same change. A stale document counts as a bug in the change that made it stale.
- **Documents describe the present.** A document states how the code works now, without a history of
  how it got there. The version history is recorded separately, in the `releases/` section,
  including when each construct was renamed or removed, so an agent that meets old syntax can find
  what replaced it.

## One construct, five views

A single construct is described in five places, and the rules require all five to agree. Following
`#[cgp_component]` through them shows what "kept in sync" means in practice:

- **The implementation:** the parsing and code generation in
  [`cgp-macro-core`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_component).
- **The tests and snapshots:** behavioral tests in the `cgp` repository, with expansion snapshots
  that record the exact code the macro generates, such as
  [`component_macro.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/tests/cgp-tests/tests/basic_delegation/component_macro.rs).
- **The reference document:** the knowledge base's description of the accepted syntax and the
  expansion, at `cgp/reference/macros/cgp_component.md`. This site's
  [`#[cgp_component]` reference page](/docs/reference/macros/cgp_component) is written from it.
- **The implementation document:** the knowledge base's account of how the macro produces that
  expansion, its corner cases, and every test that covers it, at
  `cgp/implementation/entrypoints/cgp_component.md`.
- **The agent skill:** the condensed explanation an assistant loads, in the skill's
  [Components](/docs/ai/skills/references/components) reference.

When the macro's output changes, the snapshot changes with it, and the reference document, the
implementation document, and the skill are updated in the same change. When the views disagree, the
source decides which one is wrong.

## Written rules

The rules agents follow are written down in the knowledge base and are public. A top-level
[AGENTS.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/AGENTS.md) holds the
rules for the whole base, and each section adds its own: how a reference document is structured, how
an implementation document indexes the tests, how an error class is documented with the example that
produces it. Each code repository has an `AGENTS.md` of its own that points back to these. Agents
load the CGP skill and the relevant rules before a task and read
[summary.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/summary.md), a one-line
index of every document, to find the few documents the task needs.

Public writing follows written rules too. The `communication-strategy/` section fixes how CGP is
presented: the voice of each kind of page, the vocabulary, how costs are stated beside benefits, and
how other projects are described. The `website/` section holds a writing guide for each kind of page
on this site and a record of each page: which documents it rests on, how it was made, and what a
revision must preserve.

## Who directs the work

Agents write the documents, but they do not decide what CGP is or what gets documented. The author
designs every CGP construct, writes and revises the rules above, and chooses what the knowledge base
and this site cover. The rules tell an agent to stop and ask the person directing it when a choice
has more than one defensible answer, such as an unclear intended behavior or a document whose right
home is uncertain, rather than guessing and building on the guess.

The work also moves in separate passes rather than one prompt. A review of a document first checks
every claim against the source, and only then improves how the document reads, so polished prose
cannot hide a wrong claim. A new kind of page on this site gets a writing guide before the page is
written, stating the page's job and what must not appear on it, and the draft is checked against
that guide.

## From the record to this site

The agent skill and this site's pages are derived from the record rather than written fresh. The
skill condenses it. This site's reference pages are ported from the knowledge base's reference
documents, its Concepts pages from the concept documents, and its comparison pages from the
related-work documents, each rewritten for a person rather than an agent. Code shown on a page is
also added to a test crate in the website's repository when the page is written or revised, so a
snippet there that stops compiling fails a test.

Human review varies by page. The
[AI disclaimer](/docs/ai/disclaimer#documentation-and-reference-pages) states which pages the author
reads in full and which receive spot checks.

## Where the checks stop

Some checks are mechanical and most are not. Expansion snapshots fail a test when a macro's output
changes, the [cargo-cgp](/docs/cargo-cgp) test fixtures fail when a diagnostic changes, and the
website's test crate fails when a snippet it holds no longer compiles. Agreement between prose and
code is enforced by agents following the rules and by the author's review, and both can miss a stale
sentence.

A document can therefore lag behind the code or state something wrong, and so can a page derived
from it. The knowledge base is also in an early phase and under active development, so many of its
documents still need further review by the author and by agents, as
[its current state](index.md#its-current-state) describes. The aim is to improve it step by
step toward full accuracy, meaning agreement with the author's intent as well as with the source.

The process does not make agent-written documentation correct. It makes an error checkable against a
public record and correctable at its source, after which the correction reaches the skill and the
site. If you find an error or an inconsistency in the knowledge base, file it on its
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues). For an error in a page
on this site or in the code, report it on the repository it concerns.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
