---
title: 'BindOk — one fallback bind step'
description: 'The per-step provider of the ok monad: run the continuation on a step''s Err value and stop on Ok. PipeMonadic composes it under OkMonadic.'
sidebar_label: 'BindOk'
sidebar_position: 2
---

# `BindOk`

The per-step provider for one bind of the ok monad: run the continuation on the `Err` payload, and
short-circuit on `Ok`.

## Overview

`BindOk<M, Cont>` implements a single bind step of the ok monad. [`PipeMonadic`](pipe_monadic.md)
composes it for you when it folds a pipeline under [`OkMonadic`](index.md#the-monad-markers), so
most code never names it. It is usable directly as a handler provider on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against, when a
pipeline is built step by step through [`PipeHandlers`](../handler/pipe_handlers.md) rather than
through `PipeMonadic`. Like every CGP provider, it carries no runtime value; the two parameters ride
in `PhantomData`.

## Usage

Import it from `cgp::extra::monad::monadic::ok`. It is not in the prelude. `M` is the monad layer beneath
this bind, and `Cont` is the continuation provider run on the continue branch:

```rust
use cgp::extra::monad::monadic::ok::BindOk;
use cgp::extra::monad::monadic::ident::IdentMonadic;
use cgp::extra::handler::PipeHandlers;

// Build one ok-bind step by hand inside a PipeHandlers list.
type Pipeline = PipeHandlers<Product![Classify, BindOk<IdentMonadic, Halve>]>;
```

At the bottom of a single-layer pipeline `M` is `IdentMonadic`; a stacked monad threads a deeper monad
through it.

## Examples

A fallback chain built by hand: `Classify` accepts a small value and hands a large one on as an `Err`,
and the bind step runs `Halve` only on that `Err`:

```rust
use cgp::prelude::*;
use cgp::extra::handler::PipeHandlers;
use cgp::extra::monad::monadic::ident::IdentMonadic;
use cgp::extra::monad::monadic::ok::BindOk;

/// Accept a small value, or hand a too-large one to the fallback as an `Err`.
#[cgp_computer]
pub fn classify(value: u8) -> Result<u8, u8> {
    if value < 10 { Ok(value) } else { Err(value) }
}

/// The fallback, run only on the `Err` branch: halve the rejected value.
#[cgp_computer]
pub fn halve(value: u8) -> Result<u8, u8> {
    Ok(value / 2)
}

pub type Pipeline = PipeHandlers<Product![Classify, BindOk<IdentMonadic, Halve>]>;

pub fn demo() {
    let code = PhantomData::<()>;

    // 5 is accepted, so BindOk stops and the fallback never runs.
    assert_eq!(Pipeline::compute(&(), code, 5), Ok(5));
    // 20 is rejected, so BindOk runs the fallback on the Err value: 20 -> 10.
    assert_eq!(Pipeline::compute(&(), code, 20), Ok(10));
}
```

`Pipeline` is called on the provider directly, with `()` as the context, since neither computer reads
anything from it. `PipeMonadic<OkMonadic, Product![Classify, Halve]>` builds the same pipeline.

## When to use it

**Reach for the [monad providers](index.md) rather than `BindOk` directly.** Wiring
[`PipeMonadic`](pipe_monadic.md) with the [`OkMonadic`](index.md#the-monad-markers) marker composes
`BindOk` for you. Name it yourself only when building a pipeline step by step inside a
[`PipeHandlers`](../handler/pipe_handlers.md) list. For the `?`-style mirror that continues on `Ok` and
short-circuits on `Err`, use [`BindErr`](bind_err.md).

## Under the hood

`BindOk<M, Cont>` carries the monad layer and the continuation in `PhantomData`:

```rust
pub struct BindOk<M, Cont>(pub PhantomData<(M, Cont)>);
```

It implements `Computer` and `AsyncComputer` for an input of `Result<T, E1>`. The `Computer` impl
is:

```rust
#[cgp_provider]
impl<Context, Code, T, E1, E2, M, Cont> Computer<Context, Code, Result<T, E1>> for BindOk<M, Cont>
where
    Cont: Computer<Context, Code, E1>,
    M: ContainsValue<Cont::Output, Value = Result<T, E2>> + LiftValue<Result<T, E2>, Cont::Output>,
{
    type Output = M::Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: Result<T, E1>) -> Self::Output {
        match input {
            Err(value) => M::lift_output(Cont::compute(context, code, value)),
            Ok(err) => M::lift_value(Ok(err)),
        }
    }
}
```

On `Err(payload)` it runs `Cont` on the payload and lifts the continuation's output back through
`M`. On `Ok(value)` it short-circuits, lifting the value directly to the output and skipping `Cont`.
It is the mirror of [`BindErr`](bind_err.md), which branches the other way.

## Related constructs

- [`BindErr`](bind_err.md) — the mirror, binding the ok branch and short-circuiting on `Err`.
- [`PipeMonadic`](pipe_monadic.md) — composes `BindOk` internally under `OkMonadic`.
- The [monad markers](index.md#the-monad-markers) — `OkMonadic` is the marker `BindOk` implements the
  bind for.
- [`PipeHandlers`](../handler/pipe_handlers.md) — where a hand-built bind step is placed.
- [`ContainsValue`](../../traits/monad/contains_value.md), [`LiftValue`](../../traits/monad/lift_value.md) — the
  traits it uses to inspect and lift a step's value.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — the bind step and how the monads compose.

## Source

- [`monadic/ok.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/monadic/ok.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
