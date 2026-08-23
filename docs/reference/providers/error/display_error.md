---
sidebar_label: 'DisplayError'
sidebar_position: 5
---

# `DisplayError`

Raise or wrap any `Display` source by formatting it into a `String` and forwarding to the context's own
string handling.

## Overview

`DisplayError` is the `Display` counterpart of [`DebugError`](debug_error.md). It implements both error
components by formatting the source error or the detail with the `Display` trait into a `String`, then
forwarding to the **context**'s own `CanRaiseError<String>` or `CanWrapError<String>`, where the context
is the type a capability runs against. It carries the source's user-facing message rather than its debug
representation. Like every CGP provider, `DisplayError` carries no runtime value.

As with `DebugError`, it does not know the context's error type. It reduces any `Display` source to the
`String` case and leaves the final step to whatever provider the context wires for `String`.

## Usage

Import the provider from `cgp::extra::error` and the wiring keys from `cgp::core::error`. `DisplayError`
lives behind the crate's `alloc` feature, because it allocates a `String`. It takes no type parameter:

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

A `String` entry must be present for `DisplayError` to forward to.

## When to reach for it, and when not

**Reach for `DisplayError` when a source implements `Display` and you want its user-facing message
carried as a string.** Pair it with a provider on the `String` source, since it forwards rather than
finishes.

Reach for [`DebugError`](debug_error.md) when you want the source's debug form instead, or when the
source implements `Debug` but not `Display`. Reach for [`RaiseFrom`](raise_from.md) when a real `From`
impl exists, which keeps the source rather than reducing it to a string.

## Under the hood

`DisplayError` matches [`DebugError`](debug_error.md) in shape, but formats with `Display` through
`to_string()`:

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

Both bounds require the context to already handle `String`. Each impl is paired with a matching
[`IsProviderFor`](../../traits/is_provider_for.md) impl.

## Related constructs

- [`DebugError`](debug_error.md) — the same provider formatting with `Debug` instead.
- [`CanRaiseError`](../../components/can_raise_error.md) and [`CanWrapError`](../../components/can_wrap_error.md) — the components it supplies,
  and the `String` forms it forwards to.
- [`RaiseFrom`](raise_from.md) — the usual provider on the `String` source that finishes the raise.
- [`UseDelegate`](../use_delegate.md) — dispatches `DisplayError` for the source types it should format.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — formatting providers as a redirect
  onto one concrete string rule.

## Source

- [`impls/alloc/display_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/alloc/display_error.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
