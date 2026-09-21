---
sidebar_position: 9
---

# Resources

This page lists some resources related to CGP. More resources will be added as the project continues to develop.

## Crates

- [`cgp`](https://crates.io/crates/cgp) - The main Rust crate that provides the core constructs for programming using CGP.
- [`cgp-error-anyhow`](https://crates.io/crates/cgp-error-anyhow) - A CGP crate for handling modular errors using `anyhow`.
- [`cgp-error-eyre`](https://crates.io/crates/cgp-error-eyre) - The same, using `eyre`.
- [`cgp-error-std`](https://crates.io/crates/cgp-error-std) - The same, using only the standard library.
- [`cgp-serde`](https://crates.io/crates/cgp-serde) - Modular serialization library for Serde.

## Tooling

- [`cargo-cgp`](https://crates.io/crates/cargo-cgp) - CGP's error toolchain. Its `check` command replaces `cargo check` when you are diagnosing a wiring failure, and its `expand` command shows the ordinary Rust that CGP's macros generate.

Install it with `cargo install cargo-cgp`, then run `cargo cgp setup` to provision the pinned nightly its driver needs. CGP itself compiles on stable Rust without it; the tool is optional.

`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class. The [`cargo-cgp` section](/docs/cargo-cgp/) covers installation, both commands, and troubleshooting.

## Tutorials

- [Hello World Tutorial](/docs/tutorials/hello)
- [Area Calculation Tutorial](/docs/tutorials/area-calculation/)

## Videos

### RustLab 2025 Presentation

<p>
<iframe width="560" height="315" src="https://www.youtube.com/embed/gXIfP-W9074?si=Q1qztb6J2PQ0b-jd" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>
</p>

[Read the full transcript here](/blog/rustlab-2025-coherence)

## Books

- [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/) - Builds CGP's constructs from first principles, for readers who want to understand what happens behind the macros. Its early chapters explain the ideas in ordinary Rust and remain accurate; its later chapters were written against an earlier release and show syntax that has since changed.

## CGP in production

[Hermes SDK](https://github.com/informalsystems/hermes-sdk/) is the system CGP was built for and the
largest real-world use of it. It is an inter-blockchain relayer developed at
[Informal Systems](https://informal.systems/), and its need to support many chains and configurations
from one codebase is the problem the paradigm grew out of. If you are evaluating whether CGP holds up
beyond small examples, this is the codebase to read.

## Projects

- [CGP Examples](https://github.com/contextgeneric/cgp-examples) - A repository hosting various examples of using CGP.
- [Hypershell](https://github.com/contextgeneric/hypershell) - A type-level DSL for shell-scripting in Rust, built with CGP.
