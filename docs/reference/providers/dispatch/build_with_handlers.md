---
sidebar_label: 'BuildWithHandlers'
sidebar_position: 10
---

# `BuildWithHandlers`

Turn a list of builder adapters into a complete record, starting from an empty builder and finalizing
the fully-populated result.

## Overview

`BuildWithHandlers<Output, Handlers>` is the entry point on the builder side. It starts from an empty
partial record, pipes that builder through a list of builder adapters so each one sets its field, and
finalizes the fully-populated result into the concrete `Output`. It runs on a **context**, the type a
capability runs against. Because the finalize step is available only when every field is present,
omitting a handler for some field is a compile error rather than a runtime failure. Like every CGP
provider, it carries no runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes the output record type and a
[`Product!`](../../macros/product.md) list of builder adapters:

```rust
use cgp::extra::dispatch::{BuildAndMerge, BuildAndSetField, BuildWithHandlers};

type Handlers = Product![
    BuildAndMerge<BuildFooBar>,
    BuildAndSetField<Symbol!("baz"), BuildBaz>,
];

let foo_bar_baz = BuildWithHandlers::<FooBarBaz, Handlers>::compute(&context, code, ());
```

`BuildWithHandlers` starts from `FooBarBaz::builder()`, runs [`BuildAndMerge`](build_and_merge.md) to
copy the `foo` and `bar` fields from a built `FooBar`, runs
[`BuildAndSetField`](build_and_set_field.md) to compute and set `baz`, and finalizes. Dropping either
handler leaves a field unset and fails to compile at the finalize step.

## When to use it

**Reach for `BuildWithHandlers` to assemble a record from a list of builder adapters**,
[`BuildAndSetField`](build_and_set_field.md) and [`BuildAndMerge`](build_and_merge.md). When the list is
plain field-producing providers instead, [`BuildAndMergeOutputs`](build_and_merge_outputs.md) wraps each
one and defers here. To take a value apart rather than assemble one, use a matcher such as
[`MatchWithHandlers`](match_with_handlers.md).

## Under the hood

`BuildWithHandlers<Output, Handlers>` carries the output type and the list in `PhantomData`:

```rust
pub struct BuildWithHandlers<Output, Handlers>(pub PhantomData<(Output, Handlers)>);

impl<Context, Code, Input, Output, Builder, Handlers, Res> Computer<Context, Code, Input>
    for BuildWithHandlers<Output, Handlers>
where
    Output: HasBuilder<Builder = Builder>,
    PipeHandlers<Handlers>: Computer<Context, Code, Builder, Output = Res>,
    Res: FinalizeBuild<Target = Output>,
{
    type Output = Output;
    // compute: PipeHandlers::compute(context, code, Output::builder()).finalize_build()
}
```

It obtains an empty builder from [`HasBuilder`](../../traits/builder/has_builder.md), threads it through the list
with [`PipeHandlers`](../handler/pipe_handlers.md) so each adapter sets its field, and calls
[`FinalizeBuild`](../../traits/builder/finalize_build.md) to recover the concrete `Output`. The original `Input`
is discarded; the output is produced from the builder. It implements `Computer`, `TryComputer`, and
`Handler`, the latter two requiring the context to have an error type. Because `finalize_build` is in
scope only for the all-present builder configuration, a missing field is caught at compile time.

## Related constructs

- [`BuildAndSetField`](build_and_set_field.md), [`BuildAndMerge`](build_and_merge.md) — the adapters
  its list is built from.
- [`BuildAndMergeOutputs`](build_and_merge_outputs.md) — the higher-level wrapper for a list of plain
  field-producing providers.
- [`PipeHandlers`](../handler/pipe_handlers.md) — the pipeline it threads the builder through.
- [`HasBuilder`](../../traits/builder/has_builder.md), [`FinalizeBuild`](../../traits/builder/finalize_build.md) — the
  traits that start and finish the build.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — building a record field by field, checked at
  compile time.

## Source

- [`providers/with_handlers/build_with_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/with_handlers/build_with_handlers.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
