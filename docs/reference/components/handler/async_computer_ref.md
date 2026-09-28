---
title: 'AsyncComputerRef — async, borrowed input'
description: 'The handler-family component for a computation that awaits, cannot fail, and only reads its input, which its async method takes as &Input.'
sidebar_label: 'AsyncComputerRef'
sidebar_position: 9
---

# `AsyncComputerRef`

The async, by-reference member of the handler family: an [`AsyncComputer`](./async_computer.md) that
borrows its input.

## Overview

`AsyncComputerRef` combines two of the family's axes: it is asynchronous like
[`AsyncComputer`](./async_computer.md) and takes its input by reference like
[`ComputerRef`](./computer_ref.md). A computation that must await and only *reads* its argument,
while still never failing, fits it: its method is `async`, receives `&Input`, and returns the
`Output` directly with no failure path. It runs against a
[**context**](/docs/reference/glossary#context), the type the implementation runs against.

Like the other computers it never names an error type, so it does not have
[`HasErrorType`](../has_error_type.md) as a [supertrait](/docs/reference/glossary#supertrait). See
the [handler family overview](./index.md) for how the members relate and promote.

## Definition

`CanComputeAsyncRef` is defined as:

```rust
#[async_trait]
#[cgp_component(AsyncComputerRef)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
pub trait CanComputeAsyncRef<Code, Input> {
    type Output;

    async fn compute_async_ref(&self, _code: PhantomData<Code>, input: &Input) -> Self::Output;
}
```

Its attributes:

- [`#[async_trait]`](../../macros/async_trait.md) — rewrites the `async fn` into a method returning `-> impl Future`, the lint-clean, allocation-free form, which adds no `Send` bound to the future.
- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `AsyncComputerRef` that implementations target and the wiring key `AsyncComputerRefComponent`, while `CanComputeAsyncRef` stays the consumer trait callers use.
- [`#[prefix]`](../../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.extra.handler`, so a context that joins that namespace binds its provider at `@cgp.extra.handler.AsyncComputerRefComponent` rather than at the bare key.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the legacy `UseDelegate` and `UseInputDelegate` providers, which dispatch on the `Code` tag and on the `Input` type through an inner table; the `open` statement replaces both, dispatching on either parameter or on the pair.

## Usage

`AsyncComputerRef` and `AsyncComputerRefComponent` are in the prelude. The consumer trait
`CanComputeAsyncRef` is not, and is imported from `cgp::extra::handler`. The method is `async` and
borrows the input:

```rust
async fn compute_async_ref(&self, _code: PhantomData<Code>, input: &Input) -> Self::Output;
```

A context gains the operation by wiring `AsyncComputerRefComponent` to a provider, and the component
dispatches on both the `Code` tag and the `Input` type. Besides a hand-written provider, the
[`PromoteRef`](../../providers/handler/promote_ref.md) combinator answers it from an `AsyncComputer`
written for `&'a Input` at every lifetime, which is the entry every promotion bundle uses. The same
combinator serves the reverse direction: an `AsyncComputerRef` provider answers `compute_async` for
an owned input that dereferences to its `Input`. See [`ComputerRef`](./computer_ref.md#usage) for
the two directions, which work the same way here.

## Examples

A provider that counts the words of a borrowed string, wired into a context and awaited:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::CanComputeAsyncRef;

#[cgp_new_provider]
impl<Context, Code> AsyncComputerRef<Context, Code, String> for CountWords {
    type Output = usize;

    async fn compute_async_ref(
        _context: &Context,
        _code: PhantomData<Code>,
        input: &String,
    ) -> usize {
        input.split_whitespace().count()
    }
}

pub struct App;

delegate_components! {
    App {
        AsyncComputerRefComponent: CountWords,
    }
}

check_components! {
    App {
        AsyncComputerRefComponent: ((), String),
    }
}

pub async fn run(app: &App, text: &String) -> usize {
    app.compute_async_ref(PhantomData::<()>, text).await
}
```

`run` awaits the computation while the caller keeps ownership of `text`. `App` is an [environmental
context](/docs/reference/glossary#environmental-context), and the component is
**[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**: the computation
acts on the `Input`, while `App` decides the provider.

## When to use it

**Reach for `AsyncComputerRef` when a computation is async and infallible and only reads its
input.** That is a narrow combination, so code often implements a simpler member and lets promotion
bridge to this one, which works when that member takes the borrow as its input.

Reach for [`AsyncComputer`](./async_computer.md) when the computation takes the input by value, for
[`ComputerRef`](./computer_ref.md) when a borrowed-input computation needs no awaiting, and for
[`HandlerRef`](./handler_ref.md) when a borrowed-input async computation can also fail.

## Related constructs

- [`AsyncComputer`](./async_computer.md) — the owned-input counterpart.
- [`ComputerRef`](./computer_ref.md) — the synchronous by-reference computer.
- [`HandlerRef`](./handler_ref.md) — the fallible async by-reference member.
- [Handler combinators](../../providers/handler/index.md) — `PromoteRef` and the promotion bundles.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.

## Source

- `AsyncComputer` and `AsyncComputerRef`:
  [`async_computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/async_computer.rs)
- `PromoteRef`:
  [`promote_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_ref.rs)
- Re-exported through `cgp::extra::handler`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
