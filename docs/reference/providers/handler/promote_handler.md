---
title: 'PromoteHandler — Handler from an async Result'
description: 'The promotion bundle that fills in Handler and the async by-reference members from an AsyncComputer whose output is a Result, via TryPromote.'
sidebar_label: 'PromoteHandler'
sidebar_position: 12
---

# `PromoteHandler`

Fill in the async fallible members of the handler family from an `AsyncComputer` whose output is a
`Result`.

:::info

### Generated machinery

**You are not expected to name `PromoteHandler` directly.**
[`#[cgp_computer]`](../../macros/cgp_computer.md) wires the provider it generates for an `async fn`
returning `Result` to `PromoteHandler<Self>`. A context names the bundle to route its own components
through such a provider, or through its `HandlerComponent` entry, which serves any `AsyncComputer`
returning `Result`. This page explains what the bundle emits.

:::

## Overview

`PromoteHandler<Provider>` starts from an `AsyncComputer` whose output is a `Result` over the
context's error type, the base `#[cgp_computer]` builds for an `async fn` returning `Result`, and
fills in the fallible async members on a [**context**](/docs/reference/glossary#context), the type
the implementation runs against. Despite its name, its base is not a `Handler`: it produces the
`Handler` by reading the base's `Result` as the failure path. Like every CGP provider, it carries no
runtime value.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the base
provider:

```rust
delegate_components! {
    App {
        HandlerComponent: PromoteHandler<CheckedDouble>,
    }
}
```

It also answers `AsyncComputerRefComponent` and `HandlerRefComponent`, but those need a base that
takes its input by reference; a base over an owned `u64` answers only `HandlerComponent`.

## Examples

An async `#[cgp_computer]` function returning `Result` answers `handle` through the bundle:

```rust
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::CanHandle;

#[cgp_computer]
pub async fn checked_double(value: u64) -> Result<u64, String> {
    value.checked_mul(2).ok_or_else(|| "overflow".to_owned())
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        HandlerComponent: PromoteHandler<CheckedDouble>,
    }
}

check_components! {
    App {
        HandlerComponent: ((), u64),
    }
}

pub async fn demo() {
    let code = PhantomData::<()>;

    assert_eq!(App.handle(code, 21).await, Ok(42));
    assert_eq!(App.handle(code, u64::MAX).await, Err("overflow".to_owned()));
}
```

The function's error type is `String`, which must equal `App`'s, since `TryPromote` passes the `Err`
through unconverted. `App` is an
[environmental context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `PromoteHandler` when the base is an `AsyncComputer` returning `Result`**, generated
from an `async fn` or written by hand. For an infallible async base use
[`PromoteAsyncComputer`](promote_async_computer.md), and for a synchronous base the bundle that
matches it: [`PromoteComputer`](promote_computer.md),
[`PromoteTryComputer`](promote_try_computer.md), or [`PromoteProducer`](promote_producer.md).

## Under the hood

`PromoteHandler` is a [`delegate_components!`](../../macros/delegate_components.md) table over a
generic inner `Provider`:

```rust
delegate_components! {
    <Provider>
    new PromoteHandler<Provider> {
        HandlerComponent: TryPromote<Provider>,
        [
            AsyncComputerRefComponent,
            HandlerRefComponent,
        ] ->
            PromoteAsyncComputer<Provider>,
    }
}
```

It routes `HandlerComponent` to [`TryPromote`](try_promote.md), whose `Handler` impl reads an
`AsyncComputer` returning `Result` as a fallible handler, one step from the base, so it serves a
hand-written base too. The by-reference members forward to
[`PromoteAsyncComputer`](promote_async_computer.md)'s entries, which route through
[`PromoteRef`](promote_ref.md).

## Related constructs

- [`PromoteAsyncComputer`](promote_async_computer.md) — the bundle this forwards its by-reference
  members to.
- [`TryPromote`](try_promote.md) — the lift it routes the handler slot through.
- [`PromoteComputer`](promote_computer.md), [`PromoteTryComputer`](promote_try_computer.md),
  [`PromoteProducer`](promote_producer.md) — the bundles for the synchronous bases.
- [`Handler`](../../components/handler/handler.md) — the family member it produces.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and its most general member.

## Source

- [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
