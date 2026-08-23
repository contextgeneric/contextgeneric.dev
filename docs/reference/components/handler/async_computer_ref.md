---
sidebar_label: 'AsyncComputerRef'
sidebar_position: 9
---

# `AsyncComputerRef`

The async, by-reference member of the handler family: an [`AsyncComputer`](./async_computer.md) that
borrows its input.

## Overview

`AsyncComputerRef` combines two of the family's axes at once: it is asynchronous like
[`AsyncComputer`](./async_computer.md) and takes its input by reference like
[`ComputerRef`](./computer_ref.md). A computation that must await and only *reads* its argument, while
still never failing, fits it: its method is `async`, receives `&Input`, and returns the `Output` directly
with no failure path. It runs against a **context**, the type a capability runs against that supplies the
values an implementation needs as its own fields.

It is the async by-reference corner of the infallible computers. Like the other computers it never names
an error type, so it does not supertrait [`HasErrorType`](../has_error_type.md). See the
[handler family overview](./index.md) for how the members relate and promote.

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

- [`#[async_trait]`](../../macros/async_trait.md) — rewrites the `async fn` into a method returning `-> impl Future`, the lint-clean, allocation-free form.
- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `AsyncComputerRef` that implementations target and the wiring key `AsyncComputerRefComponent`, while `CanComputeAsyncRef` stays the consumer trait callers use.
- [`#[prefix]`](../../macros/cgp_namespace.md) — registers the generated names into the `@cgp.extra.handler` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the dispatching providers a context routes through: `UseDelegate<Code>` dispatches on the `Code` tag, and `UseInputDelegate<Input>` on the `Input` type; the `open` statement is the modern sugar for the `Code` dispatch.

## Usage

`AsyncComputerRef` and its consumer trait `CanComputeAsyncRef` are imported from `cgp::extra::handler`.
The method is `async` and borrows the input:

```rust
async fn compute_async_ref(&self, _code: PhantomData<Code>, input: &Input) -> Self::Output;
```

A context gains the capability by wiring `AsyncComputerRefComponent` to a provider, and it dispatches on
both the `Code` tag and the `Input` type. It is the least commonly wired member of the family, reached
when a computation is at once async, infallible, and read-only over its input. A simpler variant is
usually promoted into it through the [handler combinators](../../providers/handler/index.md).

## Examples

A generic consumer awaits an infallible computation over a borrowed input:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::CanComputeAsyncRef;

async fn scan<Context, Code>(context: &Context, input: &String) -> Context::Output
where
    Context: CanComputeAsyncRef<Code, String>,
{
    context.compute_async_ref(PhantomData::<Code>, input).await
}
```

`scan` awaits the computation while keeping ownership of the `String` with the caller. The example is
**parameter-targeted**: the computation acts on the `Input`, while the context decides the provider.

## When to use it

**Reach for `AsyncComputerRef` when a computation is async and infallible and only reads its input.**
That is a narrow combination, so most code reaches a simpler member first and lets promotion bridge to
this one.

Reach for [`AsyncComputer`](./async_computer.md) when the computation takes the input by value, for
[`ComputerRef`](./computer_ref.md) when a borrowed-input computation needs no awaiting, and for
[`HandlerRef`](./handler_ref.md) when a borrowed-input async computation can also fail.

## Related constructs

- [`AsyncComputer`](./async_computer.md) — the owned-input counterpart.
- [`ComputerRef`](./computer_ref.md) — the synchronous by-reference computer.
- [`HandlerRef`](./handler_ref.md) — the fallible async by-reference member.
- [Handler combinators](../../providers/handler/index.md) — promote a simpler variant into this.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.

## Source

- `AsyncComputer` and `AsyncComputerRef`:
  [`async_computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/async_computer.rs)
- Re-exported through `cgp::extra::handler`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
