---
sidebar_label: 'MatchWithHandlers'
sidebar_position: 1
---

# `MatchWithHandlers`

Match an enum against a spelled-out list of per-variant handlers, proving the match exhaustive without a
wildcard arm.

## Overview

`MatchWithHandlers<Handlers>` is the owned-input matcher. Given a value, it converts the value to its
extractor and runs a list of handlers over it, stopping at the first one that matches. It runs on a
**context**, the type a capability runs against. When the list is exhausted the remaining type has every
variant ruled out and is therefore uninhabited, which is what lets the matcher return the bare output
with no fallback arm. Like every CGP provider, it carries no runtime value; the handler list rides in
`PhantomData`.

Two borrowed forms match a value without moving it: `MatchWithHandlersRef<Handlers>` over `&Input` and
`MatchWithHandlersMut<Handlers>` over `&mut Input`. Each of the three implements both the synchronous
`Computer` and the `AsyncComputer` form.

## Usage

Import it from `cgp::extra::dispatch`. It takes one type parameter, a [`Product!`](../../macros/product.md)
list of per-variant adapters. The list is normally one [`ExtractFieldAndHandle`](extract_field_and_handle.md)
per variant, each wrapping its payload handler in [`HandleFieldValue`](handle_field_value.md):

```rust
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};

delegate_components! {
    App {
        ComputerComponent:
            MatchWithHandlers<Product![
                ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
                ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
            ]>,
    }
}
```

For a list built automatically from the enum's own variants, reach for
[`MatchWithValueHandlers`](match_with_value_handlers.md) instead, which spells this list out for you.

## Examples

Dispatching a `Shape` enum to a per-variant area computer:

```rust
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

// ComputeArea implements Computer for both the Circle and Rectangle payloads.
let circle = Shape::Circle(Circle { radius: 5.0 });

let _area = MatchWithHandlers::<
    Product![
        ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
        ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
    ],
>::compute(&(), PhantomData::<()>, circle);
```

The list tries `Circle` first. If the value is a circle, `ComputeArea` runs on the `Circle` payload and
the loop stops. Otherwise the remainder carries `Circle` ruled out into the `Rectangle` adapter, the
last arm, so its failure would leave an uninhabited remainder that the matcher discharges.

## When to reach for it, and when not

**Reach for `MatchWithHandlers` when you want to name a handler per variant explicitly**, or when the
variants map to handlers in a way the automatic form cannot express. For the common case where every
variant is routed the same way, [`MatchWithValueHandlers`](match_with_value_handlers.md) builds the list
from the enum's fields and is far shorter. When the handlers also need extra arguments passed alongside
the value, use [`MatchFirstWithHandlers`](match_first_with_handlers.md).

## Under the hood

`MatchWithHandlers<Handlers>` carries the list in `PhantomData`:

```rust
pub struct MatchWithHandlers<Handlers>(pub PhantomData<Handlers>);
```

Its `Computer` impl requires the input to implement [`HasExtractor`](../../traits/has_extractor.md), runs
the shared matcher loop over `Input::Extractor` to obtain `Result<Output, Remainder>`, and calls
`finalize_extract_result` on that result so the uninhabited remainder is discharged:

```rust
impl<Context, Code, Input, Output, Remainder, Handlers> Computer<Context, Code, Input>
    for MatchWithHandlers<Handlers>
where
    Input: HasExtractor,
    DispatchMatchers<Handlers>:
        Computer<Context, Code, Input::Extractor, Output = Result<Output, Remainder>>,
    Remainder: FinalizeExtract,
{
    type Output = Output;
    // compute: DispatchMatchers::compute(context, code, input.to_extractor())
    //              .finalize_extract_result()
}
```

`DispatchMatchers<Handlers>` is the matcher loop every matcher shares. It is a type alias for
[`PipeMonadic`](../monad/pipe_monadic.md) under the `OkMonadic` monad:

```rust
pub type DispatchMatchers<Providers> = PipeMonadic<OkMonadic, Providers>;
```

Each handler in the list returns `Ok(output)` when it matches and `Err(remainder)` when it does not,
handing back the extractor with one more variant ruled out. Under `OkMonadic` the loop threads along the
`Err` branch and short-circuits on `Ok`, so it runs handler by handler, carrying the shrinking remainder
forward, until one returns `Ok`. `DispatchMatchers` is an implementation detail rather than a construct
a user names.

## Related constructs

- [`MatchWithValueHandlers`](match_with_value_handlers.md) — builds the handler list from the enum's own
  variants instead of spelling it out.
- [`MatchFirstWithHandlers`](match_first_with_handlers.md) — the same matcher for the multi-argument
  convention.
- [`ExtractFieldAndHandle`](extract_field_and_handle.md), [`HandleFieldValue`](handle_field_value.md) —
  the adapters the list is built from.
- [`PipeMonadic`](../monad/pipe_monadic.md) — the `OkMonadic` pipeline the matcher loop is built on.
- [`HasExtractor`](../../traits/has_extractor.md), [`FinalizeExtract`](../../traits/finalize_extract.md),
  [`FinalizeExtractResult`](../../traits/finalize_extract_result.md) — the traits it stands on.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to per-variant handlers.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum pattern the matchers serve.

## Source

- [`providers/with_handlers/match_with_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/with_handlers/match_with_handlers.rs),
  with the borrowed forms in `match_with_handlers_ref.rs` and `match_with_handlers_mut.rs`, and the loop
  alias in
  [`providers/dispatchers/dispatch_matchers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/dispatchers/dispatch_matchers.rs).

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
