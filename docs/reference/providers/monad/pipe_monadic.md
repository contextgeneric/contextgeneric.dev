---
title: 'PipeMonadic — a short-circuiting pipeline'
description: 'The provider that composes a Product! list of handlers under a monad marker, so each step runs only on the previous step''s continue case.'
sidebar_label: 'PipeMonadic'
sidebar_position: 1
---

# `PipeMonadic`

Compose a list of handlers under a monad into a single handler that short-circuits on the monad's stop
branch.

## Overview

`PipeMonadic<M, Providers>` composes a handler list `Providers` under a monad `M` into one
short-circuiting handler, on a [**context**](/docs/reference/glossary#context), the type the
implementation runs against. Each step runs only on the previous step's continue branch, and the
monad decides which branch that is. The result is a provider for `Computer`, `AsyncComputer`,
`TryComputer`, and `Handler`, so it wires like any other handler. Like every CGP provider, it
carries no runtime value; `M` and the list ride in `PhantomData`.

## Usage

Import it from `cgp::extra::monad::providers`, and the monad markers from `cgp::extra::monad::monadic`.
None of these is in the prelude. `PipeMonadic` takes a monad marker and a
[`Product!`](../../macros/product.md) list of handler providers:

```rust
use cgp::extra::monad::providers::PipeMonadic;
use cgp::extra::monad::monadic::err::ErrMonadic;

delegate_components! {
    App {
        ComputerComponent:
            PipeMonadic<ErrMonadic, Product![Increment, Increment, Increment]>,
    }
}
```

Under `ErrMonadic`, each step runs on the previous step's `Ok` value and the first `Err` becomes the
pipeline's output. The [monad markers](index.md#the-monad-markers) choose the branching; the naming trap
there is worth re-reading, since `ErrMonadic` is the `?`-style monad.

## Examples

A pipeline of three fallible computers, wired under `ErrMonadic` and called through the consumer
trait:

```rust
use cgp::prelude::*;
use cgp::extra::handler::CanCompute;
use cgp::extra::monad::monadic::err::ErrMonadic;
use cgp::extra::monad::providers::PipeMonadic;

#[cgp_computer]
pub fn increment(value: u8) -> Result<u8, &'static str> {
    value.checked_add(1).ok_or("overflow")
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent: PipeMonadic<ErrMonadic, Product![Increment, Increment, Increment]>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), u8),
    }
}

pub fn demo() {
    assert_eq!(App.compute(PhantomData::<()>, 1), Ok(4));
    // 253 -> Ok(254) -> Ok(255) -> Err("overflow")
    assert_eq!(App.compute(PhantomData::<()>, 253), Err("overflow"));
}
```

Each `Increment` returns `Result<u8, &str>`, and under `ErrMonadic` each step runs on the previous
step's `Ok` value, so the first overflow becomes the pipeline's output and the remaining steps do not
run. `App` is an [environmental context](/docs/reference/glossary#environmental-context), and
`Increment` comes from [`#[cgp_computer]`](../../macros/cgp_computer.md).

Stacking monads handles nested results. These handlers return `Result<Result<(), u8>, &str>`, and
under `OkMonadicTrans<ErrMonadic>` the err monad handles the outer `Result` while the ok layer handles
the one inside it, so the pipeline stops on an outer `Err` or an inner `Ok`:

```rust
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::monad::monadic::err::ErrMonadic;
use cgp::extra::monad::monadic::ok::{OkMonadic, OkMonadicTrans};
use cgp::extra::monad::providers::PipeMonadic;

#[cgp_computer]
pub fn return_ok_ok(_value: u8) -> Result<Result<(), u8>, &'static str> {
    Ok(Ok(()))
}

#[cgp_computer]
pub fn return_ok_err(value: u8) -> Result<Result<(), u8>, &'static str> {
    Ok(Err(value))
}

#[cgp_computer]
pub fn return_err(_value: u8) -> Result<Result<(), u8>, &'static str> {
    Err("error")
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<&'static str>,
    }
}

pub fn demo() {
    let code = PhantomData::<()>;

    // An inner `Ok` stops the pipeline; the last step never runs.
    assert_eq!(
        PipeMonadic::<
            OkMonadicTrans<ErrMonadic>,
            Product![ReturnOkErr, ReturnOkOk, ReturnOkErr],
        >::compute(&App, code, 1),
        Ok(Ok(())),
    );

    // An outer `Err` stops it too.
    assert_eq!(
        PipeMonadic::<
            OkMonadicTrans<ErrMonadic>,
            Product![ReturnErr, ReturnOkOk, ReturnOkErr],
        >::compute(&App, code, 1),
        Err("error"),
    );

    // Through the fallible bridge, plain `OkMonadic` behaves the same way.
    assert_eq!(
        PipeMonadic::<
            OkMonadic,
            Product![ReturnOkErr, ReturnOkOk, ReturnOkErr],
        >::try_compute(&App, code, 1),
        Ok(Ok(())),
    );
}
```

The pipelines are called on the provider directly, through the `Computer` and `TryComputer` provider
traits the prelude supplies, rather than wired. The last call goes through `try_compute`, where
`PipeMonadic` stacks `OkMonadic` over `ErrMonadic` itself and takes the context's error, here
`&'static str`, from the outer `Result`; that is why `App` wires an error type.

## When to use it

**Reach for `PipeMonadic` when a pipeline's steps should short-circuit on a branch**, such as an error
or the first success, rather than always feeding the next step. For a pipeline where every step runs
unconditionally, use [`PipeHandlers`](../handler/pipe_handlers.md), which `PipeMonadic<IdentMonadic, …>`
reduces to. When the whole chain lives in one provider body, Rust's own `?` operator is clearer than any
composition.

## Under the hood

`PipeMonadic<M, Providers>` carries the monad and the list in `PhantomData`:

```rust
pub struct PipeMonadic<M, Providers>(pub PhantomData<(M, Providers)>);
```

It implements `ComputerComponent` and `AsyncComputerComponent` by folding the list: an internal
`BindProviders<M>` computation walks the list so the first provider runs on the input and its result
is bound, through the monad, to the monadically-composed rest of the list. For `[A, B, C]` the
result is `ComposeHandlers<A, Bind<ComposeHandlers<B, Bind<C>>>>`, where `Bind<P>` is the bind step
the monad produces, such as `BindErr<IdentMonadic, P>` for `ErrMonadic`. A one-element list is its
only provider, and an empty list builds nothing.

For the fallible components `TryComputerComponent` and `HandlerComponent`, it bridges through the
err monad. It first maps every provider to [`TryPromote`](../handler/try_promote.md), demoting
fallible handlers to plain computers whose output is an explicit `Result`; the mapper that rewrites
the whole list is `TryPromoteProviders`, which implements
[`MapType`](../../traits/type-level/map_type.md). It then applies `M` as a transformer over
`ErrMonadic`, so the err monad handles the outer `Result` and `M` the layer inside it, composes the
demoted list under that stack, and wraps the composed provider back in `TryPromote` to restore the
fallible interface. A `PipeMonadic` over fallible handlers therefore short-circuits on the context's
error type in addition to whatever branching `M` contributes. The whole fold happens during trait
resolution, so the pipeline is not a runtime structure.

## Related constructs

- The [monad markers](index.md#the-monad-markers) — the `M` argument that selects the branching.
- [`BindOk`](bind_ok.md), [`BindErr`](bind_err.md) — the per-step providers `PipeMonadic` composes
  internally.
- [`PipeHandlers`](../handler/pipe_handlers.md) — the non-monadic pipeline this generalizes.
- [`TryPromote`](../handler/try_promote.md) — the lift it uses to bridge fallible and infallible
  handlers.
- [`MonadicBind`](../../traits/monad/monadic_bind.md), [`MonadicTrans`](../../traits/monad/monadic_trans.md) — the
  traits it bounds on while folding.
- [`Computer`](../../components/handler/computer.md), [`Handler`](../../components/handler/handler.md) — the family a
  pipeline implements.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — why a pipeline short-circuits and how the monads
  compose.

## Source

- [`providers/pipe_monadic.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/providers/pipe_monadic.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
