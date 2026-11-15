---
title: 'Resources: crates, tools, talks, and projects'
sidebar_label: 'Resources'
sidebar_position: 10
description: 'The crates, tools, agent skill, talks, and projects around Context-Generic Programming, each with a line on what it is for.'
---

# Resources

[Context-Generic Programming](/docs/) (CGP) enables pluggable trait implementations in Rust, and
this page lists what exists around it outside this site's documentation: the crates you add to a
project, the tools that help you work with them, a recorded talk, and the projects built with CGP.
Each entry says in a line what it is and who it is for.

## Crates

- [`cgp`](https://crates.io/crates/cgp): the main crate, with the macros, traits, and providers
  that every CGP program uses.
- [`cgp-error-anyhow`](https://crates.io/crates/cgp-error-anyhow): an error backend that makes
  `anyhow::Error` a context's error type, with providers that raise other errors into it and add
  detail to it.
- [`cgp-error-eyre`](https://crates.io/crates/cgp-error-eyre): the same backend for `eyre::Report`.
- [`cgp-error-std`](https://crates.io/crates/cgp-error-std): the same backend for a boxed standard
  error, for code that avoids a third-party error crate.
- [`cgp-serde`](https://crates.io/crates/cgp-serde): Serde's `Serialize` and `Deserialize` rebuilt
  as CGP components, so each context chooses how a value type is encoded. Its
  [Projects section](/docs/projects/cgp-serde/) walks through its examples.

## Tools

- [`cargo-cgp`](https://crates.io/crates/cargo-cgp): CGP's error toolchain. Its `check` command
  replaces `cargo check` when you diagnose a wiring failure, and its `expand` command shows the
  ordinary Rust that CGP's macros generate.
- [CGP's agent skill](/docs/ai/skills/): instructions that teach a coding assistant to read, write,
  and debug CGP, published from the
  [cgp-skills repository](https://github.com/contextgeneric/cgp-skills). It can help with the
  vocabulary, the diagnostics, and the wiring while you learn, but you still review what the
  assistant writes.

Install `cargo-cgp` with `cargo install --locked cargo-cgp`, then run `cargo cgp setup` to
provision the pinned nightly its driver needs. The tool is optional, since CGP itself compiles on
stable Rust without it, and it is tested on Linux. `cargo cgp check` leads with the root cause for
the classes it recognizes, and the tool does not yet reshape every class. The
[`cargo-cgp` section](/docs/cargo-cgp/) covers installation, both commands, reading the output, the
error codes, and troubleshooting.

## Talks

### RustLab 2025: How to stop fighting with coherence

<p>
<iframe width="560" height="315" style="max-width: 100%; height: auto; aspect-ratio: 16 / 9" src="https://www.youtube.com/embed/gXIfP-W9074?si=Q1qztb6J2PQ0b-jd" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>
</p>

The talk explains why Rust's coherence rules exist before showing how CGP works within them. Its
[full transcript](/blog/rustlab-2025-coherence) is on the blog.

## Projects

- [CGP Examples](https://github.com/contextgeneric/cgp-examples): a repository of small example
  programs, each showing one area of CGP in use, from an HTTP service to a modular interpreter. Its
  [Projects section](/docs/projects/cgp-examples/) walks through them.
- [Hypershell](https://github.com/contextgeneric/hypershell): a type-level DSL for shell scripting in
  Rust, built with CGP. Its [Projects section](/docs/projects/hypershell/) walks through its
  examples.

---

*An AI agent revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
