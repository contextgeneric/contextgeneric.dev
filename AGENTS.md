# AGENTS.md: the CGP website

Follow these rules for every change to this repository. Read the knowledge base's
[website section](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/website/README.md)
before writing, from the checkout at `../cgp-knowledge-base`, or on GitHub when it is absent.

## Universal rules

These rules hold in every CGP repository. The
[knowledge base's AGENTS.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/AGENTS.md)
holds their canonical wording: read it first, and change a rule there before updating this block:

- Learn CGP and its tools from the `/cgp` skill and the knowledge base; never restate them in this
  file.
- Prefer the local sibling checkout when reading another project.
- Write a cross-project link as a GitHub URL on `main`.
- Name official release versions only, never a pre-release.
- Commit only when the user explicitly asks, on the current branch.
- Verify a claim against the source before writing it.
- Keep every view of the same truth in sync in the same change.
- Ask the user when a decision is ambiguous.
- Apply point-first writing to internal prose.

## Task guides

Each kind of task has a guide in the knowledge base's `website/` section. Read it before the task:

- **Writing or revising a page:** the page kind's guide in `writing-guides/`, with the shared set
  and the page kind's row of its
  [reading map](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/website/writing-guides/README.md#the-reading-map),
  and the page's record.
- **Adding or moving a page, or writing a blog post:** `writing-guides/README.md` and
  `information-architecture.md`.
- **Changing the theme or a page's styles:** `visual-design.md`.
- **Changing the build, the navigation, or the `example-code` crate:** `site.md` and
  [example-code/README.md](example-code/README.md).

## Public prose

Every page follows the communication strategy's claims, name, and style, and the author reads the
pages that carry the argument:

- Read the knowledge base's `communication-strategy/message.md` whenever a page makes a claim about
  what CGP is good for, satisfying its pain, strength, objection, and boundary, and read any strategy
  document whole when the page makes a claim its reading-map row does not cover.
- Name Context-Generic Programming in each page's prose and relate a short form of the tag line to
  its content, following the knowledge base's
  [naming rule](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/communication-strategy/identity.md#the-name-and-the-tag-line-on-every-page);
  the skill pages and the published blog posts are exempt.
- Never publish an invented benchmark, adoption figure, or quotation.
- Never disparage another crate, language, or community; say where a simpler tool is the better choice.
- Concede the learning curve, the verbose diagnostics, and the young ecosystem where a reader looks for
  them.
- Write every page in natural-writing's natural style by default, the front page and every page under
  `docs/` included; follow a page kind's writing guide where it names another style.
- Let agents draft the pages, and have the author read in full before publication the pages the
  knowledge base's
  [AI disclosure policy](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/communication-strategy/ai-disclosure.md#the-two-claims-that-are-easiest-to-get-wrong)
  lists; ship everything else on its guide plus a spot check, and flag a judgement call for the
  author's reading rather than deciding alone.
- Add one provenance line at the foot of each page written or substantially rewritten, linking the
  section of the AI disclosure page that matches how it was made, and record the level in the page's
  record; leave the front page without one, add none to an existing page unless the author asks, and
  add none to a scaffolded stub.

## Code

Every snippet on a page is verified against current CGP:

- Invoke the `/cgp` skill before writing CGP code, and verify every snippet against the current `cgp`
  release, the `cgp` source, or its tests.
- Prefer code already verified in the knowledge base's `examples/` directory, and the idioms its
  guides teach.
- Say which context shape a page's code is in, with the qualifiers the knowledge base's
  `vocabulary.md` defines; an introductory page may record the shape in its record instead.
- Never take current syntax from an existing blog post; read the post's record for its stale parts
  first, and use the knowledge base's `releases/` directory to date an unfamiliar construct.
- Put a page's code in the `example-code/` crate, one module per heading, except a Projects page,
  whose code is checked against the project's repository; do not mirror the knowledge base's own
  snippets there.
- Run the `example-code` tests after changing a page's code, as
  [example-code/README.md](example-code/README.md) describes.
- Fill an elided body with a comment saying so, write a rejected snippet as a compile-fail fixture,
  and prefer duplication between files over a shared helper neither page shows.
- Treat a page without an `example-code` file as unvisited, not unverified, and check generated code
  with `cargo cgp expand`, which the crate does not replace.

## Links

A published page links only public resources, and links into the knowledge base only through two
exceptions:

- Never link a published page into the knowledge base.
- Make only two hand-written exceptions: `docs/ai/disclaimer.md`, which may link the base's repository
  page and its issue tracker, and the pages under `docs/ai/knowledge-base/`, which may also link the
  four top-level entry files (`README.md`, `summary.md`, `AGENTS.md`, and `sibling-projects.md`) and
  name a section's directory in prose rather than link a document inside it.
- Let only the published skill pages under `docs/ai/skills/` cite the knowledge base by deep GitHub
  URL; nobody hand-writes them, and another page must not link there.
- Do not correct these links or extend the exception to another page.
- Link only public resources: other pages on this site, the `cgp` repository and its siblings,
  docs.rs, crates.io, and ordinary external sources.

## Legacy resources

The site presents only current resources:

- Do not link, list, or recommend the CGP Patterns book or the Hermes SDK, or direct a page to
  present either.
- Point a reader who wants a first-principles explanation or a real system at this site's Concepts and
  Projects pages instead.
- Leave the book's record, the project history, and the blog records that mention them as they are.
- Leave an acknowledgement of where CGP came from to the author, and raise a page that already
  mentions a legacy resource with the author.

## The agent skill

The skill pages are a published snapshot of `cgp-skills`, which is the only place to change them:

- Do not edit the symlinks under `docs/ai/skills/`; correct the skill in `cgp-skills`.
- Leave the `cgp-skills` submodule pointer to the author.
- Read the `../cgp-skills` checkout for what the skill says, and treat a lagging published snapshot
  as normal: it is not a defect and does not need an issue, note, or fix.

## Blog posts

A published post is a dated artifact, so a correction goes beside it rather than into it:

- Do not rewrite a published post; it is a dated artifact, edited only when its record says it was
  published unfinished.
- Annotate a misleading post, supersede it with a new one, or record the drift in its record, as the
  author decides; record the drift by default for release notes.
- Never change a published post's `slug`, and never rewrite a snippet in it.
- Correct a page under `docs/` in place, without a changelog note.

## Build

The site stays a stock Docusaurus build that fails on a broken link or anchor:

- Build and serve the site with the commands in [README.md](README.md), and check the build's exit
  status rather than its output.
- Keep `onBrokenLinks` and `onBrokenAnchors` set to `throw`; fix every reference when a page or heading
  moves.
- Take a page's position from `sidebar_position` and its directory's `_category_.json`.
- Update the announcement bar in `docusaurus.config.ts` by hand with every release announcement.
- Keep the site stock: do not add plugins, and do not add React beyond the landing page.
- Use the theme's tokens from `src/css/custom.css` in page styles, never a hex colour.
- Treat `notes/` as working material, not site content; do not edit or copy from it.

## Knowledge base

Each page has a record in the knowledge base that changes with it:

- Add or revise the page's knowledge-base record in the same change.
- State what needs updating in the knowledge base when its checkout is absent.
