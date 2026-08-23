---
sidebar_label: 'BuildAndSetField'
sidebar_position: 8
---

# `BuildAndSetField`

The builder adapter that computes one field's value and sets it on a partial record.

## Overview

`BuildAndSetField<Tag, Provider>` is the single-field builder adapter. It takes a builder, a partial
record, runs `Provider` over a *reference* to that builder to compute the value for `Tag`, then sets
that field and returns the advanced builder. Because the provider sees `&Builder`, it can read fields
already set on the partial record while computing the next one. It runs on a **context**, the type a
capability runs against. Like every CGP provider, it carries no runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes a field tag and the provider that computes the field's
value, and appears as an element of a builder list passed to
[`BuildWithHandlers`](build_with_handlers.md):

```rust
use cgp::extra::dispatch::{BuildAndSetField, BuildWithHandlers};

type Handlers = Product![
    BuildAndSetField<Symbol!("width"), ComputeWidth>,
    BuildAndSetField<Symbol!("height"), ComputeHeight>,
];
```

## When to reach for it, and when not

**Reach for `BuildAndSetField` when a builder step computes one named field**, inside a list passed to
[`BuildWithHandlers`](build_with_handlers.md). To copy a whole record's worth of fields in one step, use
[`BuildAndMerge`](build_and_merge.md) instead.

## Under the hood

`BuildAndSetField<Tag, Provider>` carries the tag and provider in `PhantomData`:

```rust
pub struct BuildAndSetField<Tag, Provider = UseContext>(pub PhantomData<(Tag, Provider)>);

impl<Context, Code, Tag, Value, Provider, Output, Builder> Computer<Context, Code, Builder>
    for BuildAndSetField<Tag, Provider>
where
    Provider: for<'a> Computer<Context, Code, &'a Builder, Output = Value>,
    Builder: BuildField<Tag, Value = Value, Output = Output>,
{
    type Output = Output;
    // compute: let value = Provider::compute(context, code, &builder);
    //          builder.build_field(PhantomData::<Tag>, value)
}
```

It runs `Provider` on a reference to the builder to compute the value, then calls
[`BuildField<Tag>`](../../traits/build_field.md) to set it. It implements `Computer`, `TryComputer`, and
`Handler`; the fallible and async forms require the context to have an error type and propagate the
provider's error.

## Related constructs

- [`BuildAndMerge`](build_and_merge.md) — the bulk counterpart, copying a whole record's fields at once.
- [`BuildWithHandlers`](build_with_handlers.md) — the entry point that runs a list of these adapters and
  finalizes.
- [`BuildField`](../../traits/build_field.md) — the trait it drives to set a field.
- [`UseContext`](../use_context.md) — the default provider.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — building a record field by field.

## Source

- [`providers/field_builders/build_and_set_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_builders/build_and_set_field.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
