---
sidebar_label: 'MatchFirstWithHandlers'
sidebar_position: 2
---

# `MatchFirstWithHandlers`

Match an enum for the multi-argument calling convention, where the input carries extra arguments
alongside the value.

## Overview

`MatchFirstWithHandlers<Handlers>` is the matcher for the calling convention where the input is a tuple
`(Input, Args)`: the value being matched, together with extra arguments to pass to each handler. It
threads `Args` through the loop unchanged, so on a miss both the shrinking remainder and the still-owned
arguments carry to the next handler. It runs on a **context**, the type a capability runs against. The
"first" in the name reflects that the value being matched is the *first* element of the input tuple.
Like every CGP provider, it carries no runtime value; the handler list rides in `PhantomData`.

Two borrowed forms exist: `MatchFirstWithHandlersRef<Handlers>` over `(&Input, Args)` and
`MatchFirstWithHandlersMut<Handlers>` over `(&mut Input, Args)`. Each of the three implements both
`Computer` and `AsyncComputer`.

## Usage

Import it from `cgp::extra::dispatch`. It takes one type parameter, a
[`Product!`](../../macros/product.md) list of per-variant adapters, built from
[`ExtractFirstFieldAndHandle`](extract_field_and_handle.md) and
[`HandleFirstFieldValue`](handle_field_value.md) rather than their plain forms:

```rust
use cgp::extra::dispatch::{
    ExtractFirstFieldAndHandle, HandleFirstFieldValue, MatchFirstWithHandlers,
};

// Each handler receives the matched payload together with the shared Args.
type Match = MatchFirstWithHandlers<Product![
    ExtractFirstFieldAndHandle<Symbol!("Circle"), HandleFirstFieldValue<RenderShape>>,
    ExtractFirstFieldAndHandle<Symbol!("Rectangle"), HandleFirstFieldValue<RenderShape>>,
]>;
```

For a list built automatically, use [`MatchFirstWithValueHandlers`](match_with_value_handlers.md).

## When to reach for it, and when not

**Reach for `MatchFirstWithHandlers` when the per-variant handlers need extra arguments passed alongside
the matched value**, such as a renderer that takes a target buffer or a visitor that takes an
accumulator. When the handlers need only the value, use the plain
[`MatchWithHandlers`](match_with_handlers.md), which is simpler.

## Under the hood

`MatchFirstWithHandlers<Handlers>` carries the list in `PhantomData`:

```rust
pub struct MatchFirstWithHandlers<Handlers>(pub PhantomData<Handlers>);
```

Its `Computer` impl runs the shared matcher loop over `(Input::Extractor, Args)` and returns
`Result<Output, (Remainder, Args)>`, so a miss carries both the shrunken remainder and the still-owned
arguments to the next handler. When the loop finishes with an `Err`, the remainder is uninhabited and is
discharged through `finalize_extract`:

```rust
impl<Context, Code, Input, Args, Output, Remainder, Handlers>
    Computer<Context, Code, (Input, Args)> for MatchFirstWithHandlers<Handlers>
where
    Input: HasExtractor,
    DispatchMatchers<Handlers>: Computer<
        Context, Code, (Input::Extractor, Args),
        Output = Result<Output, (Remainder, Args)>>,
    Remainder: FinalizeExtract,
{
    type Output = Output;
    // compute: match DispatchMatchers::compute(context, code, (input.to_extractor(), args)) {
    //     Ok(output) => output,
    //     Err((remainder, _)) => remainder.finalize_extract(),
    // }
}
```

The loop is the same `DispatchMatchers` alias described on
[`MatchWithHandlers`](match_with_handlers.md#under-the-hood).

## Related constructs

- [`MatchWithHandlers`](match_with_handlers.md) — the plain matcher, for handlers that need only the
  value.
- [`ExtractFirstFieldAndHandle`](extract_field_and_handle.md),
  [`HandleFirstFieldValue`](handle_field_value.md) — the first-argument adapters its list is built from.
- [`MatchWithValueHandlers`](match_with_value_handlers.md) — the automatic form, whose
  `MatchFirstWithValueHandlers` sibling matches this convention.
- [`HasExtractor`](../../traits/has_extractor.md), [`FinalizeExtract`](../../traits/finalize_extract.md)
  — the traits it stands on.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing a value, with extra arguments, to per-variant
  handlers.

## Source

- [`providers/with_handlers/match_first_with_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/with_handlers/match_first_with_handlers.rs),
  with the borrowed forms in `match_first_with_handlers_ref.rs` and `match_first_with_handlers_mut.rs`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
