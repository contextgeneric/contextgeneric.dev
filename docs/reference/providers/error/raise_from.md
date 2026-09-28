---
title: 'RaiseFrom — raise an error through From'
description: 'The error raiser that converts a source error into the context''s error type with From, so one wiring covers every source that error can absorb.'
sidebar_label: 'RaiseFrom'
sidebar_position: 1
---

# `RaiseFrom`

Raise a source error by converting it into the context's error type through the standard `From` trait.

## Overview

`RaiseFrom` is the `ErrorRaiser` provider for the common case: the abstract error of the
[**context**](/docs/reference/glossary#context), the type the implementation runs against, already
knows how to build itself from the source error through `From`. Wiring it means "convert every source
error the abstract error has a `From` impl for". It is the default choice whenever that `From` impl
exists, which covers most error raising in practice.

Because the bound is on the abstract error rather than on one source type, a single wiring of
`RaiseFrom` covers every source error the context's error can absorb. Like every CGP provider,
`RaiseFrom` carries no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorRaiserComponent` from
`cgp::core::error`; neither is in the prelude. It takes no type parameter:

```rust
use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::RaiseFrom;

delegate_components! {
    App {
        ErrorRaiserComponent: RaiseFrom,
    }
}
```

Wired this way, any provider that calls `Context::raise_error(source)` on `App` succeeds for every
`source` whose type the `App` error implements `From` for. A context whose error cannot absorb every
source wires `RaiseFrom` per source type instead, with the `open` statement, beside other raisers
such as [`DebugError`](debug_error.md).

## Examples

One `RaiseFrom` entry raises two different source types, each through its own `From` impl on the
context's `AppError`:

```rust
use core::num::ParseIntError;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::RaiseFrom;

#[derive(Debug, PartialEq)]
pub enum AppError {
    Parse(ParseIntError),
    Message(String),
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        AppError::Message(message)
    }
}

#[cgp_component(PortParser)]
#[use_type(HasErrorType.Error)]
pub trait CanParsePort {
    fn parse_port(&self, raw: &str) -> Result<u16, Error>;
}

#[cgp_impl(new ParsePort)]
#[uses(CanRaiseError<ParseIntError>, CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl PortParser {
    fn parse_port(&self, raw: &str) -> Result<u16, Error> {
        let port: u16 = raw.parse().map_err(Self::raise_error)?;

        if port == 0 {
            return Err(Self::raise_error("port 0 is reserved".to_owned()));
        }

        Ok(port)
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<AppError>,
        ErrorRaiserComponent: RaiseFrom,
        PortParserComponent: ParsePort,
    }
}

check_components! {
    App {
        PortParserComponent,
    }
}

pub fn demo() {
    assert_eq!(App.parse_port("8080"), Ok(8080));
    assert_eq!(
        App.parse_port("0"),
        Err(AppError::Message("port 0 is reserved".to_owned()))
    );
    assert!(matches!(App.parse_port("http"), Err(AppError::Parse(_))));
}
```

`ParsePort` names neither the context nor its error type; it raises a `ParseIntError` and a `String`
and leaves the conversion to the context. `App` is an
[environmental context](/docs/reference/glossary#environmental-context) that sets its error to
`AppError` and wires `RaiseFrom` once, which converts both sources through `AppError`'s two `From`
impls.

## When to use it

**Reach for `RaiseFrom` whenever the abstract error already has a `From` impl for the source.** It is
the plainest raiser and the one to try first, and it keeps the source as a value rather than
flattening it to a string.

Reach for [`ReturnError`](return_error.md) instead when the source *is* the abstract error, for
[`RaiseInfallible`](raise_infallible.md) when the source is `Infallible`, and for
[`DebugError`](debug_error.md) or [`DisplayError`](display_error.md) when no `From` impl exists but the
source is printable and the context handles `String` errors.

## Under the hood

`RaiseFrom` implements `ErrorRaiser<Context, E>` for any context whose abstract `Error` is `From<E>`:

```rust
#[cgp_new_provider]
impl<Context, E> ErrorRaiser<Context, E> for RaiseFrom
where
    Context: HasErrorType,
    Context::Error: From<E>,
{
    fn raise_error(e: E) -> Context::Error {
        e.into()
    }
}
```

The bound `Context::Error: From<E>` makes one wiring cover many source types. The generated
[`IsProviderFor`](../../traits/wiring/is_provider_for.md) impl carries the same clause, so a
[check](../../macros/check_components.md) reports a missing `From` impl at the wiring site.

## Common Mistakes

**`RaiseFrom` accepts only sources the error has a `From` impl for.** Wiring
`ErrorRaiserComponent: RaiseFrom` on a context whose error is `String` and raising a `ParseIntError`
fails at the check, since `String` has no `From<ParseIntError>`:

```text
error[E0277]: the trait bound `RaiseFrom: ErrorRaiser<App, ParseIntError>` is not satisfied
...
error[E0277]: the trait bound `String: From<ParseIntError>` is not satisfied
```

Add the `From` impl to an error type you own, or route that source to [`DebugError`](debug_error.md)
or [`DisplayError`](display_error.md) with the `open` statement, keeping `RaiseFrom` for the
`String` key.

## Related constructs

- [`CanRaiseError`](../../components/can_raise_error.md) — the component `RaiseFrom` supplies, through
  the `ErrorRaiser` provider trait.
- [`ReturnError`](return_error.md), [`RaiseInfallible`](raise_infallible.md) — the other pure raisers.
- [`DebugError`](debug_error.md), [`DisplayError`](display_error.md) — format a non-convertible source
  into a `String` that `RaiseFrom` can then convert.
- [`delegate_components!`](../../macros/delegate_components.md) — its `open` statement dispatches
  raisers per source type.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — the error type and its construction
  as separate wiring decisions.

## Source

- [`raise_from.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/raise_from.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
