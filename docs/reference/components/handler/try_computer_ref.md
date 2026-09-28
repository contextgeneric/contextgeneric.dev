---
title: 'TryComputerRef — fallible, borrowed input'
description: 'The handler-family component for a synchronous computation that can fail and only reads its input, returning a Result in the context''s error type.'
sidebar_label: 'TryComputerRef'
sidebar_position: 7
---

# `TryComputerRef`

The by-reference member of the handler family's fallible corner: a
[`TryComputer`](./try_computer.md) that borrows its input.

## Overview

`TryComputerRef` is a [`TryComputer`](./try_computer.md) that takes its input by reference. A
synchronous computation that can fail and only *reads* its argument fits it: its method receives
`&Input` and returns `Result<Output, Error>`, where `Error` is the abstract error type of the
[**context**](/docs/reference/glossary#context), the type the implementation runs against. It
differs from `TryComputer` on the input axis alone.

Because it can fail, it has [`HasErrorType`](../has_error_type.md) as a
[supertrait](/docs/reference/glossary#supertrait), which supplies the `Error` it names in its
`Result` and ties it to the same error every other fallible component in the context uses. See the
[handler family overview](./index.md) for how the members relate and promote.

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
- [`#[prefix]`](../../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.extra.handler`, so a context that joins that namespace binds its provider at `@cgp.extra.handler.TryComputerRefComponent` rather than at the bare key.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the legacy `UseDelegate` and `UseInputDelegate` providers, which dispatch on the `Code` tag and on the `Input` type through an inner table; the `open` statement replaces both, dispatching on either parameter or on the pair.
- [`#[use_type]`](../../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`TryComputerRefComponent` is in the prelude. The provider trait `TryComputerRef` and the consumer
trait `CanTryComputeRef` are not, and are imported from `cgp::extra::handler`. The method borrows
the input and returns a `Result`:

```rust
fn try_compute_ref(&self, _code: PhantomData<Code>, input: &Input) -> Result<Self::Output, Error>;
```

A context gains the operation by wiring `TryComputerRefComponent` to a provider and by wiring an
error type, and the component dispatches on both the `Code` tag and the `Input` type. Besides a
hand-written provider, the [`PromoteRef`](../../providers/handler/promote_ref.md) combinator answers
it from a [`TryComputer`](./try_computer.md) written for `&'a Input` at every lifetime, and in the
other direction lets a `TryComputerRef` provider answer `try_compute` for an owned input that
dereferences to its `Input`. See [`ComputerRef`](./computer_ref.md#usage) for the two directions,
which work the same way here.

## Examples

A provider that parses a port number from a borrowed setting, raising the parse error into the
context's error:

```rust
use core::marker::PhantomData;
use core::num::ParseIntError;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::RaiseFrom;
use cgp::extra::handler::{CanTryComputeRef, TryComputerRef};

#[cgp_impl(new ParsePort)]
#[uses(CanRaiseError<ParseIntError>)]
#[use_type(HasErrorType.Error)]
impl<Code> TryComputerRef<Code, String> {
    type Output = u16;

    fn try_compute_ref(
        &self,
        _code: PhantomData<Code>,
        input: &String,
    ) -> Result<Self::Output, Error> {
        input.trim().parse().map_err(Self::raise_error)
    }
}

#[derive(Debug, PartialEq)]
pub struct AppError(pub String);

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError(e.to_string())
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<AppError>,
        ErrorRaiserComponent: RaiseFrom,
        TryComputerRefComponent: ParsePort,
    }
}

check_components! {
    App {
        TryComputerRefComponent: ((), String),
    }
}

pub fn demo() {
    let setting = " 8080 ".to_owned();

    assert_eq!(App.try_compute_ref(PhantomData::<()>, &setting), Ok(8080));
    assert!(App.try_compute_ref(PhantomData::<()>, &"http".to_owned()).is_err());
}
```

`try_compute_ref` reads `setting` without taking it, so the caller still owns it afterwards. `App`
is an [environmental context](/docs/reference/glossary#environmental-context), and the component is
**[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**: the computation
acts on the `Input`, while `App` decides the provider and the error type.

## When to use it

**Reach for `TryComputerRef` when a fallible synchronous computation only needs to read its input.**
It keeps ownership with the caller while still reporting failure through the context's error type.

Reach for [`TryComputer`](./try_computer.md) instead when the computation takes the input by value,
for [`ComputerRef`](./computer_ref.md) when a borrowed-input computation cannot fail, and for
[`HandlerRef`](./handler_ref.md) when it must also await.

## Related constructs

- [`TryComputer`](./try_computer.md) — the owned-input counterpart.
- [`ComputerRef`](./computer_ref.md) — the infallible by-reference computer.
- [`HandlerRef`](./handler_ref.md) — the async and fallible by-reference member.
- [`HasErrorType`](../has_error_type.md) — the supertrait supplying the `Error` this returns.
- [`CanRaiseError`](../can_raise_error.md) — converts a concrete source error into that abstract
  error.
- [`PromoteRef`](../../providers/handler/promote_ref.md) — bridges the owned and borrowed forms.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Modular error handling](/docs/concepts/modular-error-handling) — the abstract error a fallible
  computer returns.

## Source

- `TryComputer` and `TryComputerRef`:
  [`try_compute.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/try_compute.rs)
- `PromoteRef`:
  [`promote_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_ref.rs)
- Re-exported through `cgp::extra::handler`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
