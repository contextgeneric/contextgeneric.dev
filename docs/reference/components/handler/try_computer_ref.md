---
sidebar_label: 'TryComputerRef'
sidebar_position: 7
---

# `TryComputerRef`

The by-reference member of the handler family's fallible corner: a [`TryComputer`](./try_computer.md)
that borrows its input.

## Overview

`TryComputerRef` is a [`TryComputer`](./try_computer.md) that takes its input by reference. A synchronous
computation that can fail and only *reads* its argument fits it: its method receives `&Input` and returns
`Result<Output, Error>`, where the error is the **context's** abstract error type. The context is the
type a capability runs against, which supplies the values an implementation needs as its own fields.
`TryComputerRef` is the owned-versus-borrowed variant of the fallible synchronous computer, differing on
the input axis alone.

Because it can fail, it supertraits [`HasErrorType`](../has_error_type.md), which supplies the
`Self::Error` it names in its `Result` and ties it to the same error every other fallible component in
the context uses. See the [handler family overview](./index.md) for how the members relate and promote.

## Definition

`CanTryComputeRef` is defined as:

```rust
#[cgp_component(TryComputerRef)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
#[use_type(HasErrorType.Error)]
pub trait CanTryComputeRef<Code, Input> {
    type Output;

    fn try_compute_ref(
        &self,
        _code: PhantomData<Code>,
        input: &Input,
    ) -> Result<Self::Output, Error>;
}
```

Its attributes:

- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `TryComputerRef` that implementations target and the wiring key `TryComputerRefComponent`, while `CanTryComputeRef` stays the consumer trait callers use.
- [`#[prefix]`](../../macros/cgp_namespace.md) — registers the generated names into the `@cgp.extra.handler` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the dispatching providers a context routes through: `UseDelegate<Code>` dispatches on the `Code` tag, and `UseInputDelegate<Input>` on the `Input` type; the `open` statement is the modern sugar for the `Code` dispatch.
- [`#[use_type]`](../../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`TryComputerRef` and its consumer trait `CanTryComputeRef` are imported from `cgp::extra::handler`. The
method borrows the input and returns a `Result`:

```rust
fn try_compute_ref(&self, _code: PhantomData<Code>, input: &Input) -> Result<Self::Output, Error>;
```

A context gains the capability by wiring `TryComputerRefComponent` to a provider, and it dispatches on
both the `Code` tag and the `Input` type. Because the consumer trait supertraits `HasErrorType`, the
context must also wire an error type before it can call `try_compute_ref`. A provider that reads rather
than consumes its input reaches for this variant; the owned-input [`TryComputer`](./try_computer.md) is
the more common one.

## Examples

A generic consumer runs a fallible computation over a borrowed input:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::CanTryComputeRef;

fn check<Context, Code>(
    context: &Context,
    input: &String,
) -> Result<Context::Output, Context::Error>
where
    Context: CanTryComputeRef<Code, String>,
{
    context.try_compute_ref(PhantomData::<Code>, input)
}
```

`check` passes the input by reference, so the caller keeps the `String`, and returns the context's
abstract error on failure. The example is **parameter-targeted**: the computation acts on the `Input`,
while the context decides the provider and the error type.

## When to use it

**Reach for `TryComputerRef` when a fallible synchronous computation only needs to read its input.** It
keeps ownership with the caller while still reporting failure through the context's error type.

Reach for [`TryComputer`](./try_computer.md) instead when the computation takes the input by value, for
[`ComputerRef`](./computer_ref.md) when a borrowed-input computation cannot fail, and for
[`Handler`](./handler.md) or [`HandlerRef`](./handler_ref.md) when it must also await.

## Related constructs

- [`TryComputer`](./try_computer.md) — the owned-input counterpart.
- [`ComputerRef`](./computer_ref.md) — the infallible by-reference computer.
- [`HandlerRef`](./handler_ref.md) — the async-and-fallible by-reference member.
- [`HasErrorType`](../has_error_type.md) — the supertrait supplying the `Self::Error` this returns.
- [`CanRaiseError`](../can_raise_error.md) — converts a concrete source error into that abstract error.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Modular error handling](/docs/concepts/modular-error-handling) — the abstract error a fallible
  computer returns.

## Source

- `TryComputer` and `TryComputerRef`:
  [`try_compute.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/try_compute.rs)
- Re-exported through `cgp::extra::handler`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
