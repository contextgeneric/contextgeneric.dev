---
title: 'BindErr — one ?-style bind step'
description: 'The per-step provider of the err monad: run the continuation on a step''s Ok value and stop on Err. PipeMonadic composes it under ErrMonadic.'
sidebar_label: 'BindErr'
sidebar_position: 3
---

# `BindErr`

The per-step provider for one bind of the err monad: run the continuation on the `Ok` value, and
short-circuit on `Err`.

## Overview

`BindErr<M, Cont>` implements a single bind step of the err monad, the ordinary `?`-style behavior.
[`PipeMonadic`](pipe_monadic.md) composes it for you when it folds a pipeline under
[`ErrMonadic`](index.md#the-monad-markers), so most code never names it. It is usable directly as a
handler provider on a [**context**](/docs/reference/glossary#context), the type the implementation
runs against, when a pipeline is built step by step through
[`PipeHandlers`](../handler/pipe_handlers.md). Like every CGP provider, it carries no runtime value;
the two parameters ride in `PhantomData`.

## Usage

Import it from `cgp::extra::monad::monadic::err`. It is not in the prelude. `M` is the monad layer
beneath this bind, and `Cont` is the continuation provider run on the continue branch:

```rust
use cgp::extra::monad::monadic::err::BindErr;
use cgp::extra::monad::monadic::ident::IdentMonadic;
use cgp::extra::handler::PipeHandlers;

// PipeMonadic builds this internally for a two-element list under ErrMonadic.
type Pipeline = PipeHandlers<Product![Increment, BindErr<IdentMonadic, Increment>]>;
// 1 -> Ok(2) -> BindErr runs the second Increment on 2 -> Ok(3)
```

## Examples

One `?`-style bind step built by hand, which is what `PipeMonadic` composes for a two-element list
under `ErrMonadic`:

```rust
use cgp::prelude::*;
use cgp::extra::handler::PipeHandlers;
use cgp::extra::monad::monadic::err::BindErr;
use cgp::extra::monad::monadic::ident::IdentMonadic;

#[cgp_computer]
pub fn increment(value: u8) -> Result<u8, &'static str> {
    value.checked_add(1).ok_or("overflow")
}

pub type Pipeline = PipeHandlers<Product![Increment, BindErr<IdentMonadic, Increment>]>;

pub fn demo() {
    let code = PhantomData::<()>;

    // 1 -> Ok(2); BindErr runs the second Increment on 2 -> Ok(3).
    assert_eq!(Pipeline::compute(&(), code, 1), Ok(3));
    // The first step overflows, so the bind skips the second.
    assert_eq!(Pipeline::compute(&(), code, 255), Err("overflow"));
}
```

`Pipeline` is called on the provider directly, with `()` as the context, since `Increment` reads
nothing from it.

## When to use it

**Reach for the [monad providers](index.md) rather than `BindErr` directly.** Wiring
[`PipeMonadic`](pipe_monadic.md) with the [`ErrMonadic`](index.md#the-monad-markers) marker composes
`BindErr` for you. Name it yourself only when building a pipeline step by step inside a
[`PipeHandlers`](../handler/pipe_handlers.md) list. For the mirror that continues on `Err` and
short-circuits on `Ok`, use [`BindOk`](bind_ok.md).

## Under the hood

`BindErr<M, Cont>` carries the monad layer and the continuation in `PhantomData`:

```rust
pub struct BindErr<M, Cont>(pub PhantomData<(M, Cont)>);
```

It implements `Computer` and `AsyncComputer` for an input of `Result<T1, E>`. The `Computer` impl
is:

```rust
#[cgp_provider]
impl<Context, Code, T1, T2, E, M, Cont> Computer<Context, Code, Result<T1, E>> for BindErr<M, Cont>
where
    Cont: Computer<Context, Code, T1>,
    M: ContainsValue<Cont::Output, Value = Result<T2, E>> + LiftValue<Result<T2, E>, Cont::Output>,
{
    type Output = M::Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: Result<T1, E>) -> Self::Output {
        match input {
            Ok(value) => M::lift_output(Cont::compute(context, code, value)),
            Err(err) => M::lift_value(Err(err)),
        }
    }
}
```

On `Ok(value)` it runs `Cont` on the value and lifts the continuation's output back through `M`. On
`Err(err)` it short-circuits, lifting the error directly to the output and skipping `Cont`. It is
the mirror of [`BindOk`](bind_ok.md), which branches the other way. The `M` parameter lets these
binds nest: at the bottom of a single-layer pipeline it is `IdentMonadic`, and a stacked monad
threads a deeper monad through it.

## Related constructs

- [`BindOk`](bind_ok.md) — the mirror, binding the err branch and short-circuiting on `Ok`.
- [`PipeMonadic`](pipe_monadic.md) — composes `BindErr` internally under `ErrMonadic`.
- The [monad markers](index.md#the-monad-markers) — `ErrMonadic` is the marker `BindErr` implements the
  bind for.
- [`PipeHandlers`](../handler/pipe_handlers.md) — where a hand-built bind step is placed.
- [`ContainsValue`](../../traits/monad/contains_value.md), [`LiftValue`](../../traits/monad/lift_value.md) — the
  traits it uses to inspect and lift a step's value.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — the bind step and how the monads compose.

## Source

- [`monadic/err.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/monadic/err.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
