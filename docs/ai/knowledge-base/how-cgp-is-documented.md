---
title: 'How CGP''s Documentation Is Written'
sidebar_label: 'How the Documentation Is Written'
sidebar_position: 2
description: 'AI agents write CGP''s documentation from a public knowledge base that is checked against the library''s source. How that differs from one-shot prompting, and where its checks stop.'
---

# How CGP's Documentation Is Written

How much can you trust documentation written by an AI agent? Agents write most of the documentation
for [Context-Generic Programming (CGP)](/docs/), a Rust library for writing several implementations
of a trait and choosing which one each type uses. To assess that work, it helps to understand what
the agents write from and how their claims are checked.

The agents use the public [CGP knowledge
base](https://github.com/contextgeneric/cgp-knowledge-base), a record written for agents and checked
against the library's source. This page explains why that record is needed, how it is maintained,
and where the checks stop. For details of who reviews which pages, see the [AI
disclaimer](/docs/ai/disclaimer).

## The problem with one-shot prompts

When an agent is asked to write about CGP in a single prompt, without a written record to work from,
it falls back on what it learned in training. For CGP, that is a poor source: the syntax has changed
across releases, and much of what has been published about it, including many posts on this site's
blog, shows constructs the current release no longer accepts, such as `#[cgp_context]` and
`cgp_preset!`. An agent writing from training reproduces that syntax and fills the gaps in what it
remembers with plausible guesses about what the macros generate.

Giving the agent the source code does not solve the problem on its own. CGP is implemented mostly as
procedural macros, so the source shows token-stream transformations rather than the traits and impls
they produce. Knowing that `#[cgp_component]` generates a consumer trait, a provider trait, and the
blanket impls that connect them requires mentally running the macro. Every agent that reads the
source has to repeat that reconstruction, and two agents can reconstruct it differently.

## Keeping the record accurate

The knowledge base records each reconstruction in prose so that later agents can read the conclusion
without deriving it again. Each document covers one subject: a construct, a concept that applies
across constructs, a class of compile errors, a worked example, the internals of one macro, or a
page of this site. When it mentions another subject, it links to the document that explains that
subject. This leaves one explanation to keep current and avoids duplicating accounts that can drift
apart.

For that record to remain useful, it must stay accurate. These rules govern every document:

- **The source outranks every document.** When a document and the code disagree, the code is right
  and the document has a defect. Agents check claims about accepted syntax, generated names, and
  macro expansions against the source, the tests, and the expansion snapshots, not against memory or
  an earlier draft of the document.
- **Behavior and documentation change together.** When a change affects how a construct behaves,
  the matching documents must be updated in the same change. A stale document counts as a bug in
  the change that made it stale.
- **Documents describe the present.** A document states how the code works now, without a history of
  how it got there. The version history is recorded separately, in the `releases/` section,
  including when each construct was renamed or removed, so an agent that encounters old syntax can
  find what replaced it.

## Consistent views of a construct

The rules require the accounts of a construct to agree wherever it is described. For
`#[cgp_component]`, that means keeping the following sources in sync:

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
  [Components](/docs/ai/skills/cgp/references/components) reference.

When the macro's output changes, the snapshot changes with it, and the reference document, the
implementation document, and the skill are updated in the same change. When the views disagree, the
source decides which one is wrong.

## Written rules

These rules are public. The knowledge base's
[AGENTS.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/AGENTS.md) holds the
rules that apply to every document, while each section has a writing guide for its own kind of
document. Those guides explain how to structure a reference document, index tests in an
implementation document, or document an error class with an example that produces it.

Each code repository has an `AGENTS.md` of its own, which directs agents to the CGP skill and the
knowledge base to learn CGP. Before a task, agents load the skill, the rules, and the guide for the
kind of document they are working on. They also read
[summary.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/summary.md), a one-line
index of every document, to find the documents the task needs.

Public writing follows written rules too. The `communication-strategy/` section sets out how to
present CGP, including the voice of each kind of page, the vocabulary, how to state costs alongside
benefits, and how to describe other projects. The `website/` section provides a writing guide for
each kind of page on this site. It also records which documents each page draws on, how it was
written, and what a revision must preserve.

## Who directs the work

Agents write the documents, but they do not decide what CGP is or what gets documented. The author
designs every CGP construct, writes and revises the rules above, and chooses what the knowledge base
and this site cover. When a choice has more than one defensible answer, the rules tell the agent to
stop and ask the person directing it. For example, an agent must ask when the intended behavior is
unclear or it is uncertain where a document belongs, so it does not build further work on a guess.

The work proceeds in separate passes. During review, every claim in a document is checked against
the source before its prose is improved, so polished wording cannot hide a wrong claim. Before a new
kind of page is written for this site, a writing guide defines its purpose and what must not appear
on it. The draft is then checked against that guide.

## From the record to this site

The knowledge base also supplies the material for the agent skill and this site's pages. The skill
condenses that material, while the site's pages adapt it for people. Reference pages come from the
knowledge base's reference documents, Concepts pages from its concept documents, and comparison
pages from its related-work documents.

When a page is written or revised, its code examples are also added to a test crate in the website's
repository. If one of those snippets stops compiling, it fails a test.

Human review varies by page. The [AI
disclaimer](/docs/ai/disclaimer#documentation-and-reference-pages) states which pages the author
reads in full and which receive spot checks.

## Where the checks stop

Some of these checks are mechanical, but most depend on judgment. Expansion snapshots fail a test
when a macro's output changes, the [cargo-cgp](/docs/tooling/cargo-cgp) test fixtures fail when a
diagnostic changes, and the website's test crate fails when a snippet it holds no longer compiles.
Agreement between prose and code is enforced by agents following the rules and by the author's
review, and both can miss a stale sentence.

A document can therefore lag behind the code or state something wrong, and so can a page derived
from it. The knowledge base's [current state](index.md#its-current-state) explains how to assess a
document with that limitation in mind. The aim is to improve the knowledge base step by step toward
full accuracy, meaning agreement with the author's intent as well as with the source.

This process cannot guarantee that agent-written documentation is correct. It does, however, make an
error checkable against a public record and correctable at its source, after which the correction
reaches both the skill and the site. If you find an error or an inconsistency in the knowledge base,
file it on its [issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues). For an
error in a page on this site or in the code, report it on the repository it concerns.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
