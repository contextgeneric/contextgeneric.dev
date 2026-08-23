---
sidebar_label: 'PromoteComputer'
sidebar_position: 8
---

# `PromoteComputer`

Fill in every member of the handler family from a provider that implements `Computer`.

:::info

### Generated machinery

**You are not expected to name `PromoteComputer` directly.**
[`#[cgp_computer]`](../../macros/cgp_computer.md) wires it across the family for the provider it
generates, so one written `Computer` impl answers the whole handler surface. You name
`PromoteComputer<MyProvider>` by hand only when wiring a hand-written computer's family explicitly. This
page explains what that wiring emits.

:::

## Overview

`PromoteComputer<Provider>` starts from a provider that implements `Computer`, the by-value synchronous
infallible base, and fills in every other member of the handler family by promotion, on a **context**,
the type a capability runs against. It is a delegation table that routes each remaining handler
component to the right single-step promotion. Like every CGP provider, it carries no runtime value.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the base
`Computer` provider. `PromoteComputer` answers every family member *except* `ComputerComponent`, so the
base is wired to `ComputerComponent` directly and the bundle fills in the rest:

```rust
delegate_components! {
    App {
        ComputerComponent: Double,
        [
            TryComputerComponent,
            AsyncComputerComponent,
            HandlerComponent,
        ]: PromoteComputer<Double>,
    }
}
```

Most code gets this wiring from [`#[cgp_computer]`](../../macros/cgp_computer.md) rather than writing it.

## When to reach for it, and when not

**Reach for `PromoteComputer` when you wire a hand-written `Computer` provider's family explicitly**
rather than through [`#[cgp_computer]`](../../macros/cgp_computer.md), which wires it for you. Use a
different bundle when the base is a different trait: [`PromoteTryComputer`](promote_try_computer.md)
from a `TryComputer`, [`PromoteProducer`](promote_producer.md) from a `Producer`,
[`PromoteAsyncComputer`](promote_async_computer.md) from an `AsyncComputer`, and
[`PromoteHandler`](promote_handler.md) from a `Handler`.

## Under the hood

`PromoteComputer` is defined with [`delegate_components!`](../../macros/delegate_components.md) over a
generic inner `Provider`. It routes the fallible slot to [`Promote`](promote.md) (wrap in `Ok`), the
async slots to [`PromoteAsync`](promote_async.md) (run synchronously in an async method), and every
`…Ref` slot to [`PromoteRef`](promote_ref.md) (dereference, then defer to the base):

```rust
delegate_components! {
    <Provider>
    new PromoteComputer<Provider> {
        ComputerRefComponent: PromoteRef<Provider>,
        TryComputerComponent: Promote<Provider>,
        TryComputerRefComponent: PromoteRef<Provider>,
        AsyncComputerComponent: PromoteAsync<Provider>,
        AsyncComputerRefComponent: PromoteRef<Provider>,
        HandlerComponent: PromoteAsync<Provider>,
        HandlerRefComponent: PromoteRef<Provider>,
    }
}
```

The base `ComputerComponent` is the inner provider itself; the table fills in the other seven.

## Related constructs

- [`#[cgp_computer]`](../../macros/cgp_computer.md) — generates a `Computer` provider and wires
  `PromoteComputer` across the family.
- [`PromoteTryComputer`](promote_try_computer.md), [`PromoteProducer`](promote_producer.md),
  [`PromoteAsyncComputer`](promote_async_computer.md), [`PromoteHandler`](promote_handler.md) — the
  bundles for the other base traits.
- [`Promote`](promote.md), [`PromoteAsync`](promote_async.md), [`PromoteRef`](promote_ref.md) — the
  single-step lifts this table wires.
- [`Computer`](../../components/handler/computer.md), [`Handler`](../../components/handler/handler.md) — the family it
  fills in.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and why one written trait can answer all of it.

## Source

- [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
