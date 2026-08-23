---
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
handler provider on a **context**, the type a capability runs against, when a pipeline is built step by
step through [`PipeHandlers`](../handler/pipe_handlers.md). Like every CGP provider, it carries no
runtime value; the two parameters ride in `PhantomData`.

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

## When to reach for it, and when not

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

It implements `Computer` and `AsyncComputer` for an input of `Result<T1, E>`. On `Ok(value)` it runs
`Cont` on the value and lifts the continuation's output back through `M`. On `Err(err)` it
short-circuits, lifting the error directly to the output and skipping `Cont`. It is the mirror of
[`BindOk`](bind_ok.md), which branches the other way. The `M` parameter lets these binds nest: at
the bottom of a single-layer pipeline it is `IdentMonadic`, and a stacked monad threads a deeper monad
through it.

## Related constructs

- [`BindOk`](bind_ok.md) — the mirror, binding the err branch and short-circuiting on `Ok`.
- [`PipeMonadic`](pipe_monadic.md) — composes `BindErr` internally under `ErrMonadic`.
- The [monad markers](index.md#the-monad-markers) — `ErrMonadic` is the marker `BindErr` implements the
  bind for.
- [`PipeHandlers`](../handler/pipe_handlers.md) — where a hand-built bind step is placed.
- [`ContainsValue`](../../traits/contains_value.md), [`LiftValue`](../../traits/lift_value.md) — the
  traits it uses to inspect and lift a step's value.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — the bind step and how the monads compose.

## Source

- [`monadic/err.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/monadic/err.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
