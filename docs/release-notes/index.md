---
title: 'Release notes for cgp and cargo-cgp'
sidebar_label: 'Overview'
sidebar_position: 0
description: 'What each official release of the cgp library and the cargo-cgp tool changed, one dated page per release, with the steps to upgrade to it.'
---

# Release notes

Each page in this section lists what one official release of
[Context-Generic Programming (CGP)](/docs/), a Rust library whose macros generate traits and their
wiring, changed for its users, and how a project upgrades to it. A page is dated and kept as it was
published, so it always describes the version it names.

## The artifacts

The notes cover two artifacts, which release on their own schedules:

- **[cgp](/docs/release-notes/cgp):** the library, its facade crate, and the workspace crates
  released with it. The newest release is [v0.8.0](/docs/release-notes/cgp/v0.8.0).
- **[cargo-cgp](/docs/release-notes/cargo-cgp):** CGP's error toolchain, which rewrites CGP's
  compile errors and prints a crate's macro expansion. The newest release is [v0.1.0](/docs/release-notes/cargo-cgp/v0.1.0).

## Reading a page

A page opens with the release's date, its links, and a summary of its main changes. *Compatibility*
gives the minimum Rust version and the versions released beside it, and *Upgrading* gives the
ordered steps for a project moving from the previous release, each linking the breaking change it
carries out. The changes follow, grouped as breaking changes, deprecations, new features,
improvements, fixes, and security fixes, with an empty group left out.

A release's post on the [blog](/blog) tells the story of its main feature, and its notes page lists
every change. Each links the other.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
