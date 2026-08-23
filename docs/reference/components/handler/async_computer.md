---
sidebar_label: 'AsyncComputer'
sidebar_position: 5
---

# `AsyncComputer`

The asynchronous, infallible member of the handler family: a [`Computer`](./computer.md) that awaits.

## Overview

`AsyncComputer` is for computations that must await but still cannot fail. Reading a value that is
already in memory is a [`Computer`](./computer.md); awaiting a timer or a channel that always yields is
an `AsyncComputer`. It is the async point on the family's synchronicity axis with the failure path still
absent, so it transforms an `Input` into an `Output` under a phantom `Code` tag, against a **context**
(the type a capability runs against, which supplies the values an implementation needs as its own
fields), and returns the `Output` directly rather than a `Result`.

It sits between [`Computer`](./computer.md), which drops the asynchrony, and [`Handler`](./handler.md),
which adds a failure path on top of the asynchrony. Like the pure computer, it never names an error type,
so it does not supertrait [`HasErrorType`](../has_error_type.md). See the
[handler family overview](./index.md) for how the members relate and promote.

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

- [`#[async_trait]`](../../macros/async_trait.md) — rewrites the `async fn` into a method returning `-> impl Future`, the lint-clean, allocation-free form.
- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `AsyncComputer` that implementations target and the wiring key `AsyncComputerComponent`, while `CanComputeAsync` stays the consumer trait callers use.
- [`#[prefix]`](../../macros/cgp_namespace.md) — registers the generated names into the `@cgp.extra.handler` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the dispatching providers a context routes through: `UseDelegate<Code>` dispatches on the `Code` tag, and `UseInputDelegate<Input>` on the `Input` type; the `open` statement is the modern sugar for the `Code` dispatch.

## Usage

`AsyncComputer` and its consumer trait `CanComputeAsync` are imported from `cgp::extra::handler`. The
method is `async` and takes the input by value:

```rust
async fn compute_async(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
```

A context gains the capability by wiring `AsyncComputerComponent` to a provider, and it dispatches on
both the `Code` tag and the `Input` type. In everyday code the provider comes from
[`#[cgp_computer]`](../../macros/cgp_computer.md), which wires the promotion table so a synchronous
function also answers `CanComputeAsync`, and the crate ships a [`UseField`](../../providers/use_field.md)
provider that forwards the async computation to a field of the context. Its by-reference sibling is
[`AsyncComputerRef`](./async_computer_ref.md).

## Examples

A generic consumer awaits an async computation wired on its context:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::CanComputeAsync;

async fn run<Context, Code>(context: &Context, input: u64) -> Context::Output
where
    Context: CanComputeAsync<Code, u64>,
{
    context.compute_async(PhantomData::<Code>, input).await
}
```

`run` works for any context that wires an async computer for the given `Code` and `u64` input. The
example is **parameter-targeted**: the computation acts on the `Input`, while the context decides the
provider. A provider is usually generated from a function with
[`#[cgp_computer]`](../../macros/cgp_computer.md) rather than written by hand.

## When to reach for it, and when not

**Reach for `AsyncComputer` for a computation that awaits but cannot fail.** It is the async infallible
corner of the family, so a provider that only reads its input should prefer the by-reference
[`AsyncComputerRef`](./async_computer_ref.md).

Reach for [`Computer`](./computer.md) instead when nothing needs awaiting, since a synchronous computer
promotes into an `AsyncComputer` for free and stays usable in more positions. Reach for
[`Handler`](./handler.md) when the computation can also fail. Implement whichever single variant fits and
let the wiring promote it.

## Related constructs

- [`Computer`](./computer.md) — the synchronous counterpart; promotes into this.
- [`AsyncComputerRef`](./async_computer_ref.md) — the by-reference variant of this component.
- [`Handler`](./handler.md) — adds a failure path on top of the asynchrony.
- [`#[cgp_computer]`](../../macros/cgp_computer.md) — builds a provider that answers this through
  promotion.
- [Handler combinators](../../providers/handler/index.md) — promote a `Computer` into this.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.

## Source

- `AsyncComputer` and `AsyncComputerRef`:
  [`async_computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/async_computer.rs)
- Re-exported through `cgp::extra::handler`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
