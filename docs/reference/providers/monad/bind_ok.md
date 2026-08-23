---
sidebar_label: 'BindOk'
sidebar_position: 2
---

# `BindOk`

The per-step provider for one bind of the ok monad: run the continuation on the `Err` payload, and
short-circuit on `Ok`.

## Overview

`BindOk<M, Cont>` implements a single bind step of the ok monad. [`PipeMonadic`](pipe_monadic.md)
composes it for you when it folds a pipeline under [`OkMonadic`](index.md#the-monad-markers), so most
code never names it. It is usable directly as a handler provider on a **context**, the type a capability
runs against, when a pipeline is built step by step through
[`PipeHandlers`](../handler/pipe_handlers.md) rather than through `PipeMonadic`. Like every CGP
provider, it carries no runtime value; the two parameters ride in `PhantomData`.

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

## When to reach for it, and when not

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

It implements `Computer` and `AsyncComputer` for an input of `Result<T, E1>`. On `Err(payload)` it runs
`Cont` on the payload and lifts the continuation's output back through `M`. On `Ok(value)` it
short-circuits, lifting the value directly to the output and skipping `Cont`. It is the mirror of
[`BindErr`](bind_err.md), which branches the other way.

## Related constructs

- [`BindErr`](bind_err.md) — the mirror, binding the ok branch and short-circuiting on `Err`.
- [`PipeMonadic`](pipe_monadic.md) — composes `BindOk` internally under `OkMonadic`.
- The [monad markers](index.md#the-monad-markers) — `OkMonadic` is the marker `BindOk` implements the
  bind for.
- [`PipeHandlers`](../handler/pipe_handlers.md) — where a hand-built bind step is placed.
- [`ContainsValue`](../../traits/contains_value.md), [`LiftValue`](../../traits/lift_value.md) — the
  traits it uses to inspect and lift a step's value.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — the bind step and how the monads compose.

## Source

- [`monadic/ok.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/monadic/ok.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
