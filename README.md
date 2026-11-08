[![Deploy to GitHub Pages](https://github.com/contextgeneric/contextgeneric.dev/actions/workflows/deploy.yaml/badge.svg)](https://github.com/contextgeneric/contextgeneric.dev/actions/workflows/deploy.yaml)
[![Test deployment](https://github.com/contextgeneric/contextgeneric.dev/actions/workflows/test-deploy.yaml/badge.svg)](https://github.com/contextgeneric/contextgeneric.dev/actions/workflows/test-deploy.yaml)
![License: Apache-2.0 for code](https://img.shields.io/badge/license-Apache_2.0-blue.svg)
![Content: CC BY-SA 4.0](https://img.shields.io/badge/content-CC_BY--SA_4.0-lightgrey.svg)
![Node 24+](https://img.shields.io/badge/node-24+-green.svg)

# The CGP website: contextgeneric.dev

**The public documentation site for [Context-Generic Programming (CGP)](https://contextgeneric.dev/),
published at [contextgeneric.dev](https://contextgeneric.dev/).**

The site is a stock [Docusaurus](https://docusaurus.io/) installation: the quickstart, the tutorials,
the concepts, the construct reference, the comparisons, the project pages, the `cargo-cgp` guide, the
AI section, and the blog. It is deliberately unmodified, without plugins or custom React beyond the landing
page, so a contributor's work is almost entirely writing Markdown.

## Install

```sh
npm ci
```

The repository carries `package-lock.json`, and CI installs with `npm ci`, so use `npm` rather than
another package manager. Node 24 or newer is required. CI runs the version `.nvmrc` pins, and
`nvm use` selects the same one locally.

## Local development

```sh
npm start
```

This starts a local development server and opens a browser window. Most changes are reflected live
without restarting the server.

## Build

```sh
npm run build
```

This generates the static site into `build/`, which any static host can serve. The build fails on a
broken internal link, because `onBrokenLinks` is set to `throw`, so a moved page needs every reference
fixed in the same change.

## Deployment

The site deploys from `main` through GitHub Pages, using the workflow in
[`.github/workflows/deploy.yaml`](.github/workflows/deploy.yaml): a push to `main` builds the site and
publishes it. Every other branch and every pull request runs the same build through
[`.github/workflows/test-deploy.yaml`](.github/workflows/test-deploy.yaml) without deploying, and that
workflow also runs the `example-code` snippet tests.

## Writing a page

Read the knowledge base's [website section](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/website/README.md)
for the page you are about to touch before writing. It records, for every page the site publishes,
what that page says, which knowledge-base documents own its content, and how far its code has drifted
from the current release. The [writing guides](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/website/writing-guides/README.md)
say how each kind of page should be written.

The site documents `cgp` v0.8.0 and `cargo-cgp` v0.1.0. Verify every snippet against the current
release, the `cgp` source, or its tests, and prefer code already verified in the knowledge base's
`examples/` directory. The `example-code` crate in this repository compiles the snippets the pages
show, so a page cannot drift from the library without a failing test.

## Links

A published page never links into the knowledge base, apart from the two hand-written exceptions the
knowledge base's website rules set: `docs/ai/disclaimer.md` and the pages under
`docs/ai/knowledge-base/`. The published skill pages under `docs/ai/skills/` may cite the knowledge
base by deep GitHub URL, because nobody hand-writes them.

Link only public resources: other pages on this site, the `cgp` repository and its siblings, docs.rs,
crates.io, and ordinary external sources.

## License

The repository's source code is licensed under the Apache License, Version 2.0, the same as `cgp`. The
published pages are licensed under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/), as the site's footer states.

## Related repositories

The [CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/sibling-projects.md)
lists every repository in the ecosystem and the revision of each to read.
