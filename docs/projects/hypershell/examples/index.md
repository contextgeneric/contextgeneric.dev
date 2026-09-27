---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'Runnable Hypershell programs, each walked through as a short tutorial on a CGP design pattern, from a one-line context to a language extension.'
---

# Hypershell examples

These pages walk through the runnable programs in the [Hypershell
repository](https://github.com/contextgeneric/hypershell), one program per page.
[Hypershell](../index.md) is a shell-scripting language whose programs are Rust types, built with
[CGP](/docs/). Each page runs a program, explains how its wiring produces what it prints, and names
the CGP pattern it shows, so the pages work as short tutorials on those patterns.

The pages are in the order they teach. Each stands alone, but a reader new to Hypershell should
start with [`hello`](./hello.md), since the later pages build on the ideas it introduces.

:::tip

### New to CGP?

These pages explain Hypershell's code without deriving CGP from first principles. The [Hello World
tutorial](/docs/tutorials/hello) introduces CGP in a few minutes, and [Type-level
DSLs](/docs/concepts/type-level-dsls) explains the technique Hypershell is built on with a much
smaller language. Neither is required reading; each page links what it relies on.

:::

## Running an example

Clone the repository and run an example from its root with `cargo run --example <name>`. The
repository selects nightly Rust and the new trait solver itself; see the [project
overview](../index.md#running-it). Most examples also need the network and some common command-line
tools, which each page lists.

## The examples

The first two programs run commands with no network access:

- [`hello`](./hello.md) — `echo hello world!`, run on a context, the type a program runs on, whose
  whole definition is one line of wiring.
- [`hello_name`](./hello-name.md) — a program that reads a runtime value from a field of its
  context.

The next three are one pipeline written three ways, each replacing more of the shell with native
stages:

- [`http_checksum_cli`](./http-checksum-cli.md) — `curl | sha256sum | cut` as three streaming
  commands.
- [`http_checksum_client`](./http-checksum-client.md) — `curl` replaced by a native HTTP request.
- [`http_checksum_native`](./http-checksum-native.md) — every stage native, from a language extended
  with checksums.

The rest show the language's other parts:

- [`nix_manual`](./nix-manual.md) — a static URL and an argument list that mixes literals and
  fields.
- [`save_webpage`](./save-webpage.md) — a streaming download written to a file.
- [`github_issues`](./github-issues.md) — a URL built from fields, a request header, and JSON
  decoded into a Rust type.
- [`rust_playground`](./rust-playground.md) — a program whose input and output are Rust values.
- [`bluesky`](./bluesky.md) — a pipeline that streams until it is stopped.
- [`bluesky_websocket`](./bluesky-websocket.md) — the same feed read by a WebSocket stage added to
  one context.

Two more programs in the repository, `parallel_compare` and `compare_and_branch`, add control syntax
that runs whole programs as operands. Their pages are still being written.

## Where to go next

- [Architecture](../architecture/index.md): the design behind every example, one idea per page.
- [Writing a program](../guides/writing-a-program.md): the steps the examples share, as a guide.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
