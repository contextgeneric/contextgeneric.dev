---
title: 'Choose one error type for the whole context'
sidebar_label: 'Modular error handling'
sidebar_position: 10
description: 'How a context chooses one error type and the providers that raise and wrap it, so reusable code reports failures without fixing their representation.'
---

# Modular error handling

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations,
including the implementations that build errors. Modular error handling lets reusable code report
failures while each application chooses how to represent them. The context selects a shared error
type and providers for converting source errors into it and attaching detail. Reusable code declares
which of those operations it needs, allowing the same implementation to work with different error
policies.

Errors built this way still propagate through ordinary Rust `Result` values and the `?` operator.
This page explains how these choices fit together, how to preserve structured failures, and when
application-specific error operations are useful. It also considers the requirements and costs
of leaving error representation to the context.

## Separating failure from error representation {#why-the-error-type-is-the-hard-one}

Ordinary Rust offers several ways for reusable code to leave error representation to an
application. A parser, for example, can return a concrete source error for callers to convert,
take a generic error parameter, or use an associated error type.

CGP builds on the associated-type approach by giving a context one shared error type. Operations
that use this type can propagate each other's errors directly. To produce an error in the first
place, reusable code asks the context to convert a source error into the shared type. Other code
can then ask it to attach detail, such as the path of a configuration file.

These choices belong to separate built-in components:

- **`HasErrorType`:** Names the context's shared `Error` type, which must implement `Debug`.
- **`CanRaiseError<SourceError>`:** Converts a source error into that shared type.
- **`CanWrapError<Detail>`:** Attaches detail to an existing error.

The context selects providers for these components through wiring. That lets it choose both the
error type and how to construct or enrich it. Reusable code only declares which raising and
wrapping operations it needs.

## A parser with an abstract error type

Consider a parser that reads an integer and checks whether it fits in a port number. Either step
can fail, so the parser needs to raise both an integer parse error and a message about an
out-of-range value:

```rust
use cgp::prelude::*;
use core::num::ParseIntError;

#[cgp_fn(CanParsePort)]
#[uses(CanRaiseError<ParseIntError>, CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
pub fn parse_port(&self, raw: &str) -> Result<u16, Error> {
    let parsed: u32 = raw.parse().map_err(Self::raise_error)?;

    if parsed > u16::MAX as u32 {
        return Err(Self::raise_error(format!("port {parsed} out of range")));
    }

    Ok(parsed as u16)
}
```

The parser names its return error as `Error`, the context's [abstract type](./abstract-types.md)
imported by `#[use_type]`. When a step fails, `Self::raise_error` converts the source into that type,
and ordinary Rust `Result` and `?` handle propagation. The raising method is an associated
function on the context type, so constructing an error does not require a context value.

The parser still decides which failures to report. For example, `"abc"` produces a `ParseIntError`,
while `"65536"` parses as `u32` but exceeds `u16::MAX` and produces a string message. A number too
large for `u32` fails during parsing instead. Only the conversion of those failures into `Error`
is left to the context.

To make this implementation available, [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) generates a
`CanParsePort` trait and a blanket implementation for contexts that satisfy the dependencies in
`#[uses]`. Those raising requirements stay on the implementation. A generic caller can therefore
require `CanParsePort` and use its shared error type without repeating the conversion requirements.
There is only one parsing implementation here; a full component would also let contexts choose
among parsing providers.

## Selecting a strategy for each source

For an application that reports errors as strings, the context can select `String` and provide the
two conversions the parser needs. Here, `App` stands for the application and carries its
error-handling choices. The port text is passed to the method as an argument:

```rust
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::{DebugError, RaiseFrom};

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,

        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,
    }
}
```

The `open ErrorRaiserComponent;` statement lets the table choose a provider for each source type.
A raised `String` goes to `RaiseFrom`, which uses `From` and returns the string unchanged. A
`ParseIntError` goes to `DebugError`, which formats it into a string and passes that string to the
same `RaiseFrom` provider. Both paths end in `String`, so `App` can use the parser.

This formatting step keeps the parse error's debug output but loses its structure. Callers receive
a message they can log, though they can no longer match on the original `ParseIntError`. If an
application needs that structure, its context can choose a conversion that preserves it.

## Changing the error type

For example, another application may need to distinguish a parsing failure from a value that
passed parsing but exceeded `u16::MAX`. An error enum can keep the original parse error in one
variant and the range message in another:

```rust
#[derive(Debug)]
pub enum AppError {
    InvalidNumber(ParseIntError),
    InvalidPort(String),
}

impl From<ParseIntError> for AppError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidNumber(error)
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::InvalidPort(message)
    }
}

pub struct StrictApp;

delegate_components! {
    StrictApp {
        ErrorTypeProviderComponent: UseType<AppError>,
        ErrorRaiserComponent: RaiseFrom,
    }
}
```

Because `AppError` implements `From` for both source types, `StrictApp` can send both to
`RaiseFrom` with a single wiring entry. Parsing `"abc"` now returns an `Err` containing
`AppError::InvalidNumber`, while parsing `"65536"` returns one containing `AppError::InvalidPort`.
A caller that knows this context can match on either variant.

The parser itself has not changed. It raises the same source values through the same operations,
but the context preserves them in a different return type. An application can also choose an
error-library backend, such as `cgp-error-anyhow` for `anyhow::Error` or `cgp-error-eyre` for `eyre`.
These optional crates package compatible type providers, raisers, and wrappers. The application
selects them in its wiring, while reusable code continues to use the shared error traits.

## Attaching detail as an error propagates

The parser knows which input failed, but a configuration loader may also know which file it came
from. When the parser returns an error, the loader can use `CanWrapError<Detail>` to attach the
file path before passing the error to its caller:

```rust
#[cgp_fn(CanParseConfigPort)]
#[uses(CanParsePort, CanWrapError<String>)]
#[use_type(HasErrorType.Error)]
pub fn parse_config_port(&self, raw: &str, path: &str) -> Result<u16, Error> {
    self.parse_port(raw).map_err(|error| {
        Self::wrap_error(error, format!("invalid port in {path}"))
    })
}
```

Here, the parser's error already has the type the wrapper expects, because both operations use the
context's shared `Error`. The loader only needs to require `CanParsePort` and a wrapper for strings.
It does not repeat the parser's raising requirements, just as its own caller can require
`CanParseConfigPort` without repeating the loader's dependencies.

Wrapping adds information to an error that has already been raised. How much of that information
survives depends on the selected provider: a backend wrapper can retain both the error and the file
path, while `DiscardDetail` returns the error unchanged. Neither `App` nor `StrictApp` selects a
wrapper yet. Each would need a compatible `ErrorWrapperComponent` entry to use this loader, even
though both already support the parser.

## An application's own error vocabulary

Choosing behavior by source type works well when the type identifies the failure. In `StrictApp`,
for example, every raised `String` becomes `AppError::InvalidPort`. That suits a context where
strings only report port failures, but a web service may use strings to describe many unrelated
problems. It needs more information to choose the right error.

An application-specific component can supply that information explicitly. For HTTP errors, it can
accept a status marker alongside the detail:

```rust
#[cgp_component(HttpErrorRaiser)]
#[use_type(HasErrorType.Error)]
pub trait CanRaiseHttpError<Code, Detail> {
    fn raise_http_error(code: Code, detail: Detail) -> Error;
}
```

The status markers can be empty structs such as `ErrUnauthorized` and `ErrNotFound`. A provider
maps them to HTTP status codes and constructs the context's error, so a handler with the
corresponding `CanRaiseHttpError` dependency can write:

```rust
Self::raise_http_error(ErrUnauthorized, "you must log in first")
```

The marker identifies the failure even though the detail is a string. As with the port parser,
the handler leaves the concrete error representation to its context; here, the selected provider
also determines the numeric HTTP status.

## What it costs

Leaving error representation to the context moves the conversion choices into wiring. Those
choices still have to agree: the string context needs a provider that can format `ParseIntError`,
while the enum context needs `From` implementations for both sources. Changing the error type may
therefore mean changing the raising providers too. An enum cannot recover the original error if
a formatting provider has already reduced it to text.

Rust verifies these requirements when they are needed. Calling `App.parse_port(...)` or asserting
`App: CanParsePort` checks the raising dependencies, while checking `CanParseConfigPort` also
checks the wrapper. For components,
[`check_components!`](/docs/reference/macros/check_components) verifies the selected providers
against their contexts. Until then, a context can name an error type while still lacking a
conversion it needs.

Sharing the error type through `HasErrorType` lets operations compose, but it also gives each
context a single choice. An application that needs independent error types must use separate type
components or contexts. In either case, useful messages, preserved causes, and matchable variants
still depend on the application's error design.

For a parser that only needs one fixed error type, a concrete `Result` and ordinary conversions
may be enough. The additional traits and wiring are useful when the same parsing logic needs to
work with different error policies, as it does for `App` and `StrictApp`.

## Where to go next

To use the same parser with your own error policy, start with the shared type and the conversions
it needs. These pages explain the supporting mechanisms and available providers:

- [Abstract types](./abstract-types.md): How the context selects a shared type.
- [Impl-side dependencies](./impl-side-dependencies.md): Why conversion requirements stay on the
  provider.
- [Dispatching](./dispatching.md): Selecting providers by type.
- [`HasErrorType`](/docs/reference/components/has_error_type) and
  [`CanRaiseError`](/docs/reference/components/can_raise_error) /
  [`CanWrapError`](/docs/reference/components/can_wrap_error): The error interfaces.
- [Error providers](/docs/reference/providers/error): Strategies and their requirements.
- [Comparison: Algebraic effects](/docs/comparisons/algebraic-effects): Why raising selects an
  interpretation while ordinary Rust control flow still propagates the error.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
