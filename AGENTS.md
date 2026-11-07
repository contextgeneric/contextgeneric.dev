# AGENTS.md: the CGP website

Read the knowledge base's [website section](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/website/README.md)
for the page you are about to touch before writing. Find the checkout at `../cgp-knowledge-base`, or
read the file on GitHub when it is absent.

## Universal rules

The [knowledge base's AGENTS.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/AGENTS.md)
is the canonical wording. Read it first; change a universal rule there first, then update this
block to match. These are the actionable rules here:

- Prefer the local sibling checkout when reading another project.
- Write a cross-project link as a GitHub URL on `main`.
- Name official release versions only, never a pre-release.
- Commit only when the user explicitly asks, on the current branch.
- Verify a claim against the source before writing it.
- Keep every view of the same truth in sync in the same change.
- Ask the user when a decision is ambiguous.
- Apply point-first writing to internal prose.

## Versions

- The site describes `cgp` 0.8.0 and `cargo-cgp` 0.1.0.
- Quote command output with the official version.

## Links

- Never link a published page into the knowledge base.
- The two hand-written exceptions are `docs/ai/disclaimer.md` and the pages under
  `docs/ai/knowledge-base/`, within the bounds the knowledge base's website rules set.
- The published skill pages under `docs/ai/skills/` may cite the knowledge base by deep GitHub URL;
  nobody hand-writes them, and another page must not link there.
- Do not correct these links or extend the exception to another page.
- In the knowledge-base pages, name a section's directory in prose rather than linking a document
  inside it.
- Link only public resources: other pages on this site, the `cgp` repository and its siblings,
  docs.rs, crates.io, and ordinary external sources.

## Legacy resources

- Do not link, list, or recommend the CGP Patterns book or the Hermes SDK.
- Point a reader who wants a first-principles explanation or a real system at this site's Concepts and
  Projects pages instead.
- Leave the book's record, the project history, and the blog records that mention them as they are.
- Leave an acknowledgement of where CGP came from to the author.

## Code

- Verify every snippet against the current `cgp` release, the `cgp` source, or its tests.
- Invoke the `/cgp` skill before writing CGP code.
- Prefer code already verified in the knowledge base's `examples/` directory.
- Never take current syntax from an existing blog post; read the post's internal document for its
  stale parts first.
- Put a page's code in the website repository's `example-code/` crate, except a Projects page, whose
  code is checked against the project's repository.
- Use the knowledge base's `releases/` directory to date an unfamiliar construct.

## Public prose

- Read the communication strategy before outlining or drafting a page.
- Never publish an invented benchmark, adoption figure, or quotation.
- Never disparage another crate, language, or community; say where a simpler tool is the better choice.
- Concede the learning curve, the verbose diagnostics, and the young ecosystem where a reader looks for
  them.
- Use natural-writing for `docs/concepts/` and `docs/cargo-cgp/` pages.

## Blog posts

- Do not rewrite a published post; it is a dated artifact.
- Annotate a misleading post, supersede it with a new one, or record the drift in its internal
  document, as the author decides.
- Never change a published post's `slug`.
- Correct a page under `docs/` in place, without a changelog note.

## Build

- Run `yarn start` for the dev server and `yarn build` for the static output, or the `node_modules`
  binaries of the same names.
- Check the build's exit status rather than its output.
- Keep `onBrokenLinks` set to `throw`; fix every reference when a page moves.
- Take a page's position from `sidebar_position` and its directory's `_category_.json`.
- Update the announcement bar in `docusaurus.config.ts` by hand with every release announcement.
- Keep the site stock: do not add plugins, and do not add React beyond the landing page.

## Content conventions

- Name a blog post `blog/<YYYY-MM-DD>-<name>.md`, or a directory with an `index.md` when it carries
  images.
- Set `authors`, `tags` from the fixed set, and usually an explicit `slug`; separate the excerpt with
  `{/* truncate */}`.
- Match a new release slug to its neighbours rather than assuming one rule.
- Do not edit the symlinks under `docs/ai/skills/`; correct the skill in `cgp-skills`, and leave the
  submodule pointer to the author.
- Treat a lagging skill snapshot as normal; it is not a defect and needs no issue or fix.
- Treat `notes/` as working material, not site content; do not edit or copy from it.

## Committing

- Commit only when the user explicitly asks, one commit per request, on the current branch.
- Do not commit as a side effect, bring commits up unprompted, or treat a request as standing
  authorization.

## Knowledge base

- Add or revise the page's knowledge-base document in the same change.
- State what needs updating in the knowledge base when its checkout is absent.
