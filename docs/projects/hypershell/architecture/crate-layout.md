---
sidebar_label: 'Crate layout'
sidebar_position: 4
description: 'Why Hypershell is split into a crate per backend, what each crate adds, what a program file imports, and the nightly toolchain the workspace builds with.'
---

# Crate layout

Why is a small language split into nine crates? So that each backend library lives in a crate of its
own, and a program compiles only the backends it uses. [Hypershell](../index.md) is a small
shell-scripting language built with [CGP](/docs/). This page lists the crates and what each adds,
what a program file imports, and what the workspace needs to build.

## One crate per backend

The syntax, and the interfaces its providers implement, live in one crate that depends only on CGP.
Each backend then adds one external library in a crate of its own:

| Crate | What it adds |
|---|---|
| `hypershell-components` | the syntax, the argument handling, and the control syntax such as `Pipe` |
| `hypershell-tokio-components` | commands, files, and streams, using Tokio |
| `hypershell-reqwest-components` | HTTP requests, using `reqwest` |
| `hypershell-json-components` | JSON encoding and decoding, using `serde_json` |
| `hypershell-hash-components` | checksums, using `sha2` and `hex` |
| `hypershell-tungstenite-components` | WebSockets, using `tokio-tungstenite` |
| `hypershell-macro` | the `hypershell!` macro |
| `hypershell` | the namespace that assembles the language, its error handling, the two ready-made contexts, and the prelude |
| `hypershell-examples` | the runnable examples, and a small extension used by some of them |

The split works because providers do not name the context they will run on, or its error type. Those
are chosen last, in the `hypershell` crate. So the Tokio crate builds without `reqwest`, and the
JSON crate builds without Tokio. The HTTP and WebSocket crates do depend on the Tokio crate, because
they reuse its stream types.

The `hypershell` crate does not include the checksum and WebSocket extensions. A program that uses
them adds their crates and joins a larger language, as
[`http_checksum_native`](../examples/http-checksum-native.md) and
[`bluesky_websocket`](../examples/bluesky-websocket.md) show.

## What a program imports

A program file needs one import, `hypershell::prelude::*`. It brings the syntax, the `hypershell!`
macro, the two contexts `HypershellCli` and `HypershellHttp`, the `anyhow` error type as `Error`,
and the CGP names a program uses. Two things are imported separately: `HypershellNamespace`, from
`hypershell::namespaces`, which a context of your own joins; and `ToTokioAsyncRead`, from
`hypershell_tokio_components::dsl`, the adapter an HTTP stream needs before `StreamToString`.

## What the workspace needs to build

The repository pins nightly Rust in its `rust-toolchain.toml`, and its `.cargo/config.toml` turns on
the compiler's next-generation trait solver with `-Z next-solver=globally`. The setting matters for
the largest programs, the ones that nest whole programs inside others. Checked on stable Rust
without it, the two largest examples grew the compiler to about 7 GB of memory before they were
stopped; on nightly with the new solver, `parallel_compare`, one of the two, checked in about 9
seconds and 460 MB. Why the solvers differ so much was not investigated. A crate of your own with
large programs should use the same settings.

## What it costs

The split keeps each crate's dependencies small, at the price of more crates to depend on: a program
using an extension imports from several Hypershell crates and from the library the extension wraps,
such as `sha2` for checksums. The nightly requirement is the larger cost, because it applies to
every crate that builds Hypershell programs of any size, not only to Hypershell itself.

## Where to go next

- [How Hypershell works](./index.md): the design these crates implement.
- [Writing a program](../guides/writing-a-program.md): the imports and toolchain in practice.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
