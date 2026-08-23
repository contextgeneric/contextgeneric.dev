---
sidebar_label: 'BuildAndMerge'
sidebar_position: 9
---

# `BuildAndMerge`

The builder adapter that computes a whole record's worth of fields and copies every shared field into
the builder at once.

## Overview

`BuildAndMerge<Provider>` is the bulk counterpart of [`BuildAndSetField`](build_and_set_field.md).
Instead of setting one field, it runs `Provider` over a reference to the builder to produce another
record, then copies every shared field from that result into the builder in one step. It runs on a
**context**, the type a capability runs against, and is the field-list analogue of setting a single
field. Like every CGP provider, it carries no runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes the provider that produces the record to merge, and
appears in a builder list alongside single-field adapters:

```rust
use cgp::extra::dispatch::{BuildAndMerge, BuildAndSetField, BuildWithHandlers};

type Handlers = Product![
    BuildAndMerge<BuildFooBar>,
    BuildAndSetField<Symbol!("baz"), BuildBaz>,
];
```

## When to use it

**Reach for `BuildAndMerge` when a builder step produces several fields at once**, a sub-record merged
into the result, inside a list passed to [`BuildWithHandlers`](build_with_handlers.md). For a single
field use [`BuildAndSetField`](build_and_set_field.md). When the list is plain field-producing providers
rather than builder adapters, [`BuildAndMergeOutputs`](build_and_merge_outputs.md) wraps each one in
`BuildAndMerge` for you.

## Under the hood

`BuildAndMerge<Provider>` carries the provider in `PhantomData`:

```rust
pub struct BuildAndMerge<Provider = UseContext>(pub PhantomData<Provider>);

impl<Context, Code, Builder, Provider, Output, Res> Computer<Context, Code, Builder>
    for BuildAndMerge<Provider>
where
    Provider: for<'a> Computer<Context, Code, &'a Builder, Output = Res>,
    Builder: CanBuildFrom<Res, Output = Output>,
{
    type Output = Output;
    // compute: let output = Provider::compute(context, code, &builder);
    //          builder.build_from(output)
}
```

It runs `Provider` on a reference to the builder to produce a result record, then uses
[`CanBuildFrom`](../../traits/can_build_from.md) to copy every shared field into the builder. It
implements `Computer`, `TryComputer`, and `Handler`; the fallible and async forms require the context to
have an error type.

## Related constructs

- [`BuildAndSetField`](build_and_set_field.md) — the single-field counterpart.
- [`BuildWithHandlers`](build_with_handlers.md) — the entry point that runs a list of these adapters.
- [`BuildAndMergeOutputs`](build_and_merge_outputs.md) — wraps a list of plain field-producing providers
  in `BuildAndMerge` automatically.
- [`CanBuildFrom`](../../traits/can_build_from.md) — the cast it uses to copy shared fields.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — assembling a record from independent parts.

## Source

- [`providers/field_builders/build_and_merge.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_builders/build_and_merge.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
