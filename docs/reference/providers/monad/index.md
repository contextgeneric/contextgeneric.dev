---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Monad providers

Turn a list of handlers and a choice of monad into a single handler that short-circuits on the
appropriate branch.

## Overview

The monad providers build [monadic handler composition](/docs/concepts/monadic-handlers) on top of the
[`Computer`](../../components/handler/computer.md) family. They let a sequence of handlers whose outputs carry a
"continue" case and a "stop" case be chained without pattern-matching each step by hand: the monad
decides which case threads forward and which short-circuits. A built pipeline is itself a provider for
`Computer`, `AsyncComputer`, `TryComputer`, and `Handler`, so it wires into a **context**, the type a
capability runs against, exactly like any other handler.

The providers divide into three groups:

- [`PipeMonadic`](pipe_monadic.md) is the pipeline builder, the provider a user wires or invokes.
- The **monad markers** below are the zero-sized types that select the short-circuiting behavior.
- [`BindOk`](bind_ok.md) and [`BindErr`](bind_err.md) are the per-step bind providers that `PipeMonadic`
  composes internally, and that can also be used directly.

## The monad markers

A monad marker selects which branch of a step's output continues the pipeline and which stops it. CGP
defines three base markers and a transformer form for the two that branch on `Result`. They are not
providers, so they have no page of their own; they are the `M` argument the providers above take.

**`IdentMonadic`** is the identity monad. It threads every value forward and never short-circuits, so a
`PipeMonadic<IdentMonadic, …>` is equivalent to plain composition with
[`PipeHandlers`](../handler/pipe_handlers.md).

**`OkMonadic`** short-circuits on `Ok` and continues on `Err`. It is the mirror behavior, running until
something succeeds, which suits a fallback chain.

**`ErrMonadic`** short-circuits on `Err` and continues on `Ok`. This is the ordinary early-return-on-error
behavior, the same as Rust's `?` operator.

The naming reads the opposite of the behavior, and getting it wrong produces a pipeline that runs
exactly when you expected it to stop: **`OkMonadic` continues on `Err`, and `ErrMonadic` continues on
`Ok`**. `ErrMonadic` is the one that behaves like `?`.

**`OkMonadicTrans<M>`** and **`ErrMonadicTrans<M>`** are transformer forms that apply the same behavior
on top of a base monad `M`, so monads can stack over nested result types. Writing
`OkMonadicTrans<ErrMonadic>` builds a monad that short-circuits on an outer `Ok` while threading an
inner `Result` through the err monad beneath it. A single layer of branching needs no explicit
transformer, because the bare markers produce their own transformer form over `IdentMonadic` when used
as transformers. The trait layer that gives a marker its meaning is documented under
[`MonadicBind`](../../traits/monadic_bind.md) and [`MonadicTrans`](../../traits/monadic_trans.md).

## Related constructs

- [`PipeMonadic`](pipe_monadic.md) — the pipeline builder that consumes a marker and a handler list.
- [`BindOk`](bind_ok.md), [`BindErr`](bind_err.md) — the per-step bind providers.
- [`PipeHandlers`](../handler/pipe_handlers.md) — non-monadic composition, which `PipeMonadic<IdentMonadic, …>`
  reduces to.
- [`MonadicBind`](../../traits/monadic_bind.md), [`MonadicTrans`](../../traits/monadic_trans.md),
  [`ContainsValue`](../../traits/contains_value.md), [`LiftValue`](../../traits/lift_value.md) — the
  four traits a marker implements.
- [`Product!`](../../macros/product.md) — the type-level list a pipeline's steps are given in.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — why a pipeline short-circuits and how the monads
  compose.

## Source

- The pipeline provider is in
  [`providers/pipe_monadic.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/providers/pipe_monadic.rs),
  and the markers and bind providers in
  [`monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic).

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
