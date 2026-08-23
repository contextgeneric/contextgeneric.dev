---
sidebar_label: 'ComputerRef'
sidebar_position: 6
---

# `ComputerRef`

The by-reference member of the handler family: a [`Computer`](./computer.md) that borrows its input.

## Overview

`ComputerRef` is a [`Computer`](./computer.md) that takes its input by reference instead of by value. A
synchronous, infallible computation that only *reads* its argument and should not take ownership of it
fits `ComputerRef`, whose method receives `&Input` where `CanCompute` receives `Input`. Everything else
is the same: it transforms toward an `Output` under a phantom `Code` tag, against a **context** (the type
a capability runs against, which supplies the values an implementation needs as its own fields), and
returns the `Output` directly with no failure path.

It is the owned-versus-borrowed variant of the pure computer, differing on the input axis alone. Like
`Computer` it never names an error type, so it does not supertrait
[`HasErrorType`](../has_error_type.md). See the [handler family overview](./index.md) for how the members
relate.

## Definition

`CanComputeRef` is defined as:

```rust
#[cgp_component(ComputerRef)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
pub trait CanComputeRef<Code, Input> {
    type Output;

    fn compute_ref(&self, _code: PhantomData<Code>, input: &Input) -> Self::Output;
}
```

Its attributes:

- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `ComputerRef` that implementations target and the wiring key `ComputerRefComponent`, while `CanComputeRef` stays the consumer trait callers use.
- [`#[prefix]`](../../macros/cgp_namespace.md) — registers the generated names into the `@cgp.extra.handler` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the dispatching providers a context routes through: `UseDelegate<Code>` dispatches on the `Code` tag, and `UseInputDelegate<Input>` on the `Input` type; the `open` statement is the modern sugar for the `Code` dispatch.

## Usage

`ComputerRef` and its consumer trait `CanComputeRef` are imported from `cgp::extra::handler`. The method
borrows the input:

```rust
fn compute_ref(&self, _code: PhantomData<Code>, input: &Input) -> Self::Output;
```

A context gains the capability by wiring `ComputerRefComponent` to a provider, and it dispatches on both
the `Code` tag and the `Input` type. A provider that reads rather than consumes its input reaches for
this variant; a [`Computer`](./computer.md) can also be bridged to it by the
[`PromoteRef`](../../providers/handler/index.md) combinator, which supplies an owned input by
dereferencing the borrow. Its async counterpart is [`AsyncComputerRef`](./async_computer_ref.md).

## Examples

A generic consumer computes from a borrowed input:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::CanComputeRef;

fn measure<Context, Code>(context: &Context, input: &String) -> Context::Output
where
    Context: CanComputeRef<Code, String>,
{
    context.compute_ref(PhantomData::<Code>, input)
}
```

`measure` passes the input by reference, so the caller keeps ownership of the `String`. The example is
**parameter-targeted**: the computation acts on the `Input`, while the context decides the provider.

## When to use it

**Reach for `ComputerRef` when a synchronous, infallible computation only needs to read its input.** It
avoids handing ownership to a computation that does not consume the value, which matters when the caller
reuses the input afterwards.

Reach for [`Computer`](./computer.md) instead when the computation takes the input by value, which is the
more common case, and for [`AsyncComputerRef`](./async_computer_ref.md) when a borrowed-input computation
must also await. If the computation can fail, use [`TryComputer`](./try_computer.md) or its by-reference
sibling [`TryComputerRef`](./try_computer_ref.md).

## Related constructs

- [`Computer`](./computer.md) — the owned-input counterpart.
- [`AsyncComputerRef`](./async_computer_ref.md) — the async version of this by-reference variant.
- [`TryComputerRef`](./try_computer_ref.md) — the fallible by-reference computer.
- [Handler combinators](../../providers/handler/index.md) — the `PromoteRef` combinator bridging owned
  and borrowed inputs.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.

## Source

- `Computer` and `ComputerRef`:
  [`computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/computer.rs)
- Re-exported through `cgp::extra::handler`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
