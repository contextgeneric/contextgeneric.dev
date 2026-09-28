---
title: 'HandlerRef — async and fallible, borrowed'
description: 'The handler-family component for an async computation that can fail and only reads its input, which its async method takes as &Input.'
sidebar_label: 'HandlerRef'
sidebar_position: 8
---

# `HandlerRef`

The by-reference member of the handler family's general corner: a [`Handler`](./handler.md) that
borrows its input.

## Overview

`HandlerRef` is a [`Handler`](./handler.md) that takes its input by reference. It is asynchronous
and fallible like the general handler, so it awaits and may return the abstract error of the
[**context**](/docs/reference/glossary#context), the type the implementation runs against, but its
method receives `&Input` where `CanHandle` receives `Input`. It suits an async, fallible computation
that reads its argument rather than consuming it, such as a request handler that inspects a borrowed
request.

Because it can fail, it has [`HasErrorType`](../has_error_type.md) as a
[supertrait](/docs/reference/glossary#supertrait), which supplies the `Error` it names in its
`Result`. See the [handler family overview](./index.md) for how the members relate and promote.

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

- [`#[async_trait]`](../../macros/async_trait.md) — rewrites the `async fn` into a method returning `-> impl Future`, the lint-clean, allocation-free form, which adds no `Send` bound to the future.
- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `HandlerRef` that implementations target and the wiring key `HandlerRefComponent`, while `CanHandleRef` stays the consumer trait callers use.
- [`#[prefix]`](../../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.extra.handler`, so a context that joins that namespace binds its provider at `@cgp.extra.handler.HandlerRefComponent` rather than at the bare key.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the legacy `UseDelegate` and `UseInputDelegate` providers, which dispatch on the `Code` tag and on the `Input` type through an inner table; the `open` statement replaces both, dispatching on either parameter or on the pair.
- [`#[use_type]`](../../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`HandlerRefComponent` is in the prelude. The provider trait `HandlerRef` and the consumer trait
`CanHandleRef` are not, and are imported from `cgp::extra::handler`. The method is `async`, borrows
the input, and returns a `Result`:

```rust
async fn handle_ref(&self, _tag: PhantomData<Code>, input: &Input) -> Result<Self::Output, Error>;
```

A context gains the operation by wiring `HandlerRefComponent` to a provider and by wiring an error
type, and the component dispatches on both the `Code` tag and the `Input` type. Besides a
hand-written provider, the [`PromoteRef`](../../providers/handler/promote_ref.md) combinator answers
it from a `Handler` written for `&'a Input` at every lifetime, which is the `HandlerRefComponent`
entry of every promotion bundle. In the other direction it lets a `HandlerRef` provider answer
`handle` for an owned input that dereferences to its `Input`. See
[`ComputerRef`](./computer_ref.md#usage) for the two directions, which work the same way here.

## Examples

An async, fallible provider over a borrowed request, wired into a context and awaited:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::RaiseFrom;
use cgp::extra::handler::{CanHandleRef, HandlerRef};

#[cgp_impl(new NonEmptyLength)]
#[uses(CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl<Code> HandlerRef<Code, String> {
    type Output = usize;

    async fn handle_ref(
        &self,
        _code: PhantomData<Code>,
        input: &String,
    ) -> Result<Self::Output, Error> {
        if input.is_empty() {
            return Err(Self::raise_error("empty request".to_owned()));
        }

        Ok(input.len())
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ErrorRaiserComponent: RaiseFrom,
        HandlerRefComponent: NonEmptyLength,
    }
}

check_components! {
    App {
        HandlerRefComponent: ((), String),
    }
}

pub async fn demo() {
    let request = "GET /".to_owned();

    assert_eq!(App.handle_ref(PhantomData::<()>, &request).await, Ok(5));
    assert_eq!(
        App.handle_ref(PhantomData::<()>, &String::new()).await,
        Err("empty request".to_owned())
    );
}
```

`NonEmptyLength` raises a `String` message into the context's error, which `App` sets to `String`,
so [`RaiseFrom`](../../providers/error/raise_from.md) raises it through the identity `From` impl.
The caller keeps ownership of `request`. `App` is an [environmental
context](/docs/reference/glossary#environmental-context), and the component is
**[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**: the computation
acts on the `Input`, while `App` decides the provider and the error type.

## When to use it

**Reach for `HandlerRef` when an async, fallible computation only needs to read its input.** Generic
code bounded by `CanHandleRef` accepts a provider for any by-reference member that is lifted into
it, through a promotion bundle or a hand-written chain of promotions.

Reach for [`Handler`](./handler.md) instead when the computation takes the input by value, which is
the more common case, and for [`TryComputerRef`](./try_computer_ref.md) when a borrowed-input
fallible computation is synchronous. Do not write a `HandlerRef` by hand when a simpler member fits:
implement that and let the wiring promote it.

## Related constructs

- [`Handler`](./handler.md) — the owned-input counterpart, and the family's general corner.
- [`TryComputerRef`](./try_computer_ref.md) — the synchronous fallible by-reference computer.
- [`AsyncComputerRef`](./async_computer_ref.md) — the infallible async by-reference computer.
- [`HasErrorType`](../has_error_type.md) — the supertrait supplying the `Error` this returns.
- [`PromoteRef`](../../providers/handler/promote_ref.md) — bridges `Handler` and `HandlerRef`.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Recovering `Send` bounds](/docs/concepts/send-bounds) — restoring the `Send` guarantee the async
  method does not carry.

## Source

- `Handler` and `HandlerRef`:
  [`handler.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/handler.rs)
- `PromoteRef`:
  [`promote_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_ref.rs)
- Re-exported through `cgp::extra::handler`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
