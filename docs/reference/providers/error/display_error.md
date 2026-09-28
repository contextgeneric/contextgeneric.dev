---
title: 'DisplayError — raise errors via Display'
description: 'The error provider that formats a Display source or detail with to_string() and forwards it to the context''s own String raiser or wrapper.'
sidebar_label: 'DisplayError'
sidebar_position: 5
---

# `DisplayError`

Raise or wrap any `Display` source by formatting it into a `String` and forwarding to the context's own
string handling.

## Overview

`DisplayError` implements both error components by redirecting through a string. Rather than
producing the abstract error directly, it formats the source error or the detail with the `Display`
trait through `to_string()` into a `String`, then forwards to the
[**context**](/docs/reference/glossary#context)'s own `CanRaiseError<String>` or
`CanWrapError<String>`, where the context is the type the implementation runs against. It does not
know the context's error type: it turns a `Display` value into a `String` and hands it off, leaving
the final step to whatever provider the context wires for the `String` key. Like every CGP provider,
`DisplayError` carries no runtime value.

It is the `Display` counterpart of [`DebugError`](debug_error.md): it carries the source's user-facing
message rather than its debug representation.

## Usage

Import the provider from `cgp::extra::error` and the wiring keys from `cgp::core::error`; none of them
is in the prelude. `DisplayError` lives behind the crate's `alloc` feature, which is on by default,
because it allocates a `String`. It takes no type parameter and is wired to `ErrorRaiserComponent`,
`ErrorWrapperComponent`, or both, usually per source type with the `open` statement:

```rust
use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::{DisplayError, RaiseFrom};

delegate_components! {
    App {
        open ErrorRaiserComponent;

        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DisplayError,
    }
}
```

The `String` entry must be present, and must be some other provider, for `DisplayError` to forward to.
Here a raised `ParseIntError` is formatted and handed to the `String` entry, which
[`RaiseFrom`](raise_from.md) converts into the abstract error.

## Examples

A provider raises a `ParseIntError`, and the context formats it with `DisplayError` into the
`String` it uses as its error:

```rust
use core::num::ParseIntError;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::{DisplayError, RaiseFrom};

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
        @ErrorRaiserComponent.ParseIntError: DisplayError,
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
        Err("invalid digit found in string".to_owned())
    );
}
```

`App` is an [environmental context](/docs/reference/glossary#environmental-context) whose error is
`String`. The `ParseIntError` key goes to `DisplayError`, which produces
`"invalid digit found in string"` and raises it through the `String` key, and `RaiseFrom` returns
that string through the reflexive `From<String> for String`.

## When to use it

**Reach for `DisplayError` when a source implements `Display` and you want its user-facing message
carried as a string.** Pair it with a provider on the `String` key, since it forwards rather than
finishes.

Reach for [`DebugError`](debug_error.md) when you want the source's debug form instead, or when the
source implements `Debug` but not `Display`. Reach for [`RaiseFrom`](raise_from.md) when a real `From`
impl exists, which keeps the source rather than reducing it to a string.

## Under the hood

`DisplayError` implements `ErrorRaiser` for any `Display` source over a context that raises
`String`, and `ErrorWrapper` for any `Display` detail over a context that wraps `String`:

```rust
#[cgp_provider]
impl<Context, E> ErrorRaiser<Context, E> for DisplayError
where
    Context: CanRaiseError<String>,
    E: Display,
{
    fn raise_error(e: E) -> Context::Error {
        Context::raise_error(e.to_string())
    }
}

#[cgp_provider]
impl<Context, Detail> ErrorWrapper<Context, Detail> for DisplayError
where
    Context: CanWrapError<String>,
    Detail: Display,
{
    fn wrap_error(error: Context::Error, detail: Detail) -> Context::Error {
        Context::wrap_error(error, detail.to_string())
    }
}
```

Both bounds require the context to already handle the `String` case, which is the indirection that
reduces any `Display` source to the one case the context knows. Each impl is paired with a matching
[`IsProviderFor`](../../traits/wiring/is_provider_for.md) impl.

## Common Mistakes

**The `String` key itself cannot be wired to `DisplayError`.** For a `String` source, `DisplayError`
formats it and raises the result through the context's `CanRaiseError<String>`, which is
`DisplayError` again, so the lookup never ends. A check reports the overflow:

```text
error[E0275]: overflow evaluating the requirement `DisplayError: ErrorRaiser<App, String>`
```

The same holds for `@ErrorWrapperComponent.String: DisplayError`, which forwards to
`CanWrapError<String>`. Wire the `String` key to a provider that finishes the job, such as
[`RaiseFrom`](raise_from.md) for raising or a wrapper written for the context's error type, and route
only the other source types to `DisplayError`.

## Related constructs

- [`DebugError`](debug_error.md) — the same provider formatting with `Debug` instead.
- [`CanRaiseError`](../../components/can_raise_error.md) and
  [`CanWrapError`](../../components/can_wrap_error.md) — the components it supplies, and the `String`
  forms it forwards to.
- [`RaiseFrom`](raise_from.md) — the usual provider on the `String` key that finishes the raise.
- [`delegate_components!`](../../macros/delegate_components.md) — its `open` statement routes the
  source types `DisplayError` should format.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — formatting providers as a redirect
  onto one concrete string rule.

## Source

- [`impls/alloc/display_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/alloc/display_error.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
