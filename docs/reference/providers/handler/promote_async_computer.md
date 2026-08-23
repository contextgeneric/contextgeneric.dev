---
sidebar_label: 'PromoteAsyncComputer'
sidebar_position: 11
---

# `PromoteAsyncComputer`

Fill in the handler family from a provider that implements `AsyncComputer`.

:::info

### Generated machinery

**You are not expected to name `PromoteAsyncComputer` directly.** It is wired for a generated provider
whose base is the asynchronous infallible computer. You name it by hand only when wiring a hand-written
`AsyncComputer`'s family explicitly. This page explains what that wiring emits.

:::

## Overview

`PromoteAsyncComputer<Provider>` starts from a provider that implements `AsyncComputer`, the
asynchronous infallible base, and fills in the rest of the handler family on a **context**, the type a
capability runs against. It is the async-base counterpart of [`PromoteComputer`](promote_computer.md).
Like every CGP provider, it carries no runtime value.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the base
`AsyncComputer` provider. `PromoteAsyncComputer` answers `HandlerComponent` and the async-ref members
but not `AsyncComputerComponent`, so the base is wired to `AsyncComputerComponent` directly and the
bundle fills in the rest:

```rust
delegate_components! {
    App {
        AsyncComputerComponent: Double,
        HandlerComponent: PromoteAsyncComputer<Double>,
    }
}
```

## When to reach for it, and when not

**Reach for `PromoteAsyncComputer` when the base is an infallible async `AsyncComputer`** and you wire
its family by hand. For a synchronous base use [`PromoteComputer`](promote_computer.md), and for a
fallible async `Handler` base use [`PromoteHandler`](promote_handler.md).

## Under the hood

`PromoteAsyncComputer` is a [`delegate_components!`](../../macros/delegate_components.md) table over a
generic inner `Provider`. It routes `HandlerComponent` to [`Promote`](promote.md), which wraps the
awaited value in `Ok`, and routes `AsyncComputerRefComponent` and `HandlerRefComponent` to
[`PromoteRef`](promote_ref.md).

## Related constructs

- [`PromoteComputer`](promote_computer.md) — the synchronous-base counterpart.
- [`PromoteHandler`](promote_handler.md) — defers its async-ref components here.
- [`Promote`](promote.md), [`PromoteRef`](promote_ref.md) — the lifts this table wires.
- [`Handler`](../../components/handler/handler.md), [`Computer`](../../components/handler/computer.md) — the family it
  fills in.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and its sync-to-async ordering.

## Source

- [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
