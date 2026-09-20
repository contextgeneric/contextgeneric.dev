---
sidebar_label: 'Modular error handling'
sidebar_position: 10
---

# Modular error handling

CGP lets reusable fallible code use an error type chosen by its context. The context selects the
error type, how source errors become that type, and how additional detail is attached. This page
follows a port parser through those choices, then applies the pattern to application-specific errors
and explains its costs.

## Separating failure from error representation {#why-the-error-type-is-the-hard-one}

A reusable parser can detect a failure without deciding how every application should represent it.
One application may want a string, another a domain enum, and another an error-library type. Fixing
that choice inside the parser limits where the same implementation can be reused.

Ordinary Rust offers several ways to expose the choice. A function can return a concrete error for
callers to convert, take a generic error parameter, or use an associated error type. CGP combines a
shared associated type with interchangeable providers for constructing and enriching errors.

The built-in components separate these responsibilities:

- **`HasErrorType`:** Names the context's shared `Error` type, which must implement `Debug`.
- **`CanRaiseError<SourceError>`:** Converts a source error into that shared type.
- **`CanWrapError<Detail>`:** Attaches detail to an existing error.

A provider declares the raising and wrapping operations it needs. The context then selects
compatible implementations, so these choices can vary without changing the provider's source.

## A parser with an abstract error type

The parser names its return error abstractly and raises the concrete failures it encounters.
This fragment assumes `cgp::prelude::*` and `core::num::ParseIntError` are imported:

```rust
#[cgp_component(PortParser)]
#[use_type(HasErrorType.Error)]
pub trait CanParsePort {
    fn parse_port(&self, raw: &str) -> Result<u16, Error>;
}

#[cgp_impl(new ParsePortFromStr)]
#[uses(CanRaiseError<ParseIntError>, CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl PortParser {
    fn parse_port(&self, raw: &str) -> Result<u16, Error> {
        let parsed: u32 = raw.parse().map_err(Self::raise_error)?;

        if parsed > u16::MAX as u32 {
            return Err(Self::raise_error(format!("port {parsed} out of range")));
        }

        Ok(parsed as u16)
    }
}
```

`Error` is the context's [abstract type](./abstract-types.md), imported by `#[use_type]`.
`Self::raise_error` converts either a `ParseIntError` or a `String` into it. The `#[uses]` line
requires those conversions on the implementation; a caller bounded by `CanParsePort` needs only
the parser interface and its shared error type.

`raise_error` is an associated function called on the context type. Error construction therefore
uses the context's selected implementation without requiring a context value.

## Selecting a strategy for each source

The context can route different source errors through different providers. This table assumes an
`App` context, the error component keys imported from `cgp::core::error`, and `RaiseFrom` and
`DebugError` imported from `cgp::extra::error`:

```rust
delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        PortParserComponent: ParsePortFromStr,

        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,
    }
}
```

`App` uses `String` as its error type. `RaiseFrom` converts a raised `String` through `From`, which
returns the same string here. `DebugError` formats a `ParseIntError` into a string, then calls the
context's `String` raiser. The formatting provider thus relies on the conversion selected by the
context instead of naming the final error type itself.

Other providers express different policies. `DisplayError` uses `Display` formatting,
`ReturnError` accepts a source that already has the context's error type, and `RaiseInfallible`
handles `Infallible`. Per-source dispatch lets a context combine these strategies where needed.

## Changing the error type

The same parser can return a domain error when another context supplies compatible conversions.
In this fragment, `AppError` must implement `Debug` and `From<String>`:

```rust
delegate_components! {
    StrictApp {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<AppError>,
        PortParserComponent: ParsePortFromStr,

        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,
    }
}
```

`ParsePortFromStr` can be reused unchanged because it never fixed its return error to `String`.
It still raises a `String` for an out-of-range port, and `RaiseFrom` now converts that message into
`AppError`. Rust checks and monomorphizes the provider for the new context as usual.

Error-library backends package compatible type providers, raisers, and wrappers. For example,
`cgp-error-anyhow` supplies providers for `anyhow::Error`, while `cgp-error-eyre` supplies the
corresponding `eyre` integration. Choosing a backend keeps those concrete dependencies in the
application's wiring rather than in reusable providers.

## Attaching detail as an error propagates

`CanWrapError<Detail>` lets a provider enrich an error it did not construct. A caller might attach
the configuration path after a port parser fails. The provider declares the wrapping requirement
and calls `Self::wrap_error(error, detail)`; the context chooses how the detail is stored or formatted.

Wrapping is separate from raising because a source error and propagation detail serve different
purposes. A source explains what failed, while detail can identify which request or resource was
involved. `DiscardDetail` can deliberately ignore that detail, and a backend wrapper can preserve it.
The parser above only raises errors, so its context does not need a wrapper until another provider
requires one.

## An application's own error vocabulary

Application-specific components can express failures that need more structure than a message.
A service can define a raising operation that takes a status marker and detail:

```rust
#[cgp_component(HttpErrorRaiser)]
#[use_type(HasErrorType.Error)]
pub trait CanRaiseHttpError<Code, Detail> {
    fn raise_http_error(code: Code, detail: Detail) -> Error;
}
```

Markers such as `ErrUnauthorized` and `ErrNotFound` identify domain failures. Providers map them to
status codes and construct the context's error. A handler requiring the appropriate
`CanRaiseHttpError` implementation can then call:

```rust
Self::raise_http_error(ErrUnauthorized, "you must first login")
```

The handler names the failure and its detail. Its selected provider determines the numeric status
and concrete error representation, just as the built-in raiser determines how a parse error is stored.

## What it costs

The selected error type must satisfy every chosen provider's requirements. A formatting route can
lose the original error's structure, and a `From` route requires a suitable conversion. Changing
the type may therefore require changing strategies as well as the type-setting entry.

Missing raisers and wrappers can remain undetected until the relevant operation is checked or used.
Use [`check_components!`](/docs/reference/macros/check_components) to verify the parser and other
application components against their contexts. Naming an error type alone does not establish that
all required conversions exist.

`HasErrorType` selects one shared error type per context. That makes component errors compose, but
an application needing independent error types must model them with separate type components or
contexts. CGP also leaves error design to the application: useful messages, preserved causes, and
matchable variants depend on the chosen representations and providers.

## Where to go next

These pages explain the supporting mechanisms and available strategies:

- [Abstract types](./abstract-types.md): How the context selects a shared type.
- [Impl-side dependencies](./impl-side-dependencies.md): Why conversion requirements stay on the
  provider.
- [Dispatching](./dispatching.md): Selecting providers by type.
- [`HasErrorType`](/docs/reference/components/has_error_type) and
  [`CanRaiseError` / `CanWrapError`](/docs/reference/components/can_raise_error): The error interfaces.
- [Error providers](/docs/reference/providers/error): Strategies and their requirements.
- [Comparison: Algebraic effects](/docs/comparisons/algebraic-effects): Why raising selects an
  interpretation while ordinary Rust control flow still propagates the error.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
