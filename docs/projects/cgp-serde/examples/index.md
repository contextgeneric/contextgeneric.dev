---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'The cgp-serde examples as short tutorials: a JSON round trip with no serialization derive, and one value encoded two ways by two applications.'
---

# cgp-serde examples

These pages walk through the examples in the [cgp-serde
repository](https://github.com/contextgeneric/cgp-serde), one example per page.
[cgp-serde](../index.md) rebuilds Serde's `Serialize` and `Deserialize` traits as components of
[CGP](/docs/), so that each application chooses how a type is encoded. Each page runs an example,
explains how its wiring produces what it prints, and names the CGP pattern it shows, so the pages
work as short tutorials on those patterns.

The pages are in the order they teach. Each stands alone, but a reader new to cgp-serde should start
with [`basic`](./basic.md), since the later pages build on the wiring it introduces.

:::tip

### New to CGP?

These pages explain cgp-serde's code without deriving CGP from first principles. The [Hello World
tutorial](/docs/tutorials/hello) introduces CGP in a few minutes, and
[Coherence](/docs/concepts/coherence) explains the Rust rule that cgp-serde works around, with a
smaller encoding example. Neither is required reading; each page links what it relies on.

:::

## Running an example

Clone the repository and run an example from its root with
`cargo run -p cgp-serde-examples --example <name>`. Each example also checks what it prints in a
test, which `cargo test -p cgp-serde-examples --example <name>` runs. The repository's toolchain
file selects the Rust version, and no example needs a network or any program outside the build.

## The examples

- [`basic`](./basic.md) — one struct written to JSON and read back by a context, the type that holds
  an application's choices, with the struct's bytes as hex and a missing field reported as an error.
- [`messages`](./messages.md) — one nested archive encoded by two contexts that differ in three
  wiring lines, with hex bytes and RFC 3339 dates from one and base64 bytes and Unix timestamps from
  the other.

Two more tests, `arena_simplified` and `arena`, deserialize borrowed values into an arena that the
context supplies. Their pages are still being written; [context
services](../architecture/context-services.md) explains the design they show.

## Where to go next

- [Architecture](../architecture/index.md): the design behind every example, one idea per page.
- [Wiring a context](../guides/wiring-a-context.md): the steps the examples share, as a guide.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
