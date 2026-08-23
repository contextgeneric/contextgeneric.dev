---
sidebar_label: 'HandlerRef'
sidebar_position: 8
---

# `HandlerRef`

The by-reference member of the handler family's general corner: a [`Handler`](./handler.md) that borrows
its input.

## Overview

`HandlerRef` is a [`Handler`](./handler.md) that takes its input by reference. It is asynchronous and
fallible like the general handler, so it awaits and may return the **context's** abstract error, but its
method receives `&Input` where `CanHandle` receives `Input`. The context is the type a capability runs
against, which supplies the values an implementation needs as its own fields. `HandlerRef` is the
owned-versus-borrowed variant of the family's most general member, differing on the input axis alone, for
an async-and-fallible computation that reads rather than consumes its argument.

Because it can fail, it supertraits [`HasErrorType`](../has_error_type.md), which supplies the
`Self::Error` it names in its `Result`. See the [handler family overview](./index.md) for how the members
relate and promote.

## Definition

`CanHandleRef` is defined as:

```rust
#[async_trait]
#[cgp_component(HandlerRef)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
#[use_type(HasErrorType.Error)]
pub trait CanHandleRef<Code, Input> {
    type Output;

    async fn handle_ref(
        &self,
        _tag: PhantomData<Code>,
        input: &Input,
    ) -> Result<Self::Output, Error>;
}
```

Its attributes:

- [`#[async_trait]`](../../macros/async_trait.md) — rewrites the `async fn` into a method returning `-> impl Future`, the lint-clean, allocation-free form.
- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `HandlerRef` that implementations target and the wiring key `HandlerRefComponent`, while `CanHandleRef` stays the consumer trait callers use.
- [`#[prefix]`](../../macros/cgp_namespace.md) — registers the generated names into the `@cgp.extra.handler` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the dispatching providers a context routes through: `UseDelegate<Code>` dispatches on the `Code` tag, and `UseInputDelegate<Input>` on the `Input` type; the `open` statement is the modern sugar for the `Code` dispatch.
- [`#[use_type]`](../../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`HandlerRef` and its consumer trait `CanHandleRef` are imported from `cgp::extra::handler`. The method is
`async`, borrows the input, and returns a `Result`:

```rust
async fn handle_ref(&self, _tag: PhantomData<Code>, input: &Input) -> Result<Self::Output, Error>;
```

A context gains the capability by wiring `HandlerRefComponent` to a provider, and it dispatches on both
the `Code` tag and the `Input` type. Because the consumer trait supertraits `HasErrorType`, the context
must also wire an error type. A [`Handler`](./handler.md) is bridged to `HandlerRef` by the
[`PromoteRef`](../../providers/handler/index.md) combinator, which supplies an owned input by dereferencing
the borrow, so most `HandlerRef` implementations come from promotion rather than being written by hand.

## Examples

A generic consumer awaits a fallible computation over a borrowed input:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::CanHandleRef;

async fn serve<Context, Code>(
    context: &Context,
    request: &String,
) -> Result<Context::Output, Context::Error>
where
    Context: CanHandleRef<Code, String>,
{
    context.handle_ref(PhantomData::<Code>, request).await
}
```

`serve` passes the request by reference, so the caller keeps ownership, and returns the context's
abstract error on failure. The example is **parameter-targeted**: the computation acts on the `Input`,
while the context decides the provider and the error type.

## When to reach for it, and when not

**Reach for `HandlerRef` when an async, fallible computation only needs to read its input**, for instance
a request handler that inspects a borrowed request without consuming it. Bounding generic pipeline code
against it accepts any provider that fits, since the simpler by-reference variants promote up to it.

Reach for [`Handler`](./handler.md) instead when the computation takes the input by value, which is the
more common case, and for [`TryComputerRef`](./try_computer_ref.md) when a borrowed-input fallible
computation is synchronous. Do not write a `HandlerRef` by hand when a simpler by-reference variant fits:
implement that and let the wiring promote it.

## Related constructs

- [`Handler`](./handler.md) — the owned-input counterpart, and the family's general corner.
- [`TryComputerRef`](./try_computer_ref.md) — the synchronous fallible by-reference computer.
- [`AsyncComputerRef`](./async_computer_ref.md) — the infallible async by-reference computer.
- [`HasErrorType`](../has_error_type.md) — the supertrait supplying the `Self::Error` this returns.
- [Handler combinators](../../providers/handler/index.md) — the `PromoteRef` combinator that bridges
  `Handler` and `HandlerRef`.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Recovering `Send` bounds](/docs/concepts/send-bounds) — restoring the `Send` guarantee an async trait
  method drops.

## Source

- `Handler` and `HandlerRef`:
  [`handler.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/handler.rs)
- Re-exported through `cgp::extra::handler`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
