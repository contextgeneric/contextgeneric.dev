---
sidebar_label: 'ReturnError'
sidebar_position: 2
---

# `ReturnError`

Raise a source error that already is the context's abstract error, so raising is the identity.

## Overview

`ReturnError` is the `ErrorRaiser` provider for the case where the source error is exactly the abstract
error the **context** chose, where the context is the type a capability runs against. Raising then has
nothing to convert: it returns its argument untouched. A context uses it when generic code
raises a value that is already of the context's own error type. Like every CGP provider, `ReturnError`
carries no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorRaiserComponent` from
`cgp::core::error`. It takes no type parameter, and it type-checks only for the one source type that
equals the context's `Error`:

```rust
use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::ReturnError;

delegate_components! {
    App {
        ErrorRaiserComponent: ReturnError,
    }
}
```

## Examples

`ReturnError` most often sits in a per-source table as the entry for the context's own error type,
beside other raisers for the foreign types. If `App` chose `AppError` as its error, this entry raises an
`AppError` source by returning it and converts other sources elsewhere:

```rust
use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::{DebugError, RaiseFrom, ReturnError};

delegate_components! {
    App {
        open ErrorRaiserComponent;

        @ErrorRaiserComponent.AppError: ReturnError,
        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,
    }
}
```

## When to use it

**Reach for `ReturnError` when generic code raises a value that is already the context's error type.** A
`From<AppError> for AppError` impl exists in the standard library, so [`RaiseFrom`](raise_from.md) would
also work here; `ReturnError` states the intent directly and needs no `From` bound.

For a source the error can absorb through `From`, use [`RaiseFrom`](raise_from.md). For a printable
source with no `From` impl, use [`DebugError`](debug_error.md) or [`DisplayError`](display_error.md).

## Under the hood

`ReturnError` implements `ErrorRaiser<Context, E>` only when the context's `Error` is exactly `E`:

```rust
#[cgp_new_provider]
impl<Context, E> ErrorRaiser<Context, E> for ReturnError
where
    Context: HasErrorType<Error = E>,
{
    fn raise_error(e: E) -> E {
        e
    }
}
```

The `HasErrorType<Error = E>` bound ties the source type to the abstract error, so `raise_error` returns
its argument. The generated [`IsProviderFor`](../../traits/is_provider_for.md) impl carries the same
bound.

## Related constructs

- [`CanRaiseError`](../../components/can_raise_error.md) — the component `ReturnError` supplies.
- [`RaiseFrom`](raise_from.md) — converts through `From` rather than requiring an exact match.
- [`RaiseInfallible`](raise_infallible.md) — the raiser for `Infallible`.
- [`UseDelegate`](../use_delegate.md) — dispatches `ReturnError` per source-error type.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — the error type and its construction
  as separate wiring decisions.

## Source

- [`return_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/return_error.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
