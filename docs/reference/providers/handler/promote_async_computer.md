---
title: 'PromoteAsyncComputer — the async family'
description: 'The promotion bundle that fills in Handler and the async by-reference members from an AsyncComputer base, such as an async #[cgp_computer] function.'
sidebar_label: 'PromoteAsyncComputer'
sidebar_position: 11
---

# `PromoteAsyncComputer`

Fill in the async members of the handler family from a provider that implements `AsyncComputer`.

:::info

### Generated machinery

**You are not expected to name `PromoteAsyncComputer` directly.**
[`#[cgp_computer]`](../../macros/cgp_computer.md) wires the provider it generates for an `async fn`
returning a plain value to `PromoteAsyncComputer<Self>`. A context names the bundle to route its own
components through such a provider, or through its `HandlerComponent` entry, which serves any
`AsyncComputer`. This page explains what the bundle emits.

:::

## Overview

`PromoteAsyncComputer<Provider>` starts from a provider that implements `AsyncComputer`, the
asynchronous infallible base, and fills in the other async members of the handler family on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against. It is the
async-base counterpart of [`PromoteComputer`](promote_computer.md), and it fills in only async
members, because the synchronous ones cannot be derived from an async base. Like every CGP provider,
it carries no runtime value.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the base
`AsyncComputer` provider. It answers `HandlerComponent`, `AsyncComputerRefComponent`, and
`HandlerRefComponent` but not `AsyncComputerComponent`, so the base is wired to
`AsyncComputerComponent` directly and the bundle fills in the rest:

```rust
delegate_components! {
    App {
        AsyncComputerComponent: DoubleLater,
        HandlerComponent: PromoteAsyncComputer<DoubleLater>,
    }
}
```

## Examples

An async `#[cgp_computer]` function is the base, and the bundle answers `handle` from it:

```rust
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanComputeAsync, CanHandle};

#[cgp_computer]
pub async fn double_later(value: u64) -> u64 {
    value * 2
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        AsyncComputerComponent: DoubleLater,
        HandlerComponent: PromoteAsyncComputer<DoubleLater>,
    }
}

check_components! {
    App {
        [AsyncComputerComponent, HandlerComponent]: ((), u64),
    }
}

pub async fn demo() {
    let code = PhantomData::<()>;

    assert_eq!(App.compute_async(code, 5).await, 10);
    assert_eq!(App.handle(code, 5).await, Ok(10));
}
```

`handle` wraps the awaited `10` in `Ok`, and needs the error type `App` wires. The
`HandlerComponent` entry is one step from the base, so it serves a hand-written `AsyncComputer` too.
`App` is an [environmental context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `PromoteAsyncComputer` when the base is an infallible `AsyncComputer`**, generated from
an `async fn` or written by hand. For a synchronous base use
[`PromoteComputer`](promote_computer.md), and for an async base returning `Result` use
[`PromoteHandler`](promote_handler.md).

## Under the hood

`PromoteAsyncComputer` is a [`delegate_components!`](../../macros/delegate_components.md) table over
a generic inner `Provider`:

```rust
delegate_components! {
    <Provider>
    new PromoteAsyncComputer<Provider> {
        AsyncComputerRefComponent: PromoteRef<Provider>,
        HandlerComponent: Promote<Provider>,
        HandlerRefComponent: PromoteRef<Provider>,
    }
}
```

It routes `HandlerComponent` to [`Promote`](promote.md), which wraps the awaited value in `Ok`, and
the two by-reference members to [`PromoteRef`](promote_ref.md), which need the base to take the
borrow as its input.

## Related constructs

- [`PromoteComputer`](promote_computer.md) — the synchronous-base counterpart.
- [`PromoteHandler`](promote_handler.md) — the counterpart for an async base returning `Result`,
  which forwards its by-reference members here.
- [`Promote`](promote.md), [`PromoteRef`](promote_ref.md) — the lifts this table wires.
- [`AsyncComputer`](../../components/handler/async_computer.md),
  [`Handler`](../../components/handler/handler.md) — the base and the family members it fills in.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and its sync-to-async ordering.

## Source

- [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
