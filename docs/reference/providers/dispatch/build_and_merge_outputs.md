---
sidebar_label: 'BuildAndMergeOutputs'
sidebar_position: 11
---

# `BuildAndMergeOutputs`

Build a record from a list of plain field-producing providers, wrapping each in a merge step
automatically.

## Overview

`BuildAndMergeOutputs<Output, Handlers>` is a higher-level wrapper over
[`BuildWithHandlers`](build_with_handlers.md), used when the handler list is a list of plain
field-producing providers rather than builder adapters. It wraps each provider in
[`BuildAndMerge`](build_and_merge.md) for you, so a caller supplies result-producing providers and each
one is merged into the builder without the caller writing the merge step. It runs on a **context**, the
type a capability runs against. Like every CGP provider, it carries no runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes the output record type and a
[`Product!`](../../macros/product.md) list of field-producing providers:

```rust
use cgp::extra::dispatch::BuildAndMergeOutputs;

// Each provider produces some fields; BuildAndMergeOutputs merges them all into an App.
delegate_components! {
    AppBuilder {
        ComputerComponent:
            BuildAndMergeOutputs<App, Product![BuildDatabase, BuildHttpServer, BuildLogger]>,
    }
}
```

Each of the three providers produces a record's worth of fields, and `BuildAndMergeOutputs` merges them
in turn into an `App`.

## When to use it

**Reach for `BuildAndMergeOutputs` when your handlers are plain field-producing providers** rather than
builder adapters, so each is merged into the builder without you writing the merge step. When you want
to control each step, setting one field or merging explicitly, use
[`BuildWithHandlers`](build_with_handlers.md) directly with
[`BuildAndSetField`](build_and_set_field.md) and [`BuildAndMerge`](build_and_merge.md).

## Under the hood

`BuildAndMergeOutputs<Output, Handlers>` is a [`delegate_components!`](../../macros/delegate_components.md)
table that maps the whole handler family (`ComputerComponent`, `TryComputerComponent`,
`HandlerComponent`, and their `Ref` forms) to
`BuildWithHandlers<Output, Handlers::Mapped>`, where each provider in `Handlers` has first been wrapped
in [`BuildAndMerge`](build_and_merge.md). The wrapping is done by mapping the list through the
`ToBuildAndMergeHandler` [`MapType`](../../traits/map_type.md) marker, so a caller supplies plain
result-producing providers and each is merged into the builder automatically.

## Related constructs

- [`BuildWithHandlers`](build_with_handlers.md) — the entry point this wraps, for a list of builder
  adapters directly.
- [`BuildAndMerge`](build_and_merge.md) — the merge step each provider in the list is wrapped in.
- [`MapType`](../../traits/map_type.md) — the trait the list-mapping marker implements.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — assembling a record from independent
  per-subsystem outputs.

## Source

- [`providers/builders/build_and_merge_outputs.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/builders/build_and_merge_outputs.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
