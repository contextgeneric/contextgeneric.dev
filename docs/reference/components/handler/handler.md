---
title: 'Handler — an async, fallible computation'
description: 'The general handler-family component: an async computation that can fail, returning a Result in the context''s error type. Simpler members promote into it.'
sidebar_label: 'Handler'
sidebar_position: 3
slug: /reference/components/handler/handler
---

# `Handler`

The general computation component: asynchronous and fallible.

## Overview

`Handler` is for computations that must both await and fail, such as a call to a remote service,
which awaits the response and must report a failure. It is the corner of the [handler
family](/docs/concepts/handlers) that has both properties: it turns an `Input` into an `Output`
under a phantom `Code` tag and returns a `Result` in the abstract error type of the
[**context**](/docs/reference/glossary#context), the type the implementation runs against.

The other owned-input members each drop one of these properties. Without the failure path a
`Handler` is an [`AsyncComputer`](./async_computer.md), without the asynchrony it is a
[`TryComputer`](./try_computer.md), and without both it is a [`Computer`](./computer.md). Each of
them promotes into a `Handler` through the handler combinators, so generic code bounded by
`CanHandle` accepts a provider written for any of them once the context wires the matching
promotion. The reverse does not hold, since a general computation cannot be assumed synchronous or
infallible.

## Definition

`CanHandle` is defined as:

```rust
#[async_trait]
#[cgp_component(Handler)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
#[use_type(HasErrorType.Error)]
pub trait CanHandle<Code, Input> {
    type Output;

    async fn handle(&self, _tag: PhantomData<Code>, input: Input) -> Result<Self::Output, Error>;
}
```

Its attributes:

- [`#[async_trait]`](../../macros/async_trait.md) — rewrites the `async fn` into a method returning `-> impl Future`, the lint-clean, allocation-free form, which adds no `Send` bound to the future.
- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `Handler` that implementations target and the wiring key `HandlerComponent`, while `CanHandle` stays the consumer trait callers use.
- [`#[prefix]`](../../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.extra.handler`, so a context that joins that namespace binds its provider at `@cgp.extra.handler.HandlerComponent` rather than at the bare key.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the legacy `UseDelegate` and `UseInputDelegate` providers, which dispatch on the `Code` tag and on the `Input` type through an inner table; the `open` statement replaces both, dispatching on either parameter or on the pair.
- [`#[use_type]`](../../attributes/use_type.md) — adds `HasErrorType` as a [supertrait](/docs/reference/glossary#supertrait) and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`Handler` and `HandlerComponent` are in the prelude. The consumer trait `CanHandle` is not, and is
imported from `cgp::extra::handler`. The consumer method is `async` and returns a `Result` in the
context's error type:

```rust
async fn handle(&self, _tag: PhantomData<Code>, input: Input) -> Result<Self::Output, Error>;
```

A context gains the operation by wiring `HandlerComponent` to a provider and by wiring an error
type. Most `Handler` implementations are not written directly but produced by promoting a simpler
provider, and each promotion provider, imported from `cgp::extra::handler`, takes one step:

- **[`PromoteAsync<P>`](../../providers/handler/promote_async.md)** makes a `Handler` from a
  [`TryComputer`](./try_computer.md) by running it inside an async method.
- **[`Promote<P>`](../../providers/handler/promote.md)** makes a `Handler` from an
  [`AsyncComputer`](./async_computer.md) by wrapping its awaited output in `Ok`.
- **[`TryPromote<P>`](../../providers/handler/try_promote.md)** makes a `Handler` from an
  `AsyncComputer` whose `Output` is already `Result<T, Error>`.

A plain [`Computer`](./computer.md) takes two steps, as `PromoteAsync<Promote<P>>`: `Promote` makes
the `TryComputer`, and `PromoteAsync` makes the `Handler` from it. A provider written with
[`#[cgp_computer]`](../../macros/cgp_computer.md) or
[`#[cgp_producer]`](../../macros/cgp_producer.md) is wired to its own promotion bundle, so a context
can wire `HandlerComponent` to it directly.

The component dispatches on both the `Code` tag and the `Input` type, so wiring can route different
codes or inputs to different handlers, as covered under [dispatching](/docs/concepts/dispatching).
Its by-reference sibling [`HandlerRef`](./handler_ref.md), documented on its own page, is identical
except that its `handle_ref` method borrows the input as `&Input`.

## Examples

A generic consumer bounded by `CanHandle`, and a context that answers it with a synchronous computer
lifted in two steps:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Promote, PromoteAsync};

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        HandlerComponent: PromoteAsync<Promote<Double>>,
    }
}

check_components! {
    App {
        HandlerComponent: ((), u64),
    }
}

pub async fn run_with<Context, Code>(
    context: &Context,
    input: u64,
) -> Result<Context::Output, Context::Error>
where
    Context: CanHandle<Code, u64>,
{
    context.handle(PhantomData::<Code>, input).await
}

pub async fn demo() -> Result<u64, String> {
    run_with::<App, ()>(&App, 21).await // Ok(42)
}
```

`run_with` works for any context that wires a handler for the given `Code` and a `u64` input,
whether the provider behind it is a genuine `Handler` or a simpler one lifted by promotion, as
`Double` is here. `App` is an [environmental
context](/docs/reference/glossary#environmental-context), and the component is
**[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**: the computation
acts on the `Input`, while `App` decides which provider answers and which error type it returns.

## When to use it

**Bound generic pipeline code by `CanHandle` when it should accept any computation, whichever of the
async and fallible properties the provider actually has.** A consumer written against `Handler` is
the most reusable, and it is the right target for I/O steps, request handlers, and composed
pipelines.

Do not *write* a `Handler` provider by hand when a simpler member fits: implement a
[`Computer`](./computer.md) for a pure transform, a [`TryComputer`](./try_computer.md) for a
fallible synchronous one, or a [`Producer`](./producer.md) for an input-free one, and let the wiring
promote it. The simpler provider stays usable in the positions a `Handler` cannot fill.

## Common Mistakes

**A promotion bundle wired on a context expects its provider to be wired to the same bundle.** The
`HandlerComponent` entry of `PromoteComputer<P>` is `PromoteAsync<P>`, which needs `P` to be a
`TryComputer`, and a hand-written computer is not one. So wiring `HandlerComponent` to
`PromoteComputer<Double>`, for a `Double` that implements only `Computer`, fails at the check:

```text
error[E0277]: the trait bound `Double: DelegateComponent<TryComputerComponent>` is not satisfied
...
   = note: required for `Double` to implement `TryComputer<App, (), u64>`
   = note: required for `PromoteAsync<Double>` to implement `IsProviderFor<cgp::prelude::HandlerComponent, App, ((), u64)>`
```

Spell both steps as `PromoteAsync<Promote<Double>>`, or write the provider with
[`#[cgp_computer]`](../../macros/cgp_computer.md), which wires it to `PromoteComputer<Self>` so the
bundle finds the `TryComputer` it needs.

## Related constructs

- [`HandlerRef`](./handler_ref.md) — the by-reference variant that borrows its input.
- [`Computer`](./computer.md) — the synchronous, infallible member; promotes into `Handler` in two
  steps.
- [`TryComputer`](./try_computer.md) — the fallible synchronous member; promotes into `Handler`.
- [`AsyncComputer`](./async_computer.md) — the async infallible member; promotes into `Handler`.
- [`Producer`](./producer.md) — the input-free member of the family.
- [`HasErrorType`](../has_error_type.md) — the supertrait supplying the `Error` a handler returns.
- [Handler combinators](../../providers/handler/index.md) — compose handlers and promote simpler
  members into them.
- [`#[cgp_computer]`](../../macros/cgp_computer.md) / [`#[cgp_producer]`](../../macros/cgp_producer.md) —
  wire a function so it answers `CanHandle` through promotion.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Monadic handlers](/docs/concepts/monadic-handlers) — chaining handlers that short-circuit through
  a monad.
- [Recovering `Send` bounds](/docs/concepts/send-bounds) — restoring the `Send` guarantee the async
  method does not carry.

## Source

- `Handler` and `HandlerRef`:
  [`handler.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/handler.rs)
- The promotion providers and bundles:
  [`cgp-handler/src/providers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-handler/src/providers)
- Re-exported through `cgp::extra::handler`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
