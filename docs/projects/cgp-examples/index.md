---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'Five small demonstration crates built with CGP: an extensible interpreter, an application builder, a web service, a wiring study, and a greeting.'
---

# cgp-examples

`cgp-examples` is a repository of five small programs built with [CGP](/docs/), a language extension
for Rust with pluggable trait implementations at compile-time. Each crate shows one area of CGP
working in code, and between them they cover a wide range of CGP's design patterns. They share a
workspace and nothing else, so each has a section of its own.

The crates are demonstrations, not libraries. None is published, and nobody adds them as a
dependency: the way to use one is to clone the repository and run it, or read it. Their code uses
current CGP idioms throughout, so it is safe to copy for the patterns it shows.

## The crates

| Crate | What it shows | How it runs |
|---|---|---|
| [`expression`](./expression/index.md) | an arithmetic interpreter that gains operators and operations without editing old code: the extensible visitor pattern | three unit tests, and a test to add for each of the other contexts |
| [`web-app`](./web-app/index.md) | one social-media backend wired four ways, from one trait per domain to namespace defaults | a check of its wiring; every provider body is `todo!()` |
| `builder` | an application assembled from per-subsystem builders that never name the struct they build: the extensible builder pattern | compile-time checks of its builders |
| `transfer` | a balance-and-transfer HTTP service whose every domain type and operation is a component, organized with a namespace | a server binary |
| `greet` | a greeting written as a CGP function, as a component with two providers, and over an abstract type | three binaries |

The pages for `builder`, `transfer`, and `greet` are still being written. Until they are, each crate
is in its own directory of the repository, and `transfer` has a README of its own.

## Where to start

- **To see the extensible visitor pattern**, start with [`expression`](./expression/index.md), whose
  four examples build on each other.
- **To see how wiring changes as an application grows**, read the four stages of
  [`web-app`](./web-app/index.md) in order.
- **To learn CGP from the beginning**, start with the [Hello World tutorial](/docs/tutorials/hello)
  instead; these pages assume it rather than teach it.

## Running the crates

Clone the [repository](https://github.com/contextgeneric/cgp-examples) and work from its root. The
workspace builds on stable Rust 1.90 or later, with the 2024 edition, and each crate is a package of
its own, such as `cgp-example-expression`, so `cargo test -p cgp-example-expression` runs one
crate's tests. Each crate's pages give the command for its programs.

## Where to go next

- [`expression`](./expression/index.md): the extensible visitor pattern, in four contexts.
- [`web-app`](./web-app/index.md): one application's wiring at four scales.
- [Projects](../index.md): the other projects on this site, and a table from each CGP pattern to an
  example that shows it.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
