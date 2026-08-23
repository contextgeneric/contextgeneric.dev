---
sidebar_label: 'HandleFieldValue'
sidebar_position: 6
---

# `HandleFieldValue`

Strip the variant tag from a matched payload and pass the bare value to an inner handler.

## Overview

`HandleFieldValue<Provider>` is the unwrapping adapter that sits between an extract adapter and the
handler that does the work. An extract adapter such as
[`ExtractFieldAndHandle`](extract_field_and_handle.md) delivers a `Field<Tag, Value>`, so the variant
name stays attached to the payload. `HandleFieldValue` strips the `Field` wrapper and passes the bare
`Value` to `Provider`, on a **context**, the type a capability runs against. This lets an
ordinary computer over the payload type serve as a per-variant handler. Like every CGP provider, it
carries no runtime value.

`Provider` defaults to [`UseContext`](../use_context.md). The first-argument variant
`HandleFirstFieldValue<Provider>` does the same for the `(Field<Tag, Input>, Args)` tuple, forwarding
`(Input, Args)`.

## Usage

Import it from `cgp::extra::dispatch`. It takes one optional type parameter, the inner provider, and
wraps the handler inside an extract adapter:

```rust
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue};

// ComputeArea is an ordinary computer over the payload type; HandleFieldValue
// removes the Field<Tag, _> wrapper so it receives the bare payload.
type CircleArm = ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>;
```

The convenience matcher [`MatchWithValueHandlers`](match_with_value_handlers.md) adds this wrapper
automatically, which is the one difference between it and
[`MatchWithFieldHandlers`](match_with_field_handlers.md).

## When to reach for it, and when not

**Reach for `HandleFieldValue` inside an extract adapter when the inner handler is an ordinary computer
over the payload type**, so it receives the bare value rather than a tagged `Field`. You rarely write it
directly, because [`MatchWithValueHandlers`](match_with_value_handlers.md) adds it for you. Omit it, or
use [`MatchWithFieldHandlers`](match_with_field_handlers.md), when the handler needs the variant tag.

## Under the hood

`HandleFieldValue<Provider>` carries the inner provider in `PhantomData`:

```rust
pub struct HandleFieldValue<Provider = UseContext>(pub PhantomData<Provider>);
pub struct HandleFirstFieldValue<Provider = UseContext>(pub PhantomData<Provider>);
```

It takes a `Field<Tag, Value>` input, unwraps it to the bare `Value`, and forwards to `Provider`.
`HandleFirstFieldValue` unwraps the `(Field<Tag, Input>, Args)` tuple and forwards `(Input, Args)`.

## Related constructs

- [`ExtractFieldAndHandle`](extract_field_and_handle.md) — the extract adapter that delivers the
  `Field<Tag, Value>` this strips.
- [`MatchWithValueHandlers`](match_with_value_handlers.md) — adds this wrapper automatically;
  [`MatchWithFieldHandlers`](match_with_field_handlers.md) does not.
- [`Field`](../../types/field.md) — the tagged payload it unwraps.
- [`UseContext`](../use_context.md) — the default inner provider.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — preparing a matched payload for a handler.

## Source

- [`providers/field_matchers/field_value.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_matchers/field_value.rs),
  and `first_field_value.rs` for the first-argument form.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
