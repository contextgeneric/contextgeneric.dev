---
sidebar_label: 'PromoteTryComputer'
sidebar_position: 9
---

# `PromoteTryComputer`

Fill in the handler family from a provider that implements `TryComputer`.

:::info

### Generated machinery

**You are not expected to name `PromoteTryComputer` directly.** It is wired for a generated provider
that implements the fallible synchronous base. You name it by hand only when wiring a hand-written
`TryComputer`'s family explicitly. This page explains what that wiring emits.

:::

## Overview

`PromoteTryComputer<Provider>` starts from a provider that implements `TryComputer`, the synchronous
fallible base, and fills in the rest of the handler family on a **context**, the type a capability runs
against. It first turns the fallible base into a plain computer, then derives the rest of the family
from there. Like every CGP provider, it carries no runtime value.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the base
`TryComputer` provider:

```rust
delegate_components! {
    App {
        [TryComputerComponent, HandlerComponent]: PromoteTryComputer<CheckedAdd>,
    }
}
```

## When to use it

**Reach for `PromoteTryComputer` when the base provider is a synchronous fallible `TryComputer`** and
you wire its family by hand. For a plain `Computer` base use [`PromoteComputer`](promote_computer.md);
for the other bases use [`PromoteProducer`](promote_producer.md),
[`PromoteAsyncComputer`](promote_async_computer.md), or [`PromoteHandler`](promote_handler.md).

## Under the hood

`PromoteTryComputer` is a [`delegate_components!`](../../macros/delegate_components.md) table over a
generic inner `Provider`. It routes `TryComputerComponent` to [`TryPromote`](try_promote.md), which
exposes the fallible base as a computer returning an explicit `Result`, and defers every remaining
component to [`PromoteComputer`](promote_computer.md), so the rest of the family is derived from that
computer form.

## Related constructs

- [`PromoteComputer`](promote_computer.md) — the bundle this defers to once the base is a plain
  computer.
- [`TryPromote`](try_promote.md) — the lift it routes the fallible slot through.
- [`PromoteProducer`](promote_producer.md), [`PromoteAsyncComputer`](promote_async_computer.md),
  [`PromoteHandler`](promote_handler.md) — the bundles for the other base traits.
- [`TryComputer`](../../components/handler/try_computer.md), [`Handler`](../../components/handler/handler.md) — the
  family it fills in.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and its fallibility axis.

## Source

- [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
