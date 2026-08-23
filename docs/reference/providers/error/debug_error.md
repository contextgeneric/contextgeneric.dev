---
sidebar_label: 'DebugError'
sidebar_position: 4
---

# `DebugError`

Raise or wrap any `Debug` source by formatting it into a `String` and forwarding to the context's own
string handling.

## Overview

`DebugError` implements both error components by redirecting through a string. Rather than producing the
abstract error directly, it formats the source error or the detail with the `Debug` trait into a
`String`, then forwards to the **context**'s own `CanRaiseError<String>` or `CanWrapError<String>`,
where the context is the type a capability runs against. It does not know the context's error type: it
only knows how to turn a `Debug` value into a `String` and hand it off, leaving the final step to
whatever string-handling provider the context already wires. Like every CGP provider, `DebugError`
carries no runtime value.

This design lets a context handle an open-ended set of error types with one concrete
string-raising rule. `DebugError` routes every `Debug` source through the single `String` path, and the
context wires one provider (often [`RaiseFrom`](raise_from.md)) for that path.

## Usage

Import the provider from `cgp::extra::error` and the wiring keys from `cgp::core::error`. `DebugError`
lives behind the crate's `alloc` feature, because it allocates a `String`. It takes no type parameter
and is wired to `ErrorRaiserComponent`, `ErrorWrapperComponent`, or both:

```rust
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

The `String` entry must be present for `DebugError` to forward to. Here a raised `ParseIntError` is
formatted with `Debug` and handed to the `String` entry, which `RaiseFrom` converts into the abstract
error.

## When to reach for it, and when not

**Reach for `DebugError` when a source error implements `Debug` but the abstract error has no `From` impl
for it, and you want its debug output carried as a string.** Pair it with a provider on the `String`
source, since it forwards rather than finishes.

Reach for [`DisplayError`](display_error.md) when the source implements `Display` and you want its
user-facing message rather than its debug form. Reach for [`RaiseFrom`](raise_from.md) when a real `From`
impl exists, which preserves the source rather than flattening it to a string.

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
[`IsProviderFor`](../../traits/is_provider_for.md) impl.

## Related constructs

- [`DisplayError`](display_error.md) — the same provider formatting with `Display` and `to_string()`.
- [`CanRaiseError` and `CanWrapError`](../../components/can_raise_error.md) — the components it supplies,
  and the `String` forms it forwards to.
- [`RaiseFrom`](raise_from.md) — the usual provider on the `String` source that finishes the raise.
- [`UseDelegate`](../use_delegate.md) — dispatches `DebugError` for the source types it should format.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — formatting providers as a redirect
  onto one concrete string rule.

## Source

- [`impls/alloc/debug_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/alloc/debug_error.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
