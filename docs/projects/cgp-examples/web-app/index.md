---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'One social-media backend wired four ways with CGP, from one trait per domain to namespace defaults, to show how wiring changes as an app grows.'
---

# web-app

`web-app` wires one small social-media backend four times, each time a step further, to show how
the wiring of a CGP application changes as it grows: from one trait per domain, to one trait per
operation grouped into bundles, to wiring grouped under paths, to defaults a namespace supplies. It
is one of the demonstration crates in [cgp-examples](../index.md), built with [CGP](/docs/), a
language extension for Rust with pluggable trait implementations at compile-time.

**Nothing in the crate runs.** Every provider body is `todo!()`, and there is no binary or test. The
crate is a study of wiring, and the compiler is what checks it, so each page's "run it" step is a
check of the crate's wiring rather than a program's output.

## The application

The backend manages users and posts, and filters their content: a username is checked by a censor,
and a post's message by a spam detector, before either is created. The same application is written
in four modules, one per stage, each self-contained, with its own components, providers, and a
context named `ProductionApp`. The context in every stage is a type that stands for the
application, holds its database handle as a field, and holds its wiring:

```rust
#[derive(HasField)]
pub struct ProductionApp {
    pub database: PostgresDb,
}
```

The providers read the database as an [implicit argument](/docs/concepts/implicit-arguments), a
parameter filled from the context's field of the same name.

## The stages

Each stage has a page, in the order they build on each other:

- [`coarse_grained`](./examples/coarse-grained.md) — one manager trait per domain, and the cost it
  hides: a dependency one method needs is carried by the whole manager.
- [`fine_grained`](./examples/fine-grained.md) — one component per operation, content checks moved
  into wrapper providers, and the providers grouped into bundles.
- [`namespace`](./examples/namespaces.md) — the components placed under paths, so a context wires
  the whole application in two lines, and a test context differs from production in one.
- [`default_impls`](./examples/default-impls.md) — a namespace that supplies seven of the nine
  providers as defaults, and the limit that comes with it.

## Status

`web-app` is a demonstration, not a library: it is unpublished, and nobody adds it as a dependency.
Its code uses current CGP idioms, so its wiring is safe to copy. Since every provider body is
`todo!()`, calling any method panics with `not yet implemented`; the value of the crate is in its
wiring, which every stage checks in full.

## Checking it

Clone the [cgp-examples repository](https://github.com/contextgeneric/cgp-examples) and check the
crate from its root:

```sh
cargo check -p cgp-example-web-app
```

It builds on stable Rust 1.90 or later. The check passes, which means every context in every stage
satisfies every component it wires. The stage pages make one change each and show what
[`cargo cgp check`](/docs/cargo-cgp/check) then reports.

## Where to go next

- [`coarse_grained`](./examples/coarse-grained.md): the first stage, and the one to read first.
- [Namespaces](/docs/concepts/namespaces): the CGP idea the later stages build to.
- [Hello World](/docs/tutorials/hello): a first CGP program, for a reader new to CGP.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
