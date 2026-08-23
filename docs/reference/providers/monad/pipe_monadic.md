---
sidebar_label: 'PipeMonadic'
sidebar_position: 1
---

# `PipeMonadic`

Compose a list of handlers under a monad into a single handler that short-circuits on the monad's stop
branch.

## Overview

`PipeMonadic<M, Providers>` composes a handler list `Providers` under a monad `M` into one
short-circuiting handler, on a **context**, the type a capability runs against. Each step runs only on
the previous step's continue branch, and the monad decides which branch that is. The result is a
provider for `Computer`, `AsyncComputer`, `TryComputer`, and `Handler`, so it wires like any other
handler. Like every CGP provider, it carries no runtime value; `M` and the list ride in `PhantomData`.

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

Composing a homogeneous list under a base monad. With an `Increment` computer that returns
`Result<u8, &str>`, `Ok` on success and `Err("overflow")` on overflow, three under `ErrMonadic` chain on
the `Ok` value and stop at the first error:

```rust
PipeMonadic::<ErrMonadic, Product![Increment, Increment, Increment]>::compute(&context, code, 253)
// 253 -> Ok(254) -> Ok(255) -> Err("overflow")
```

Stacking monads handles nested results. Composing handlers that return `Result<Result<(), u8>, &str>`
under `OkMonadicTrans<ErrMonadic>` short-circuits on the outer `Ok` while threading the inner `Result`
through the err monad, and the same list composed under `OkMonadic` can be driven through the fallible
`try_compute` and async `handle` entry points, because `PipeMonadic` implements `TryComputer` and
`Handler` as well:

```rust
PipeMonadic::<OkMonadic, Product![ReturnOkErr, ReturnOkOk, ReturnOkErr]>::try_compute(&context, code, 1)
```

## When to reach for it, and when not

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
`BindProviders<M>` computation walks the list so the first provider runs on the input and its result is
bound, through the monad, to the monadically-composed rest of the list.

For the fallible components `TryComputerComponent` and `HandlerComponent`, it bridges through the err
monad. It first maps every provider to [`TryPromote`](../handler/try_promote.md), demoting fallible
handlers to plain computers whose output is an explicit `Result`; the mapper that rewrites the whole list
is `TryPromoteProviders`, which implements [`MapType`](../../traits/map_type.md). It then applies
`ErrMonadic` as a transformer on top of `M`, composes the demoted list under the transformed monad, and
wraps the composed provider back in `TryPromote` to restore the fallible interface. A `PipeMonadic` over
fallible handlers therefore short-circuits on the context's error type in addition to whatever branching
`M` contributes. The whole fold happens during trait resolution, so the pipeline is not a runtime
structure.

## Related constructs

- The [monad markers](index.md#the-monad-markers) — the `M` argument that selects the branching.
- [`BindOk`](bind_ok.md), [`BindErr`](bind_err.md) — the per-step providers `PipeMonadic` composes
  internally.
- [`PipeHandlers`](../handler/pipe_handlers.md) — the non-monadic pipeline this generalizes.
- [`TryPromote`](../handler/try_promote.md) — the lift it uses to bridge fallible and infallible
  handlers.
- [`MonadicBind`](../../traits/monadic_bind.md), [`MonadicTrans`](../../traits/monadic_trans.md) — the
  traits it bounds on while folding.
- [`Computer`](../../components/handler/computer.md), [`Handler`](../../components/handler/handler.md) — the family a
  pipeline implements.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — why a pipeline short-circuits and how the monads
  compose.

## Source

- [`providers/pipe_monadic.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/providers/pipe_monadic.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
