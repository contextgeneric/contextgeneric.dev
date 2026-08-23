---
sidebar_label: 'DowncastAndHandle'
sidebar_position: 7
---

# `DowncastAndHandle`

The adapter that matches a whole group of variants at once by narrowing to a smaller enum.

## Overview

`DowncastAndHandle<Inner, Provider>` is the matcher adapter that handles a *group* of variants in one
step rather than a single variant. Instead of extracting one field, it tries to narrow the input to a
smaller enum type `Inner`. On success it hands the whole `Inner` value to `Provider` and returns `Ok`;
on failure it returns `Err` of the remainder. It runs on a **context**, the type a capability runs
against, and lets a matcher delegate several variants to one sub-matcher in a single step. Like every
CGP provider, it carries no runtime value.

`Provider` defaults to [`UseContext`](../use_context.md).

## Usage

Import it from `cgp::extra::dispatch`. It takes the inner enum type to narrow to and an optional inner
provider, and appears as an element of a matcher's list alongside single-variant adapters:

```rust
use cgp::extra::dispatch::{DowncastAndHandle, ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};

// Handle the two "primitive" variants together via a sub-matcher, and the rest one at a time.
type Match = MatchWithHandlers<Product![
    DowncastAndHandle<PrimitiveShape, HandlePrimitive>,
    ExtractFieldAndHandle<Symbol!("Compound"), HandleFieldValue<HandleCompound>>,
]>;
```

## When to use it

**Reach for `DowncastAndHandle` when several variants share one sub-handler** and you want to route them
as a group rather than name each one. For a single variant use
[`ExtractFieldAndHandle`](extract_field_and_handle.md). Both go in a list consumed by
[`MatchWithHandlers`](match_with_handlers.md).

## Under the hood

`DowncastAndHandle<Inner, Provider>` carries the inner enum type and the provider in `PhantomData`:

```rust
pub struct DowncastAndHandle<Input, Provider = UseContext>(pub PhantomData<(Input, Provider)>);
```

Its `Output` is `Result<Output, Remainder>`. It uses
[`CanDowncastFields<Inner>`](../../traits/casting/can_downcast_fields.md) to try to narrow the input to `Inner`;
on success it hands the whole `Inner` value to `Provider` and returns `Ok`, and on failure it returns
`Err` of the remainder. It implements both `Computer` and `AsyncComputer`.

## Related constructs

- [`ExtractFieldAndHandle`](extract_field_and_handle.md) — the single-variant adapter this generalizes to
  a group.
- [`MatchWithHandlers`](match_with_handlers.md) — the matcher that consumes a list mixing both.
- [`CanDowncastFields`](../../traits/casting/can_downcast_fields.md) — the cast it uses to narrow the input.
- [`UseContext`](../use_context.md) — the default inner provider.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing a group of variants to one sub-handler.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum pattern and its casts.

## Source

- [`providers/field_matchers/extract_handle.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_matchers/extract_handle.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
