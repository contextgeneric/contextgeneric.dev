---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'Projects built with CGP, with their runnable examples written as short tutorials on the patterns they use, and a table from each pattern to its example.'
---

# Projects

These pages document projects built with [CGP](/docs/), a language extension for Rust with pluggable
trait implementations at compile-time. Where the [Concepts](/docs/concepts/) pages explain each CGP
idea with the smallest code that shows it, these pages show the ideas working together in programs
that do something real, and each project's examples are written as short tutorials on the patterns
they use.

The projects are experiments rather than products. Each section says on its first page what state
its project is in, what it needs to build, and where its design stops, so that a reader weighing CGP
can judge the evidence for what it is.

## The projects

- **[Hypershell](./hypershell/index.md)** — a shell-scripting language whose programs are Rust
  types, interpreted at compile time through CGP wiring. A proof of concept that builds on nightly
  Rust, and the most complete example of a type-level language built with CGP.

## Find a pattern

Each row names a CGP design pattern and the example page that shows it in a running program, with
the Concepts page that explains the idea:

| Pattern | Shown in | Explained in |
|---|---|---|
| A program written as a type, interpreted by wiring | [`hello`](./hypershell/examples/hello.md) | [Type-level DSLs](/docs/concepts/type-level-dsls) |
| A library's wiring shipped as a namespace a context joins in one line | [`hello`](./hypershell/examples/hello.md) | [Namespaces](/docs/concepts/namespaces) |
| A context supplying the values its implementations read from its fields | [`hello_name`](./hypershell/examples/hello-name.md) | [Impl-side dependencies](/docs/concepts/impl-side-dependencies) |
| Providers composed in sequence, with each output checked against the next input | [`http_checksum_cli`](./hypershell/examples/http-checksum-cli.md) | [Handlers](/docs/concepts/handlers) |
| A provider chosen by the type of the value it receives | [`http_checksum_client`](./hypershell/examples/http-checksum-client.md) | [`delegate_components!`](/docs/reference/macros/delegate_components) |
| A language extended through a namespace that inherits the base | [`http_checksum_native`](./hypershell/examples/http-checksum-native.md) | [Namespaces](/docs/concepts/namespaces) |
| Providers bundled into an aggregate provider that routes point at | [`http_checksum_native`](./hypershell/examples/http-checksum-native.md) | [Aggregate providers](/docs/concepts/aggregate-providers) |
| Nested expressions, each resolved through the context | [`github_issues`](./hypershell/examples/github-issues.md) | [Impl-side dependencies](/docs/concepts/impl-side-dependencies) |
| One context extended with its own wiring entries, including a new error type | [`bluesky_websocket`](./hypershell/examples/bluesky-websocket.md) | [Modular error handling](/docs/concepts/modular-error-handling) |

## Where to start

- **To see CGP's ideas in a real program**, read [`hello`](./hypershell/examples/hello.md), then
  follow the examples in order.
- **To weigh whether CGP holds up past a toy**, read the [Hypershell
  overview](./hypershell/index.md) and its [limitations](./hypershell/limitations.md), which state
  the costs first.
- **To learn CGP from the beginning**, start with the [Hello World tutorial](/docs/tutorials/hello)
  instead; these pages assume it rather than teach it.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
