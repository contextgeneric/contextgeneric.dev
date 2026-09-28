---
title: 'TryComputer — a fallible computation'
description: 'The handler-family component for a synchronous computation that can fail, returning a Result in the context''s abstract error type.'
sidebar_label: 'TryComputer'
sidebar_position: 2
---

# `TryComputer`

The fallible synchronous member of the handler family: a computer that can return an error.

## Overview

`TryComputer` is for synchronous computations that can fail. A computation that parses a string,
looks up a key, or checks an invariant may not be able to produce its output, and it needs a way to
report the failure. Where [`Computer`](./computer.md) returns its `Output` directly, `TryComputer`
returns `Result<Output, Error>`, where `Error` is the abstract error type of the
[**context**](/docs/reference/glossary#context), the type the implementation runs against. It sits
one step from `Computer` on the fallibility axis of the [handler family](/docs/concepts/handlers),
and one step from [`Handler`](./handler.md), which is also async.

Returning the *context's* abstract error rather than a concrete one keeps a `TryComputer` provider
generic. A provider does not commit to `anyhow::Error` or `std::io::Error`: it returns the bare
`Error`, and the context decides the concrete type when it wires one. This is why the component has
[`HasErrorType`](../has_error_type.md) as a [supertrait](/docs/reference/glossary#supertrait): the
supertrait supplies that `Error`, and every fallible component in a context names the same one, so
their results compose with `?`.

## Definition

`CanTryCompute` is defined as:

```rust
#[cgp_component(TryComputer)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
#[use_type(HasErrorType.Error)]
pub trait CanTryCompute<Code, Input> {
    type Output;

    fn try_compute(&self, _code: PhantomData<Code>, input: Input) -> Result<Self::Output, Error>;
}
```

Its attributes:

- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `TryComputer` that implementations target and the wiring key `TryComputerComponent`, while `CanTryCompute` stays the consumer trait callers use.
- [`#[prefix]`](../../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.extra.handler`, so a context that joins that namespace binds its provider at `@cgp.extra.handler.TryComputerComponent` rather than at the bare key.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the legacy `UseDelegate` and `UseInputDelegate` providers, which dispatch on the `Code` tag and on the `Input` type through an inner table; the `open` statement replaces both, dispatching on either parameter or on the pair.
- [`#[use_type]`](../../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`. The trait's own `Output` stays written as `Self::Output`.

## Usage

`TryComputer` and `TryComputerComponent` are in the prelude. The consumer trait `CanTryCompute` is
not, and is imported from `cgp::extra::handler`. The consumer method mirrors `compute` but returns a
`Result`:

```rust
fn try_compute(&self, _code: PhantomData<Code>, input: Input) -> Result<Self::Output, Error>;
```

A context gains the operation by wiring `TryComputerComponent` to a provider and by wiring an error
type, since the consumer trait has `HasErrorType` as a supertrait. A provider comes from one of
three places:

- **A hand-written provider** returns the context's `Error`, usually raising a concrete error into
  it with [`CanRaiseError`](../can_raise_error.md), as the example below does.
- **The promotion providers**, imported from `cgp::extra::handler`, answer it from a neighbouring
  member. [`Promote<P>`](../../providers/handler/promote.md) wraps the output of a
  [`Computer`](./computer.md) in `Ok`, and [`TryPromote<P>`](../../providers/handler/try_promote.md)
  reads a `Computer` whose `Output` is `Result<T, Error>` as success or failure. `TryPromote` also
  works the other way, presenting a `TryComputer` as a `Computer` whose `Output` is that `Result`.
- **[`#[cgp_computer]`](../../macros/cgp_computer.md)** on a function returning `Result<T, E>`
  generates a `Computer` whose `Output` is the `Result`, and wires it to the
  [`PromoteTryComputer`](../../providers/handler/promote_try_computer.md) bundle, whose
  `TryComputerComponent` entry is `TryPromote`. So `compute` returns the whole `Result`, while
  `try_compute` propagates its `Err`. The function's `E` must be the context's error type, because
  the bundle passes the `Err` through unconverted.

The component dispatches on both the `Code` tag and the `Input` type. Its by-reference sibling
[`TryComputerRef`](./try_computer_ref.md), documented on its own page, differs on one axis: its
`try_compute_ref` method borrows the input as `&Input`. The async and fallible counterpart is
[`Handler`](./handler.md), and [`PromoteAsync<P>`](../../providers/handler/promote_async.md) lifts a
`TryComputer` into one.

## Examples

A `TryComputer` provider that parses a string into a number, raising the parse error into the
context's error:

```rust
use core::marker::PhantomData;
use core::num::ParseIntError;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::RaiseFrom;
use cgp::extra::handler::CanTryCompute;

#[cgp_impl(new ParseU64)]
#[uses(CanRaiseError<ParseIntError>)]
#[use_type(HasErrorType.Error)]
impl<Code> TryComputer<Code, String> {
    type Output = u64;

    fn try_compute(
        &self,
        _code: PhantomData<Code>,
        input: String,
    ) -> Result<Self::Output, Error> {
        input.parse().map_err(Self::raise_error)
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
        TryComputerComponent: ParseU64,
    }
}

check_components! {
    App {
        TryComputerComponent: ((), String),
    }
}

pub fn demo() {
    assert_eq!(App.try_compute(PhantomData::<()>, "12".to_owned()), Ok(12));
    assert!(App.try_compute(PhantomData::<()>, "twelve".to_owned()).is_err());
}
```

`ParseU64` names neither the context nor its error type. `App` supplies both: its error type is
`AppError`, and [`RaiseFrom`](../../providers/error/raise_from.md) raises the `ParseIntError`
through `AppError`'s `From` impl. `App` is an [environmental
context](/docs/reference/glossary#environmental-context), and the component is
**[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**: the computation
acts on the `Input`, while `App` decides the provider and the error type.

## When to use it

**Reach for `TryComputer` for a synchronous computation that can fail.** It is the fallible middle
of the family: it receives providers promoted from a [`Computer`](./computer.md) and is itself
promoted into a [`Handler`](./handler.md). Prefer [`TryComputerRef`](./try_computer_ref.md) when the
computation only reads its input.

Reach for [`Computer`](./computer.md) instead when the computation cannot fail, since `Promote`
lifts a computer into a `TryComputer` and the computer stays usable in the infallible positions too,
and for [`Handler`](./handler.md) when the computation must also await. Write a `TryComputer`
provider by hand rather than a `#[cgp_computer]` function when the error has to be raised into the
context's error type, since the function's error type is passed through unconverted.

## Related constructs

- [`TryComputerRef`](./try_computer_ref.md) — the by-reference variant that borrows its input.
- [`Computer`](./computer.md) — the infallible counterpart; promotes into a `TryComputer`.
- [`Handler`](./handler.md) — the async and fallible member.
- [`HasErrorType`](../has_error_type.md) — the supertrait supplying the `Error` this returns.
- [`CanRaiseError`](../can_raise_error.md) — converts a concrete source error into that abstract
  error.
- [`#[cgp_computer]`](../../macros/cgp_computer.md) — builds a provider from a fallible function.
- [Handler combinators](../../providers/handler/index.md) — `Promote`, `TryPromote`, `PromoteAsync`,
  and the promotion bundles.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Modular error handling](/docs/concepts/modular-error-handling) — the abstract error a fallible
  computer returns.

## Source

- `TryComputer` and `TryComputerRef`:
  [`try_compute.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/try_compute.rs)
- `Promote`, `TryPromote`, and the promotion bundles:
  [`cgp-handler/src/providers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-handler/src/providers)
- Re-exported through `cgp::extra::handler`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
