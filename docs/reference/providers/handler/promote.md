---
sidebar_label: 'Promote'
sidebar_position: 4
---

# `Promote`

Lift a handler upward along the infallible-to-fallible and sync-to-async axes, without adding behavior
of its own.

:::info

### Generated machinery

**You are not expected to name `Promote` directly.** The promotion bundle
[`PromoteComputer`](promote_computer.md) and the others wire it into the right handler slots, and those
bundles are what [`#[cgp_computer]`](../../macros/cgp_computer.md) and
[`#[cgp_producer]`](../../macros/cgp_producer.md) generate. You reach for it by hand only when wiring the
handler family one slot at a time. This page explains what that wiring emits.

:::

## Overview

`Promote<Provider>` re-exposes one inner provider under a more capable member of the handler family,
treating a less capable provider as a more capable one without introducing error or async behavior
itself. The **context**, the type a capability runs against, sees the promoted shape while the inner
provider does the same work. Like every CGP provider, it carries no runtime value; the inner provider
rides in `PhantomData`.

## Usage

Import it from `cgp::extra::handler`. It takes one type parameter, the inner provider:

```rust
use cgp::extra::handler::Promote;

// Fill a TryComputer slot from a plain Computer, wrapping its result in Ok.
delegate_components! {
    App {
        TryComputerComponent: Promote<Double>,
    }
}
```

## When to reach for it, and when not

**You rarely reach for `Promote` by hand.** A provider written with
[`#[cgp_computer]`](../../macros/cgp_computer.md) or [`#[cgp_producer]`](../../macros/cgp_producer.md)
gets the right lifts wired through a bundle such as [`PromoteComputer`](promote_computer.md), so the
whole family is filled in for you. Name `Promote` yourself only when you wire the family one slot at a
time and a slot needs exactly this lift: a producer filling a computer slot, or an infallible result
wrapped in `Ok`. For the sync-to-async lift use [`PromoteAsync`](promote_async.md), for the borrow lift
[`PromoteRef`](promote_ref.md), and for the `Result` bridge [`TryPromote`](try_promote.md).

## Under the hood

`Promote<Provider>` carries the inner provider in `PhantomData` and gives three impls:

```rust
pub struct Promote<Provider>(pub PhantomData<Provider>);
```

- As a `Computer`, it requires the inner `Provider: Producer<Context, Code>` and ignores its own input,
  calling `Provider::produce`. This adapts a producer, which takes no input, to fill a computer slot
  that is handed an input it does not need.
- As a `TryComputer`, it requires `Provider: Computer` and wraps the infallible result in `Ok`.
- As a `Handler`, it requires `Provider: AsyncComputer` and wraps the awaited result in `Ok`.

Each promotion adds the missing capability, either discarding an input or introducing an always-`Ok`
result, without changing what the inner provider computes.

## Related constructs

- [`PromoteAsync`](promote_async.md), [`PromoteRef`](promote_ref.md), [`TryPromote`](try_promote.md) —
  the other single-step lifts, along different axes.
- [`PromoteComputer`](promote_computer.md) and the other bundles — wire `Promote` into the family
  automatically.
- [`#[cgp_computer]`](../../macros/cgp_computer.md), [`#[cgp_producer]`](../../macros/cgp_producer.md) —
  generate providers that the bundles promote.
- [`Producer`](../../components/producer.md), [`Computer`](../../components/computer.md),
  [`Handler`](../../components/handler.md) — the family members it lifts between.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and the orderings the promotions trade on.

## Source

- [`providers/promote.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
