---
title: 'DebugError — raise errors via Debug'
description: 'The error provider that formats a Debug source or detail into a String and forwards it to the context''s own String raiser or wrapper.'
sidebar_label: 'DebugError'
sidebar_position: 4
---

# `DebugError`

Raise or wrap any `Debug` source by formatting it into a `String` and forwarding to the context's own
string handling.

## Overview

`DebugError` implements both error components by redirecting through a string. Rather than producing
the abstract error directly, it formats the source error or the detail with the `Debug` trait, as
`{:?}` would print it, into a `String`, then forwards to the
[**context**](/docs/reference/glossary#context)'s own `CanRaiseError<String>` or
`CanWrapError<String>`, where the context is the type the implementation runs against. It does not
know the context's error type: it turns a `Debug` value into a `String` and hands it off, leaving
the final step to whatever provider the context wires for the `String` key. Like every CGP provider,
`DebugError` carries no runtime value.

This design lets a context handle an open-ended set of error types with one concrete string rule.
`DebugError` routes every `Debug` source through the single `String` key, and the context wires one
provider, often [`RaiseFrom`](raise_from.md), for that key.

## Usage

Import the provider from `cgp::extra::error` and the wiring keys from `cgp::core::error`; none of them
is in the prelude. `DebugError` lives behind the crate's `alloc` feature, which is on by default,
because it allocates a `String`. It takes no type parameter and is wired to `ErrorRaiserComponent`,
`ErrorWrapperComponent`, or both, usually per source type with the `open` statement:

```rust
use core::num::ParseIntError;
use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::{DebugError, RaiseFrom};

delegate_components! {
    App {
        open ErrorRaiserComponent;

        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,
    }
}
```

The `String` entry must be present, and must be some other provider, for `DebugError` to forward to.
Here a raised `ParseIntError` is formatted and handed to the `String` entry, which
[`RaiseFrom`](raise_from.md) converts into the abstract error.

## Examples

A provider raises a `ParseIntError`, and the context formats it with `DebugError` into the `String` it
uses as its error:

```rust
use core::num::ParseIntError;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::{DebugError, RaiseFrom};

#[cgp_component(PortParser)]
#[use_type(HasErrorType.Error)]
pub trait CanParsePort {
    fn parse_port(&self, raw: &str) -> Result<u16, Error>;
}

#[cgp_impl(new ParsePort)]
#[uses(CanRaiseError<ParseIntError>)]
#[use_type(HasErrorType.Error)]
impl PortParser {
    fn parse_port(&self, raw: &str) -> Result<u16, Error> {
        raw.parse().map_err(Self::raise_error)
    }
}

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        PortParserComponent: ParsePort,

        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,
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
        App.parse_port("http"),
        Err("ParseIntError { kind: InvalidDigit }".to_owned())
    );
}
```

`App` is an [environmental context](/docs/reference/glossary#environmental-context) whose error is
`String`. The `ParseIntError` key goes to `DebugError`, which produces
`"ParseIntError { kind: InvalidDigit }"` and raises it through the `String` key, and `RaiseFrom`
returns that string through the reflexive `From<String> for String`.

## When to use it

**Reach for `DebugError` when a source error implements `Debug` but the abstract error has no `From`
impl for it, and you want its debug output carried as a string.** Pair it with a provider on the
`String` key, since it forwards rather than finishes.

Reach for [`DisplayError`](display_error.md) when the source implements `Display` and you want its
user-facing message rather than its debug form. Reach for [`RaiseFrom`](raise_from.md) when a real
`From` impl exists, which preserves the source rather than flattening it to a string.

## Under the hood

`DebugError` implements `ErrorRaiser` for any `Debug` source over a context that raises `String`, and
`ErrorWrapper` for any `Debug` detail over a context that wraps `String`:

```rust
#[cgp_provider]
impl<Context, E> ErrorRaiser<Context, E> for DebugError
where
    Context: CanRaiseError<String>,
    E: Debug,
{
    fn raise_error(e: E) -> Context::Error {
        Context::raise_error(format!("{e:?}"))
    }
}

#[cgp_provider]
impl<Context, Detail> ErrorWrapper<Context, Detail> for DebugError
where
    Context: CanWrapError<String>,
    Detail: Debug,
{
    fn wrap_error(error: Context::Error, detail: Detail) -> Context::Error {
        Context::wrap_error(error, format!("{detail:?}"))
    }
}
```

Both bounds require the context to already handle the `String` case, which is the indirection that
reduces any `Debug` source to the one case the context knows. Each impl is paired with a matching
[`IsProviderFor`](../../traits/wiring/is_provider_for.md) impl.

## Common Mistakes

**The `String` key itself cannot be wired to `DebugError`.** For a `String` source, `DebugError`
formats it and raises the result through the context's `CanRaiseError<String>`, which is
`DebugError` again, so the lookup never ends. A check reports the overflow:

```text
error[E0275]: overflow evaluating the requirement `DebugError: ErrorRaiser<App, String>`
```

The same holds for `@ErrorWrapperComponent.String: DebugError`, which forwards to
`CanWrapError<String>`. Wire the `String` key to a provider that finishes the job, such as
[`RaiseFrom`](raise_from.md) for raising or a wrapper written for the context's error type, and route
only the other source types to `DebugError`.

## Related constructs

- [`DisplayError`](display_error.md) — the same provider formatting with `Display` instead.
- [`CanRaiseError`](../../components/can_raise_error.md) and
  [`CanWrapError`](../../components/can_wrap_error.md) — the components it supplies, and the `String`
  forms it forwards to.
- [`RaiseFrom`](raise_from.md) — the usual provider on the `String` key that finishes the raise.
- [`delegate_components!`](../../macros/delegate_components.md) — its `open` statement routes the
  source types `DebugError` should format.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — formatting providers as a redirect
  onto one concrete string rule.

## Source

- [`impls/alloc/debug_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/alloc/debug_error.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
