---
sidebar_label: 'PromoteHandler'
sidebar_position: 12
---

# `PromoteHandler`

Fill in the handler family from a provider that implements `Handler`, the most general base.

:::info

### Generated machinery

**You are not expected to name `PromoteHandler` directly.** It is wired for a generated provider whose
base is the async fallible handler. You name it by hand only when wiring a hand-written `Handler`'s
family explicitly. This page explains what that wiring emits.

:::

## Overview

`PromoteHandler<Provider>` starts from the most general base, a provider that implements
[`Handler`](../../components/handler/handler.md), and fills in the rest of the family on a **context**, the
type a capability runs against. Like every CGP provider, it carries no runtime value.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the base
`Handler` provider:

```rust
delegate_components! {
    App {
        [HandlerComponent, HandlerRefComponent]: PromoteHandler<CheckedAdd>,
    }
}
```

## When to reach for it, and when not

**Reach for `PromoteHandler` when the base is the fully general async fallible `Handler`** and you wire
its family by hand. For a less capable base, use the bundle that matches it:
[`PromoteComputer`](promote_computer.md), [`PromoteTryComputer`](promote_try_computer.md),
[`PromoteProducer`](promote_producer.md), or [`PromoteAsyncComputer`](promote_async_computer.md).

## Under the hood

`PromoteHandler` is a [`delegate_components!`](../../macros/delegate_components.md) table over a generic
inner `Provider`. It routes `HandlerComponent` to [`TryPromote`](try_promote.md), which exposes the
handler as an async computer returning an explicit `Result`, and defers the async-ref components to
[`PromoteAsyncComputer`](promote_async_computer.md).

## Related constructs

- [`PromoteAsyncComputer`](promote_async_computer.md) — the bundle this defers its async-ref components
  to.
- [`TryPromote`](try_promote.md) — the lift it routes the handler slot through.
- [`PromoteComputer`](promote_computer.md), [`PromoteTryComputer`](promote_try_computer.md),
  [`PromoteProducer`](promote_producer.md) — the bundles for the other base traits.
- [`Handler`](../../components/handler/handler.md) — the base and the most general family member.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and its most general member.

## Source

- [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
