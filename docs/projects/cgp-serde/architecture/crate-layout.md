---
sidebar_label: 'Crate layout'
sidebar_position: 6
description: 'Why cgp-serde is five crates split along their external dependencies, what each adds, where items are imported from, and what it builds with.'
---

# Crate layout

Why is a serialization library split into five crates? So that each external library it builds on
lives in a crate of its own, and an application compiles only the ones its wiring names.
[cgp-serde](../index.md) rebuilds Serde's `Serialize` and `Deserialize` as components of
[CGP](/docs/). This page lists its crates and what each adds, where items are imported from, and
what the workspace needs to build.

## One crate per external dependency

The core crate depends only on `cgp` and `serde`, and holds what every **context**, the type whose
wiring holds an application's choices, uses to serialize. Each other crate adds the external
libraries its own providers need, and nothing else:

| Crate | What it adds | Depends on, beyond `cgp` and `serde` |
|---|---|---|
| `cgp-serde` | the two components, the two adapter types, and the core providers | nothing |
| `cgp-serde-extra` | the hex, base64, RFC 3339, and Unix-timestamp encodings | `cgp-serde`, `hex`, `base64`, `chrono` |
| `cgp-serde-json` | JSON serializing and deserializing as operations on the context | `cgp-serde`, `serde_json` |
| `cgp-serde-alloc` | the allocation component, and the deserializer that allocates through it | `cgp-serde` |
| `cgp-serde-typed-arena` | an implementation of allocation over an arena | `cgp-serde`, `cgp-serde-alloc`, `typed-arena` |

Choosing an encoding is therefore also choosing a dependency: an application that wires hex and not
base64 compiles `hex` and not `base64`. None of the library crates names an error type either. The
JSON providers raise their errors into whatever error type the context wires, so the application
chooses it, as the [`basic`](../examples/basic.md) example does with `anyhow`.

The repository's sixth crate, `cgp-serde-tests`, holds the tests the
[examples](../examples/index.md) walk through.

## A data crate needs none of them

A crate that only defines data types depends on `cgp`, to derive the field traits the struct
providers read, and on no cgp-serde crate and not on `serde`. The encoding is chosen by the
applications that use the types; see [derive-free records](./derive-free-records.md).

## Why allocation is two crates

The allocation component and its arena implementation are separate so that the allocator is a wiring
choice rather than a dependency of the deserializer. `cgp-serde-alloc` defines what allocating
means, and the deserializer that uses it, with no external dependency. `cgp-serde-typed-arena` is
one way to provide it, over the `typed-arena` crate. A program with a different allocator implements
the component itself and depends on `cgp-serde-alloc` alone; see [context
services](./context-services.md).

## Where to import items from

Every crate sorts its items into modules by kind, so an item's kind says where to import it from:

- **`components`** — the serialization components, in `cgp-serde`.
- **`providers`** — the providers, in every crate, as in
  `cgp_serde::providers::SerializeRecordFields` or `cgp_serde_extra::providers::SerializeHex`.
- **`types`** — the adapter types, in `cgp-serde`.
- **`traits`** — components that support a provider, such as the allocation component in
  `cgp-serde-alloc` and the arena getter in `cgp-serde-typed-arena`.
- **`code`** — the marker types that select the JSON operations, in `cgp-serde-json`.
- **`impls`** — the JSON convenience method, in `cgp-serde-json`.

## What the workspace needs to build

The workspace builds on stable Rust. Its `rust-toolchain.toml` pins the version it is developed
with, and its crates declare a minimum of Rust 1.90, with the 2024 edition. Every library crate is
`no_std`, and those that need `String` or `Vec` use `alloc`, so the library runs where the standard
library does not.

## What it costs

The split keeps each dependency optional, at the price of more crates to add: an application that
uses the extra encodings, JSON, and an arena depends on five cgp-serde crates and imports from each.
The examples use two to four of them.

## Where to go next

- [How cgp-serde works](./index.md): the design these crates implement.
- [Wiring a context](../guides/wiring-a-context.md): the crates in use, in a context's table.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
