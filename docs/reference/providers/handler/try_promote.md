---
title: 'TryPromote — Result outputs and failure'
description: 'The single-step lift, in both directions, between a computer whose output is a Result and a genuinely fallible TryComputer or Handler.'
sidebar_label: 'TryPromote'
sidebar_position: 7
---

# `TryPromote`

Bridge in both directions between a `Result`-valued output and a fallible handler trait.

## Overview

`TryPromote<Provider>` unifies the two ways of expressing fallibility: a handler that *returns* a
`Result`, and a genuinely fallible handler trait. It converts between them in both directions on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against, and every
impl requires the context to have an error type, which the `Result`'s error must equal. The
[`PromoteTryComputer`](promote_try_computer.md) and [`PromoteHandler`](promote_handler.md) bundles
route through it, [`PipeMonadic`](../monad/pipe_monadic.md) uses it internally, and it is written by
hand when a context wires one slot at a time. Like every CGP provider, it carries no runtime value;
the inner provider rides in `PhantomData`.

## Usage

Import it from `cgp::extra::handler`; it is not in the prelude. It takes one type parameter, the
inner provider, and the direction depends on the slot it fills:

```rust
use cgp::extra::handler::TryPromote;

delegate_components! {
    App {
        // A Computer returning Result<u64, Error> serves the fallible slot.
        TryComputerComponent: TryPromote<CheckedDouble>,
        // A TryComputer serves the infallible slot, returning its Result as a value.
        ComputerComponent: TryPromote<ParseU64>,
    }
}
```

## Examples

Both directions on one context, each from a hand-written provider:

```rust
use core::num::ParseIntError;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::{DebugError, RaiseFrom};
use cgp::extra::handler::{CanCompute, CanTryCompute, TryPromote};

/// A computer whose output is a `Result` in the context's error type.
#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for CheckedDouble
where
    Context: HasErrorType<Error = String>,
{
    type Output = Result<u64, String>;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> Result<u64, String> {
        input.checked_mul(2).ok_or_else(|| "overflow".to_owned())
    }
}

/// A genuinely fallible computer.
#[cgp_impl(new ParseU64)]
#[uses(CanRaiseError<ParseIntError>)]
#[use_type(HasErrorType.Error)]
impl<Code> TryComputer<Code, String> {
    type Output = u64;

    fn try_compute(&self, _code: PhantomData<Code>, input: String) -> Result<u64, Error> {
        input.parse().map_err(Self::raise_error)
    }
}

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,

        TryComputerComponent: TryPromote<CheckedDouble>,
        ComputerComponent: TryPromote<ParseU64>,
    }
}

check_components! {
    App {
        TryComputerComponent: ((), u64),
        ComputerComponent: ((), String),
    }
}

pub fn demo() {
    let code = PhantomData::<()>;

    // The `Result` output becomes the fallible interface.
    assert_eq!(App.try_compute(code, 21), Ok(42));
    assert_eq!(App.try_compute(code, u64::MAX), Err("overflow".to_owned()));

    // The fallible computer surfaces its `Result` as a plain value.
    assert_eq!(App.compute(code, "12".to_owned()), Ok(12));
}
```

`CheckedDouble`'s output is `Result<u64, String>`, and `App`'s error type is `String`, so
`TryPromote<CheckedDouble>` reads that `Result` as the failure path. `TryPromote<ParseU64>` goes the
other way: `compute` returns `ParseU64`'s whole `Result`. `App` is an
[environmental context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `TryPromote` to convert between a `Result`-valued output and a fallible trait when
wiring a slot by hand.** A `#[cgp_computer]` function returning `Result` is already wired to
[`PromoteTryComputer`](promote_try_computer.md), which routes its fallible slot here. For the other
lifts use [`Promote`](promote.md), [`PromoteAsync`](promote_async.md), or
[`PromoteRef`](promote_ref.md).

## Under the hood

`TryPromote<Provider>` carries the inner provider in `PhantomData`:

```rust
pub struct TryPromote<Provider>(pub PhantomData<Provider>);
```

- As a `TryComputer`, it requires the inner `Provider: Computer` whose `Output` is
  `Result<Output, Context::Error>`, and returns that result. This turns a computer that *returns* a
  `Result` into a genuine fallible computer.
- As a `Computer`, it goes the other way: given `Provider: TryComputer`, its output type is
  `Result<Output, Context::Error>`, surfacing the fallible result as an ordinary value.

The analogous pair lifts between `Handler`, from an `AsyncComputer` returning a `Result`, and
`AsyncComputer`, from a `Handler`. All four impls require the context to have an error type. Neither
direction removes a failure path; the second only moves it into the output.

## Related constructs

- [`Promote`](promote.md), [`PromoteAsync`](promote_async.md), [`PromoteRef`](promote_ref.md) — the
  other single-step lifts.
- [`PromoteTryComputer`](promote_try_computer.md), [`PromoteHandler`](promote_handler.md) — the
  bundles that route through `TryPromote`.
- [`PipeMonadic`](../monad/pipe_monadic.md) — uses `TryPromote` to bridge fallible handlers when
  composing a fallible monadic pipeline.
- [`TryComputer`](../../components/handler/try_computer.md),
  [`Handler`](../../components/handler/handler.md) — the fallible family members it bridges.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and its fallibility axis.

## Source

- [`providers/try_promote.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/try_promote.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
