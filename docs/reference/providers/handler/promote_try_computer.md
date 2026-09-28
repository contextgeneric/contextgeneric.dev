---
title: 'PromoteTryComputer — the family from a Result'
description: 'The promotion bundle that fills in the handler family from a Computer whose output is a Result, the base #[cgp_computer] builds for a fallible function.'
sidebar_label: 'PromoteTryComputer'
sidebar_position: 9
---

# `PromoteTryComputer`

Fill in the handler family from a `Computer` whose output is a `Result`.

:::info

### Generated machinery

**You are not expected to name `PromoteTryComputer` directly.**
[`#[cgp_computer]`](../../macros/cgp_computer.md) wires the provider it generates for a function
returning `Result` to `PromoteTryComputer<Self>`. A context names the bundle only to route its own
components through such a provider, or through the one entry that serves any base, as
[Common Mistakes](#common-mistakes) sets out. This page explains what the bundle emits.

:::

## Overview

`PromoteTryComputer<Provider>` starts from a `Computer` whose output is a `Result` over the
context's error type, the base [`#[cgp_computer]`](../../macros/cgp_computer.md) generates for a
function returning `Result`, and fills in the rest of the handler family on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against. It reads
that `Result` as the failure path for the fallible slot through [`TryPromote`](try_promote.md), and
derives the other members from the computer form through [`PromoteComputer`](promote_computer.md).
Like every CGP provider, it carries no runtime value.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the base
provider, a `Computer` returning `Result`:

```rust
delegate_components! {
    App {
        [TryComputerComponent, HandlerComponent]: PromoteTryComputer<CheckedDouble>,
    }
}
```

Here `CheckedDouble` comes from `#[cgp_computer]`, which is what lets the `HandlerComponent` entry
resolve.

## Examples

A fallible `#[cgp_computer]` function answers the fallible members through the bundle:

```rust
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, CanTryCompute};

#[cgp_computer]
pub fn checked_double(value: u64) -> Result<u64, String> {
    value.checked_mul(2).ok_or_else(|| "overflow".to_owned())
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        [TryComputerComponent, HandlerComponent]: PromoteTryComputer<CheckedDouble>,
    }
}

check_components! {
    App {
        [TryComputerComponent, HandlerComponent]: ((), u64),
    }
}

pub async fn demo() {
    let code = PhantomData::<()>;

    assert_eq!(App.try_compute(code, 21), Ok(42));
    assert_eq!(App.try_compute(code, u64::MAX), Err("overflow".to_owned()));
    assert_eq!(App.handle(code, 21).await, Ok(42));
}
```

The function's error type is `String`, and it must equal `App`'s, because `TryPromote` passes the
`Err` through unconverted. `App` is an [environmental
context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `PromoteTryComputer` when a context routes its handler components through a fallible
`#[cgp_computer]` provider.** For a plain `Computer` base use
[`PromoteComputer`](promote_computer.md); for the other bases use
[`PromoteProducer`](promote_producer.md), [`PromoteAsyncComputer`](promote_async_computer.md), or
[`PromoteHandler`](promote_handler.md).

## Under the hood

`PromoteTryComputer` is a [`delegate_components!`](../../macros/delegate_components.md) table over a
generic inner `Provider`:

```rust
delegate_components! {
    <Provider>
    new PromoteTryComputer<Provider> {
        TryComputerComponent: TryPromote<Provider>,
        [
            ComputerRefComponent,
            TryComputerRefComponent,
            AsyncComputerComponent,
            AsyncComputerRefComponent,
            HandlerComponent,
            HandlerRefComponent,
        ] ->
            PromoteComputer<Provider>,
    }
}
```

It routes `TryComputerComponent` to [`TryPromote`](try_promote.md), which reads the base's `Result`
output as the failure path, and forwards every remaining key to
[`PromoteComputer`](promote_computer.md)'s own entry for it, with the `->` operator, so those
members derive from the computer form.

## Common Mistakes

**Only the `TryComputerComponent` entry serves a hand-written base.** `TryPromote<P>` needs only a
`Computer` returning `Result`, but `HandlerComponent` goes through `PromoteComputer<P>` to
`PromoteAsync<P>`, which looks for a `TryComputer` on the base itself. A hand-written base not wired
to the bundle fails at the check:

```text
error[E0277]: the trait bound `CheckedDouble: DelegateComponent<TryComputerComponent>` is not satisfied
...
   = note: required for `CheckedDouble` to implement `TryComputer<App, (), u64>`
   = note: required for `PromoteAsync<CheckedDouble>` to implement `IsProviderFor<cgp::prelude::HandlerComponent, App, ((), u64)>`
```

Write the base with `#[cgp_computer]`, or wire the handler slot as
`PromoteAsync<TryPromote<CheckedDouble>>`.

## Related constructs

- [`PromoteComputer`](promote_computer.md) — the bundle this forwards to for the computer form.
- [`TryPromote`](try_promote.md) — the lift it routes the fallible slot through.
- [`PromoteProducer`](promote_producer.md), [`PromoteAsyncComputer`](promote_async_computer.md),
  [`PromoteHandler`](promote_handler.md) — the bundles for the other bases.
- [`TryComputer`](../../components/handler/try_computer.md),
  [`Handler`](../../components/handler/handler.md) — the family it fills in.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and its fallibility axis.

## Source

- [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
