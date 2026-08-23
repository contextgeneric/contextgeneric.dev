---
sidebar_label: 'PromoteProducer'
sidebar_position: 10
---

# `PromoteProducer`

Fill in the handler family from a provider that implements `Producer`.

:::info

### Generated machinery

**You are not expected to name `PromoteProducer` directly.**
[`#[cgp_producer]`](../../macros/cgp_producer.md) wires it across the family for the provider it
generates. You name `PromoteProducer<MyProvider>` by hand only when wiring a hand-written producer's
family explicitly. This page explains what that wiring emits.

:::

## Overview

`PromoteProducer<Provider>` starts from a [`Producer`](../../components/handler/producer.md), a provider that
takes no input, and fills in every input-taking member of the handler family on a **context**, the
type a capability runs against. The single produced value flows out of every handler shape regardless
of the input, which the promotion discards. Like every CGP provider, it carries no runtime value.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the base
`Producer` provider. The base answers `ProducerComponent`, and `PromoteProducer` answers
`ComputerComponent` (and the rest of the family) by discarding the input and calling the producer:

```rust
delegate_components! {
    App {
        ProducerComponent: MakeDefault,
        ComputerComponent: PromoteProducer<MakeDefault>,
    }
}
```

Most code gets this wiring from [`#[cgp_producer]`](../../macros/cgp_producer.md) rather than writing it.

## When to use it

**Reach for `PromoteProducer` when the base is an input-free `Producer`** and you wire its family by
hand rather than through [`#[cgp_producer]`](../../macros/cgp_producer.md). For an input-taking base use
[`PromoteComputer`](promote_computer.md), [`PromoteTryComputer`](promote_try_computer.md),
[`PromoteAsyncComputer`](promote_async_computer.md), or [`PromoteHandler`](promote_handler.md).

## Under the hood

`PromoteProducer` is a [`delegate_components!`](../../macros/delegate_components.md) table over a generic
inner `Provider`. It routes `ComputerComponent` to [`Promote`](promote.md), which discards the
computer's input and calls the producer, and defers every remaining component to
[`PromoteComputer`](promote_computer.md), so the produced value reaches every handler shape.

## Related constructs

- [`#[cgp_producer]`](../../macros/cgp_producer.md) — generates a `Producer` provider and wires
  `PromoteProducer`.
- [`Promote`](promote.md) — the lift that discards a computer's input and calls the producer.
- [`PromoteComputer`](promote_computer.md) — the bundle this defers to.
- [`Producer`](../../components/handler/producer.md), [`Computer`](../../components/handler/computer.md) — the base and
  the family it fills in.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family, including the input-free producer.

## Source

- [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
