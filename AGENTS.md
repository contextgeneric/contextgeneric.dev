# AGENTS.md — the CGP website

This repository is the public website for Context-Generic Programming, published at
<https://contextgeneric.dev>. Everything in `docs/`, `blog/`, and `src/pages/` is read by Rust
developers evaluating CGP, most of whom have never heard of the project's internal documentation. That
audience governs every rule below.

**Before writing or revising any page, read the knowledge base's documentation for it.** This
repository is deliberately thin on context: it holds the published prose and nothing about where that
prose came from, how current it is, or how it should be framed. All of that lives in the
[CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base), whose
[`website/`](https://github.com/contextgeneric/cgp-knowledge-base/tree/main/website) section carries
one document per page here — one per blog post, one per tutorial series, and one covering the site's
configuration and standalone pages. Find the document for the page you are about to touch, read it,
follow its links, and only then write.

## Finding the knowledge base

Look for the checkout at `../cgp-knowledge-base` first; the CGP repositories are expected to sit side
by side under one parent directory, and a local checkout reflects work that has not been pushed. When
it is absent, read the files from
<https://github.com/contextgeneric/cgp-knowledge-base> on the `main` branch.

The sibling repositories worth knowing about are the same ones the knowledge base lists:
[`cgp`](https://github.com/contextgeneric/cgp) is the library and the ground truth for any code claim,
[`cargo-cgp`](https://github.com/contextgeneric/cargo-cgp) is the error toolchain,
[`cgp-skills`](https://github.com/contextgeneric/cgp-skills) is the source of the agent skill this
site inlines under `docs/ai/`, and [`hypershell`](https://github.com/contextgeneric/hypershell) and
[`cgp-serde`](https://github.com/contextgeneric/cgp-serde) are the projects built with CGP.

## Never link from a published page into the knowledge base

The knowledge base is internal documentation written for AI agents. It records unfinished work, known
defects, and vocabulary that assumes a loaded agent skill, and it is not written for the public. A
link from a page here into it would expose material intended for a different audience and would be
useless to any reader who does not have that repository.

A published page may link only to public resources: other pages on this site, the
[`cgp` repository](https://github.com/contextgeneric/cgp) and its siblings,
[docs.rs](https://docs.rs/cgp), [crates.io](https://crates.io/crates/cgp), the
[CGP Patterns book](https://patterns.contextgeneric.dev/), and ordinary external sources.

**One published page is a sanctioned exception, and only one.** `docs/ai/disclaimer.md` links to the
knowledge base repository, because the base is the subject it discloses: the page's argument is that
CGP's documentation is written against a public record whose rules and history a reader can go and
check, and that argument cannot be made while hiding the record. Neither reason behind the rule applies
there — the link is a public GitHub URL that resolves for anyone, and exposing the material is the
point rather than an accident. The exception covers the repository's front door only, never a path into
a particular file, since a reader following a deep link lands in prose written for agents. Do not
"correct" that page's link, and do not extend the exception to any other page.

This file is the exception, because it is not published — Docusaurus renders `docs/`, `blog/`, and
`src/pages/`, and a repository-root Markdown file is never part of the built site. So it may point at
internal material freely, and that is precisely its job.

## Verify every code snippet against the current library

Any Rust that appears on a page must compile against the current `cgp` release. Check it against the
[`cgp` source](https://github.com/contextgeneric/cgp) or its tests, and load the CGP agent skill
before writing CGP code, so that the snippet uses the idioms the library currently prefers rather than
ones that merely still parse.

**Never copy current syntax out of an existing blog post on this site.** Almost every post predates
the current release, and the drift is not cosmetic — posts from as recently as 2026 show
`#[cgp_context]`, `cgp_preset!`, `#[cgp_inherit]`, `HasCgpProvider`, the `Async` trait, `ProvideType`,
`symbol!`, and inside-out `#[cgp_provider]` implementations, none of which are current. The knowledge
base's document for each post lists exactly what is stale in it, and its
[`releases/`](https://github.com/contextgeneric/cgp-knowledge-base/tree/main/releases) section carries
a removal ledger that dates every renamed or deleted construct in one table. The safest source of
verified snippets is the knowledge base's
[`examples/`](https://github.com/contextgeneric/cgp-knowledge-base/tree/main/examples) directory,
which exists partly to be quoted.

## Consult the communication strategy before writing public prose

Everything published here is public writing about CGP, and how CGP is presented is a settled matter
rather than an open one. The knowledge base's
[`communication-strategy/`](https://github.com/contextgeneric/cgp-knowledge-base/tree/main/communication-strategy)
section fixes the audience model, the vocabulary, the framing of each claim, and the playbook for each
kind of artifact — the launch post, the tutorial, the landing page, the talk, the thread, the
comparison. Read the document that matches what you are writing before outlining it, not after
drafting it.

The rule that governs all of it: **honesty is the strategy**. Never publish an invented benchmark,
adoption number, or quotation. Never disparage another crate, language, or community to make CGP look
better; naming where a simpler tool is the right choice is an asset, not a concession. And concede the
genuine costs — the learning curve, the verbose diagnostics, the young ecosystem — wherever a reader
would look for them, because this audience is unusually good at detecting what is being left out.

## Do not rewrite published blog posts

A blog post is dated. A release announcement records what a release did at the time, and editing its
snippets into current syntax would make it claim that an old version shipped features it did not. Leave
published posts as they stand, even when their code no longer compiles.

When a post is genuinely misleading, the remedies are a short dated note at the top pointing to current
material, or a new post that supersedes it — both decisions for a human to make, not defaults to apply.
The one exception is a post that was published unfinished; the knowledge base's document for such a
post says so explicitly, and ordinary editing applies to it again.

Pages under `docs/` are the opposite case. They describe CGP as it is now, carry no date, and should be
corrected in place with no changelog note and no record of what they used to say.

## How the site is built

This is a stock [Docusaurus](https://docusaurus.io/) installation, and keeping it stock is a deliberate
decision recorded in the [new-website post](https://contextgeneric.dev/blog/new-website): no plugins,
no React beyond the landing page, so that effort goes into Markdown rather than theming. An agent
adding site machinery is working against a stated policy and should raise it rather than assume it.

Run `yarn start` for a live-reloading dev server and `yarn build` to produce the static output.
Deployment runs from the GitHub Actions workflows in `.github/`, publishing to GitHub Pages under the
domain fixed by `static/CNAME`.

Three configuration details matter when editing. `onBrokenLinks` is set to `throw`, so a dangling
internal link fails the build — a change that renames or moves a page must fix every reference to it in
the same change. The docs sidebar is autogenerated from the directory tree, so a page's position comes
from its `sidebar_position` front matter and its directory's `_category_.json`, never from a
hand-maintained list. And the **announcement bar** in `docusaurus.config.ts` is hardcoded rather than
derived from the newest post, so it must be updated by hand with every release announcement.

## Content conventions

Blog posts live at `blog/<YYYY-MM-DD>-<name>.md`, or in a directory with an `index.md` when the post
carries images. Front matter sets `authors: [soares]`, one or more tags from the fixed set in
`blog/tags.yml` (`release`, `deepdive`, `walkthrough`), and usually an explicit `slug`. A
`{/* truncate */}` marker must separate the excerpt from the body, or the build warns. Note that the
existing release slugs are inconsistent — dash-separated versions through v0.6.1, dotted from v0.7.0 —
so match a new post to its neighbours rather than assuming one rule.

Docs pages set `sidebar_position` and are grouped by directory, with each directory's label and
position in its `_category_.json`. The `docs/ai/` tree is a **published copy** of the agent skill from
[`cgp-skills`](https://github.com/contextgeneric/cgp-skills); never fix the skill by editing the copy —
correct it upstream and re-inline the result, or the two versions diverge.

The `notes/` directory is working material rather than site content: a local mirror of the
[Diátaxis](https://diataxis.fr/) documentation framework, and a research note comparing CGP's implicit
arguments to Scala's implicit parameters. Nothing in it is published, and the Diátaxis material is
third-party text kept for reference — read it for guidance on which of the four documentation kinds a
new page should be, but do not edit it and do not copy from it into a page.

## Committing

Make git commits **only when explicitly asked**, and treat each such request as authorizing exactly one
commit of the changes then in the working tree. Do not commit as a side effect of finishing a task, do
not bring commits up unprompted, and do not treat one request as turning on automatic committing.
Always commit on the current branch rather than creating or switching one.

## Update the knowledge base in the same change

A page added or substantially changed here needs its knowledge-base document added or revised in the
same change. That document is the only place a page's provenance is recorded — which internal material
it rests on, how current its code is, and what a future revision must preserve — so a page without one
has no recorded history, and a stale one actively misleads the next agent. When the knowledge-base
checkout is not present, say plainly what needs updating there so the work is not silently dropped.
