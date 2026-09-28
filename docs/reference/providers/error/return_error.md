---
title: 'ReturnError — raise the context''s own error'
description: 'The error raiser for a source that already is the context''s error type: raising returns the value unchanged, with no From bound.'
sidebar_label: 'ReturnError'
sidebar_position: 2
---

# `ReturnError`

Raise a source error that already is the context's abstract error, so raising is the identity.

## Overview

`ReturnError` is the `ErrorRaiser` provider for the case where the source error is exactly the
abstract error the [**context**](/docs/reference/glossary#context) chose, where the context is the
type the implementation runs against. Raising then has nothing to convert: it returns its argument
untouched. A context uses it when generic code raises a value that is already of the context's own
error type. Like every CGP provider, `ReturnError` carries no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorRaiserComponent` from
`cgp::core::error`; neither is in the prelude. It takes no type parameter, and it type-checks only for
the one source type that equals the context's `Error`:

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
beside other raisers for the foreign types:

```rust
use core::num::ParseIntError;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::{DebugError, RaiseFrom, ReturnError};

#[derive(Debug, PartialEq)]
pub struct AppError {
    pub message: String,
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        AppError { message }
    }
}

#[cgp_component(PortChecker)]
#[use_type(HasErrorType.Error)]
pub trait CanCheckPort {
    fn check_port(&self, raw: &str) -> Result<u16, Error>;
}

#[cgp_impl(new CheckPort)]
#[uses(CanRaiseError<AppError>, CanRaiseError<ParseIntError>)]
#[use_type(HasErrorType.{Error = AppError})]
impl PortChecker {
    fn check_port(&self, raw: &str) -> Result<u16, AppError> {
        let port: u16 = raw.parse().map_err(Self::raise_error)?;

        if port == 0 {
            return Err(Self::raise_error(AppError {
                message: "port 0 is reserved".to_owned(),
            }));
        }

        Ok(port)
    }
}

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<AppError>,
        PortCheckerComponent: CheckPort,

        @ErrorRaiserComponent.AppError: ReturnError,
        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,
    }
}

check_components! {
    App {
        PortCheckerComponent,
    }
}

pub fn demo() {
    assert_eq!(App.check_port("8080"), Ok(8080));
    assert_eq!(
        App.check_port("0"),
        Err(AppError { message: "port 0 is reserved".to_owned() })
    );
    assert!(App.check_port("http").unwrap_err().message.contains("ParseIntError"));
}
```

`CheckPort` pins the error to `AppError` with `#[use_type]`'s equality form, so it can construct one
and raise it. `App` is an [environmental context](/docs/reference/glossary#environmental-context)
whose table routes each source: the `AppError` through `ReturnError`, and a `ParseIntError` through
[`DebugError`](debug_error.md), which formats it and forwards the resulting `String` to the `String`
entry, where `RaiseFrom` converts it through `AppError`'s `From<String>`.

## When to use it

**Reach for `ReturnError` when generic code raises a value that is already the context's error type.**
The standard library's reflexive `impl<T> From<T> for T` means [`RaiseFrom`](raise_from.md) would also
work here; `ReturnError` states the intent directly and needs no `From` bound.

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

The `HasErrorType<Error = E>` bound ties the source type to the abstract error, so `raise_error`
returns its argument. The generated [`IsProviderFor`](../../traits/wiring/is_provider_for.md) impl
carries the same bound.

## Related constructs

- [`CanRaiseError`](../../components/can_raise_error.md) — the component `ReturnError` supplies.
- [`RaiseFrom`](raise_from.md) — converts through `From` rather than requiring an exact match.
- [`RaiseInfallible`](raise_infallible.md) — the raiser for `Infallible`.
- [`delegate_components!`](../../macros/delegate_components.md) — its `open` statement dispatches
  raisers per source type, as the example does.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — the error type and its construction
  as separate wiring decisions.

## Source

- [`return_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/return_error.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
