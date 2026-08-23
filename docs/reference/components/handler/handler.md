---
sidebar_label: 'Handler'
sidebar_position: 3
---

# `Handler`

The general computation component: asynchronous, fallible, and error-aware.

## Overview

`Handler` is the corner of the [handler family](/docs/concepts/handlers) that offers everything the
family can: a computation that runs asynchronously *and* can fail. A handler that calls a remote service
must await the response and must report failures, so it lives where the family is both async and
fallible. It transforms an `Input` into an `Output` under a phantom `Code` tag, returning a `Result`
against the **context's** abstract error type, where the context is the type a capability runs against
that supplies the values an implementation needs as its own fields.

Every other handler component is a special case of `Handler` obtained by dropping one capability: drop
the failure path and it becomes an [`AsyncComputer`](./computer.md); drop the asynchrony and it becomes
a [`TryComputer`](./try_computer.md); drop both and it becomes a [`Computer`](./computer.md). This is why
generic pipeline code bounds against `Handler`: it is the one bound *every* member of the family can
meet, because each simpler provider promotes safely up to it. The reverse does not hold, since a general
computation cannot be assumed pure or synchronous.

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

    async fn handle(
        &self,
        _tag: PhantomData<Code>,
        input: Input,
    ) -> Result<Self::Output, Error>;
}
```

Its attributes:

- [`#[async_trait]`](../../macros/async_trait.md) — rewrites the `async fn` into a method returning `-> impl Future`, the lint-clean, allocation-free form.
- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `Handler` that implementations target and the wiring key `HandlerComponent`, while `CanHandle` stays the consumer trait callers use.
- [`#[prefix]`](../../macros/cgp_namespace.md) — registers the generated names into the `@cgp.extra.handler` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the dispatching providers a context routes through: `UseDelegate<Code>` dispatches on the `Code` tag, and `UseInputDelegate<Input>` on the `Input` type; the `open` statement is the modern sugar for the `Code` dispatch.
- [`#[use_type]`](../../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`Handler` and its consumer trait `CanHandle` are imported from `cgp::extra::handler`. The consumer
method is `async` and returns a `Result` against the context's error type:

```rust
async fn handle(&self, _tag: PhantomData<Code>, input: Input) -> Result<Self::Output, Error>;
```

A context gains the capability by wiring `HandlerComponent` to a provider. In practice most `Handler`
implementations are not written directly but produced by *promoting* a simpler provider: a function
written as a [`#[cgp_computer]`](../../macros/cgp_computer.md) or [`#[cgp_producer]`](../../macros/cgp_producer.md)
wires the promotion table so the same function answers `CanHandle` too, and the
[handler combinators](../../providers/handler/index.md) lift the narrowest fitting variant to a handler
wherever one is required.

The component dispatches on both the `Code` tag and the `Input` type, so wiring can route different
codes or inputs to different handlers, as covered under [dispatching](/docs/concepts/dispatching). Its
by-reference sibling [`HandlerRef`](./handler_ref.md), documented on its own page, is identical except
that its `handle_ref` method borrows the input as `&Input`.

## Examples

A generic consumer that bounds its context by `CanHandle` accepts any wired computation, whatever its
underlying capabilities:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::CanHandle;

async fn run_with<Context, Code>(
    context: &Context,
    input: String,
) -> Result<Context::Output, Context::Error>
where
    Context: CanHandle<Code, String>,
{
    context.handle(PhantomData::<Code>, input).await
}
```

`run_with` works for any context that wires a handler for the given `Code` and `String` input, whether
the wired provider is a pure `Computer`, a fallible `TryComputer`, an `AsyncComputer`, or a genuine
`Handler`, because the promotion combinators make each of them satisfy `CanHandle`. This is why generic
pipeline code targets `Handler`. The example is **parameter-targeted**: the computation acts on the
`Input` type, while the context decides which provider answers.

## When to reach for it, and when not

**Bound generic pipeline code against `CanHandle` when it should accept any computation regardless of
which capabilities the provider actually uses.** Because every simpler member promotes up to a handler,
a consumer written against `Handler` is the most reusable, and it is the right target for I/O steps,
request handlers, and composed pipelines.

Do not *write* a `Handler` provider by hand when a simpler variant fits: implement a
[`Computer`](./computer.md) for a pure transform, a [`TryComputer`](./try_computer.md) for a fallible
synchronous one, or a [`Producer`](./producer.md) for an input-free one, and let the wiring promote it.
Writing the narrowest variant keeps the provider reusable in every position, where a `Handler` can only
be used where a full handler is wanted.

## Related constructs

- [`HandlerRef`](./handler_ref.md) — the by-reference variant that borrows its input.
- [`Computer`](./computer.md) — the pure synchronous member; promotes up to `Handler`.
- [`TryComputer`](./try_computer.md) — the fallible synchronous member; promotes up to `Handler`.
- [`AsyncComputer`](./async_computer.md) — the async infallible member; promotes up to `Handler`.
- [`Producer`](./producer.md) — the input-free member of the family.
- [`HasErrorType`](../has_error_type.md) — the supertrait supplying the `Self::Error` a handler returns.
- [Handler combinators](../../providers/handler/index.md) — compose handlers and promote simpler variants
  into them.
- [`#[cgp_computer]`](../../macros/cgp_computer.md) / [`#[cgp_producer]`](../../macros/cgp_producer.md) — wire
  a function so it answers `CanHandle` through promotion.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Monadic handlers](/docs/concepts/monadic-handlers) — chaining handlers that short-circuit through a
  monad.
- [Recovering `Send` bounds](/docs/concepts/send-bounds) — restoring the `Send` guarantee an async trait
  method drops.

## Source

- `Handler` and `HandlerRef`:
  [`handler.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/handler.rs)
- The `ReturnInput` provider and the promotion combinators:
  [`cgp-handler/src/providers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-handler/src/)
- Re-exported through `cgp::extra::handler`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
