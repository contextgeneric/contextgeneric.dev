---
title: 'Release notes for cgp, cargo-cgp, cgp-serde, and Hypershell'
sidebar_label: 'Overview'
sidebar_position: 0
description: 'What each official release of cgp, cargo-cgp, cgp-serde, and Hypershell changed, one dated page per release, with the steps to upgrade to it.'
---

# Release notes

Each page in this section lists what one official release of
[Context-Generic Programming (CGP)](/docs/), a Rust library whose macros generate traits and their
wiring, or of a project built with it, changed for its users, and how a project upgrades to it. A page is dated and kept as it was
published, so it always describes the version it names.

## The artifacts

The notes cover four artifacts, which release on their own schedules:

- **[cgp](/docs/release-notes/cgp):** the library, its facade crate, and the workspace crates
  released with it. The newest release is [v0.8.0](/docs/release-notes/cgp/v0.8.0).
- **[cargo-cgp](/docs/release-notes/cargo-cgp):** CGP's error toolchain, which rewrites CGP's
  compile errors and prints a crate's macro expansion. The newest release is [v0.1.0](/docs/release-notes/cargo-cgp/v0.1.0).
- **[cgp-serde](/docs/release-notes/cgp-serde):** Serde's `Serialize` and `Deserialize` rebuilt as
  CGP components, so each application chooses how a type is encoded. See
  [its Projects pages](/docs/projects/cgp-serde/).
- **[hypershell](/docs/release-notes/hypershell):** a type-level DSL for shell-script-like programs,
  written as Rust types and run by handlers that a context chooses. See
  [its Projects pages](/docs/projects/hypershell/).

## Versioning

`cgp` is a 0.x library, and Cargo treats the minor number as the one that may break code: a requirement such as `cgp = "0.7"` accepts every 0.7 release, and a move to the next minor number is a deliberate upgrade. Each minor release from [v0.2.0](/docs/release-notes/cgp/v0.2.0) to [v0.8.0](/docs/release-notes/cgp/v0.8.0) carries breaking changes, which its page lists under *Breaking changes* with the steps to take.

A patch release is not always compatible either. [v0.4.1](/docs/release-notes/cgp/v0.4.1) and [v0.4.2](/docs/release-notes/cgp/v0.4.2) each changed an item that earlier code used, so a project that accepts patch releases automatically can meet a break there. [v0.3.1](/docs/release-notes/cgp/v0.3.1) raised the minimum Rust version and nothing else, and [v0.6.1](/docs/release-notes/cgp/v0.6.1) changed no existing item. Read the page of a patch release before accepting it.

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
