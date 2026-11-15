---
title: 'Contribute to Context-Generic Programming'
sidebar_label: 'Contribute to CGP'
sidebar_position: 11
description: 'The ways to help Context-Generic Programming, from asking a question or reporting a problem to publishing components, changing CGP itself, and sponsoring its author.'
---

# Contribute

[Context-Generic Programming](/docs/) (CGP) lets you write several implementations of a Rust trait
and choose, for each type, which one applies. This page lists the ways you can help it, from the
smallest commitment to the largest, so you can stop at the first one that fits the time you have.
Each route helps on its own, and none of them requires the ones after it.

## Join the discussion

Ask questions about CGP, and suggest what the documentation should cover next, in the project's
discussion channels: [GitHub Discussions](https://github.com/orgs/contextgeneric/discussions), the
[Discord server](https://discord.gg/Hgk3rCw6pQ), and the [r/cgp](https://www.reddit.com/r/cgp/)
subreddit. A question that a page should have answered also shows which page needs work.

## Report a problem

If a page on this site confuses you, or says something the code contradicts, report it in GitHub
Discussions or as an issue on the
[website's repository](https://github.com/contextgeneric/contextgeneric.dev/issues).

The pages are written from the CGP knowledge base, a public record of CGP written for agents. Its
checks can miss an error or an inconsistency between two documents, so a problem you find there
saves the project a review it would otherwise have to do itself. File it on the knowledge base's
[issue tracker](https://github.com/contextgeneric/cgp-knowledge-base/issues);
[Report problems in the knowledge base](/docs/ai/knowledge-base/contributing-with-an-agent#report-problems-in-the-knowledge-base)
says what to include.

## Write about CGP and share it

A blog post, a tutorial, or a talk about what you built with CGP, or how you learned it, helps other
readers even where this site already covers the topic, because a second explanation can reach a
reader the first one did not. You can also follow CGP on Bluesky at
[@contextgeneric.dev](https://bsky.app/profile/contextgeneric.dev) for news of its development, and
share what you find useful there.

## Publish reusable components

CGP aims at an ecosystem of published components and providers, in which most users write only the
wiring that selects them and little context-generic code of their own. A developer who wants
interchangeable implementations could then use them without first learning to write providers. That
ecosystem does not exist yet. If you write context-generic code, publishing your components and
providers as crates is the most direct way to help start it.

## Change CGP itself

To change the library, the [cargo-cgp](/docs/cargo-cgp) toolchain, the agent skill, or the
documentation, start from
[Contributing to CGP with an Agent](/docs/ai/knowledge-base/contributing-with-an-agent). It gives the
contribution policy, how to set up the repositories, and the rules each repository follows, whether
or not you work with a coding agent.

## Sponsor Me

If you appreciate my work and want to see CGP gain wider adoption, the best way to support it is to *sponsor* me, regardless of the amount. I have sponsorship pages on [Github Sponsor](https://github.com/sponsors/soareschen/), [Patreon](https://www.patreon.com/c/maybevoid/about), and [Ko-Fi](https://ko-fi.com/maybevoid).

## Acknowledgement

CGP is created by [Soares Chen](https://maybevoid.com/), with inspiration drawn from various programming languages and paradigms, particularly Haskell typeclasses.

The development of CGP would not have been possible without the strong support of my employer, [Informal Systems](https://informal.systems/). CGP was initially introduced and refined as part of the Hermes SDK project, which leverages CGP to build a highly modular relayer for inter-blockchain communication.

---

*An AI agent rewrote this page, apart from the sponsorship section and the acknowledgement, which are
the author's own. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
