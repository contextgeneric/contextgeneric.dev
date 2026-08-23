---
sidebar_label: 'PromoteAsync'
sidebar_position: 5
---

# `PromoteAsync`

Lift a synchronous handler into an asynchronous one.

:::info

### Generated machinery

**You are not expected to name `PromoteAsync` directly.** The promotion bundle
[`PromoteComputer`](promote_computer.md) wires it into the async slots, and the bundles are what
[`#[cgp_computer]`](../../macros/cgp_computer.md) generates. You reach for it by hand only when wiring
the handler family one slot at a time. This page explains what that wiring emits.

:::

## Overview

`PromoteAsync<Provider>` runs a synchronous inner provider inside an async method, so a synchronous
handler serves an asynchronous slot on a **context**, the type a capability runs against. The returned
future is ready immediately, so no actual asynchrony is added. Like every CGP provider, it carries no
runtime value; the inner provider rides in `PhantomData`.

## Usage

Import it from `cgp::extra::handler`. It takes one type parameter, the inner provider:

```rust
use cgp::extra::handler::PromoteAsync;

// Serve an AsyncComputer slot from a synchronous Computer.
delegate_components! {
    App {
        AsyncComputerComponent: PromoteAsync<Double>,
    }
}
```

## When to reach for it, and when not

**You rarely reach for `PromoteAsync` by hand.** The [`PromoteComputer`](promote_computer.md) bundle
wires it into the async slots, so a synchronous provider serves the async family without you naming it.
Reach for it directly only when wiring a slot by hand and that slot needs a synchronous provider run in
an async method. For the infallible-to-fallible or producer lift use [`Promote`](promote.md), and for
the borrow lift [`PromoteRef`](promote_ref.md).

## Under the hood

`PromoteAsync<Provider>` carries the inner provider in `PhantomData`:

```rust
pub struct PromoteAsync<Provider>(pub PhantomData<Provider>);
```

- As an `AsyncComputer`, it requires `Provider: Computer` and runs it synchronously inside the async
  method, returning a future that is already ready.
- As a `Handler`, it requires `Provider: TryComputer` and returns that fallible synchronous result, so
  a synchronous fallible computer becomes an async fallible handler.

## Related constructs

- [`Promote`](promote.md), [`PromoteRef`](promote_ref.md), [`TryPromote`](try_promote.md) — the other
  single-step lifts.
- [`PromoteComputer`](promote_computer.md) and the other bundles — wire `PromoteAsync` into the family
  automatically.
- [`Computer`](../../components/computer.md), [`TryComputer`](../../components/try_computer.md),
  [`Handler`](../../components/handler.md) — the family members it lifts between.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and the sync-to-async ordering this lift trades on.

## Source

- [`providers/promote_async.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_async.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
