---
sidebar_label: 'ExtractFieldAndHandle'
sidebar_position: 5
---

# `ExtractFieldAndHandle`

The per-variant adapter that tries one variant and forwards its matched payload to an inner handler.

## Overview

`ExtractFieldAndHandle<Tag, Provider>` is the variant adapter a matcher's handler list is built from. It
tries to extract the variant named `Tag` from the input. On success it wraps the payload in a
`Field<Tag, Value>` and hands it to `Provider`, returning `Ok` of the provider's output. On failure it
returns `Err` of the remainder, the extractor with that variant ruled out. It runs on a **context**, the
type a capability runs against. The `Result<Output, Remainder>` shape it returns is exactly what the
matcher loop expects, so a [`Product!`](../../macros/product.md) of these adapters is the list a matcher
consumes. Like every CGP provider, it carries no runtime value.

`Provider` defaults to [`UseContext`](../use_context.md), so a matched payload routes back through the
context's own handler wiring unless another provider is named. The first-argument variant
`ExtractFirstFieldAndHandle<Tag, Provider>` is the same adapter for the `(Input, Args)` calling
convention: it threads the arguments into the provider call as `(Field<Tag, Value>, Args)` and returns
`Result<Output, (Remainder, Args)>`.

## Usage

Import it from `cgp::extra::dispatch`. It takes a variant tag and an optional inner provider, and
appears as an element of a matcher's list:

```rust
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};

type Match = MatchWithHandlers<Product![
    ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
    ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
]>;
```

The payload arrives at `Provider` still wrapped as a `Field<Tag, Value>`, so wrapping the inner handler
in [`HandleFieldValue`](handle_field_value.md) is what strips the tag to a bare value. The convenience
matchers [`MatchWithValueHandlers`](match_with_value_handlers.md) and
[`MatchWithFieldHandlers`](match_with_field_handlers.md) generate lists of these adapters from an enum's
fields, so you rarely write them out by hand.

## When to reach for it, and when not

**Reach for `ExtractFieldAndHandle` when you spell out a matcher's per-variant list by hand** with
[`MatchWithHandlers`](match_with_handlers.md), naming one adapter per variant. When every variant is
handled the same way, [`MatchWithValueHandlers`](match_with_value_handlers.md) generates the list of
these adapters for you and is far shorter. Use the first-argument form `ExtractFirstFieldAndHandle`
documented here when the handlers also take extra arguments, and
[`DowncastAndHandle`](downcast_and_handle.md) to match a group of variants in one step.

## Under the hood

`ExtractFieldAndHandle<Tag, Provider>` carries the tag and provider in `PhantomData`:

```rust
pub struct ExtractFieldAndHandle<Tag, Provider = UseContext>(pub PhantomData<(Tag, Provider)>);
pub struct ExtractFirstFieldAndHandle<Tag, Provider = UseContext>(pub PhantomData<(Tag, Provider)>);
```

Its `Output` is `Result<Output, Remainder>`. It calls [`ExtractField<Tag>`](../../traits/extract_field.md)
on the input; on success it forwards a `Field<Tag, Value>` to `Provider` and returns `Ok`, and on
failure it returns `Err` of the remainder, the extractor with that variant ruled out. It implements both
`Computer` and `AsyncComputer`.

## Related constructs

- [`HandleFieldValue`](handle_field_value.md) — strips the `Field` wrapper this adapter delivers, so the
  inner handler receives the bare payload.
- [`DowncastAndHandle`](downcast_and_handle.md) — the adapter that matches a group of variants at once
  instead of one.
- [`MatchWithHandlers`](match_with_handlers.md) — the matcher that consumes a list of these adapters.
- [`ExtractField`](../../traits/extract_field.md) — the trait it drives, and
  [`Field`](../../types/field.md) — the tagged payload it forwards.
- [`UseContext`](../use_context.md) — the default inner provider.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing one variant to a handler.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum pattern it serves.

## Source

- [`providers/field_matchers/extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_matchers/extract_field.rs),
  and `extract_first_field.rs` for the first-argument form.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
