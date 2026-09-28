---
title: 'AsyncComputer — an async computation'
description: 'The handler-family component for a computation that must await but cannot fail: its async method returns the Output directly rather than a Result.'
sidebar_label: 'AsyncComputer'
sidebar_position: 5
---

# `AsyncComputer`

The asynchronous, infallible member of the handler family: a [`Computer`](./computer.md) that
awaits.

## Overview

`AsyncComputer` is for computations that must await but still cannot fail. Reading a value that is
already in memory is a [`Computer`](./computer.md); awaiting a timer, or a channel that always
yields a value, is an `AsyncComputer`. It turns an `Input` into an `Output` under a phantom `Code`
tag, against a [**context**](/docs/reference/glossary#context) (the type the implementation runs
against), and its async method returns the `Output` directly rather than a `Result`.

It sits between [`Computer`](./computer.md), which drops the asynchrony, and
[`Handler`](./handler.md), which adds a failure path to it. Like the synchronous computer it never
names an error type, so it does not have [`HasErrorType`](../has_error_type.md) as a
[supertrait](/docs/reference/glossary#supertrait). See the [handler family overview](./index.md) for
how the members relate and promote.

## Definition

`CanComputeAsync` is defined as:

```rust
#[async_trait]
#[cgp_component(AsyncComputer)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
pub trait CanComputeAsync<Code, Input> {
    type Output;

    async fn compute_async(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
}
```

Its attributes:

- [`#[async_trait]`](../../macros/async_trait.md) — rewrites the `async fn` into a method returning `-> impl Future`, the lint-clean, allocation-free form, which adds no `Send` bound to the future.
- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `AsyncComputer` that implementations target and the wiring key `AsyncComputerComponent`, while `CanComputeAsync` stays the consumer trait callers use.
- [`#[prefix]`](../../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.extra.handler`, so a context that joins that namespace binds its provider at `@cgp.extra.handler.AsyncComputerComponent` rather than at the bare key.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the legacy `UseDelegate` and `UseInputDelegate` providers, which dispatch on the `Code` tag and on the `Input` type through an inner table; the `open` statement replaces both, dispatching on either parameter or on the pair.

## Usage

`AsyncComputer` and `AsyncComputerComponent` are in the prelude. The consumer trait
`CanComputeAsync` is not, and is imported from `cgp::extra::handler`. The method is `async` and
takes the input by value:

```rust
async fn compute_async(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
```

A context gains the operation by wiring `AsyncComputerComponent` to a provider, and the component
dispatches on both the `Code` tag and the `Input` type. A provider comes from one of four places:

- **A hand-written provider** declares `async fn compute_async` in its impl, as the example below
  does.
- **[`PromoteAsync<P>`](../../providers/handler/promote_async.md)**, imported from
  `cgp::extra::handler`, runs a synchronous [`Computer`](./computer.md) `P` inside the async method,
  so `AsyncComputerComponent: PromoteAsync<Double>` answers `compute_async` from a `Double`
  computer.
- **[`#[cgp_computer]`](../../macros/cgp_computer.md)** implements `AsyncComputer` directly for an
  `async fn`, and reaches it through promotion for a synchronous one.
- **[`UseField<Tag>`](../../providers/use_field.md)** forwards the computation to the value stored
  in the context's `Tag` field, which must itself implement `CanComputeAsync`.

Its by-reference sibling is [`AsyncComputerRef`](./async_computer_ref.md).

## Examples

An async computer provider, wired into a context and awaited through the consumer trait:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::CanComputeAsync;

#[cgp_new_provider]
impl<Context, Code> AsyncComputer<Context, Code, u64> for DoubleAsync {
    type Output = u64;

    async fn compute_async(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

pub struct App;

delegate_components! {
    App {
        AsyncComputerComponent: DoubleAsync,
    }
}

check_components! {
    App {
        AsyncComputerComponent: ((), u64),
    }
}

pub async fn run(app: &App) -> u64 {
    app.compute_async(PhantomData::<()>, 21).await // 42
}
```

`DoubleAsync` implements the provider trait with an `async fn`, so a real provider could await
inside it. `App` is an [environmental context](/docs/reference/glossary#environmental-context), and
the component is **[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**:
the computation acts on the `Input`, while `App` decides the provider. The future `run` returns
needs an executor to run it, which CGP leaves to the application.

## When to use it

**Reach for `AsyncComputer` for a computation that awaits but cannot fail.** A computation that only
reads its input fits the by-reference [`AsyncComputerRef`](./async_computer_ref.md) better.

Reach for [`Computer`](./computer.md) instead when nothing needs awaiting, since `PromoteAsync`
lifts a synchronous computer into an `AsyncComputer` and the computer stays usable in the
synchronous positions too. Reach for [`Handler`](./handler.md) when the computation can also fail.
Implement whichever single member fits and let the wiring promote it.

## Related constructs

- [`Computer`](./computer.md) — the synchronous counterpart; promotes into this.
- [`AsyncComputerRef`](./async_computer_ref.md) — the by-reference variant of this component.
- [`Handler`](./handler.md) — adds a failure path to the asynchrony.
- [`#[async_trait]`](../../macros/async_trait.md) — the attribute that rewrites its `async fn`.
- [`#[cgp_computer]`](../../macros/cgp_computer.md) — builds a provider that answers this, directly or
  through promotion.
- [Handler combinators](../../providers/handler/index.md) — `PromoteAsync` and the promotion
  bundles.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Recovering `Send` bounds](/docs/concepts/send-bounds) — restoring the `Send` guarantee the async
  method does not carry.

## Source

- `AsyncComputer` and `AsyncComputerRef`, and the `UseField` async computer:
  [`async_computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/async_computer.rs)
- `PromoteAsync`:
  [`promote_async.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_async.rs)
- Re-exported through `cgp::extra::handler`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
