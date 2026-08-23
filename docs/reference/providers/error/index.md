---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Error providers

The zero-sized providers that raise and wrap a context's abstract error, without depending on which
concrete error type the context chose.

## Overview

The error providers supply common error-handling strategies to any **context**, the type a capability
runs against. [`CanRaiseError` and `CanWrapError`](../../components/can_raise_error.md) define *what* a
context can do with an error: turn a source error into its abstract `Self::Error`, or attach detail to
one it already holds. They say nothing about *how*. These providers supply the how for the cases that
need no particular error library. Each one is generic over the context, so it works with whatever error
type the context's [`HasErrorType`](../../components/has_error_type.md) names.

Each provider implements one or both of two provider traits. `CanRaiseError`'s provider trait is
`ErrorRaiser`, wired with `ErrorRaiserComponent`. `CanWrapError`'s provider trait is `ErrorWrapper`,
wired with `ErrorWrapperComponent`. A provider that implements `ErrorRaiser` supplies raising; one that
implements `ErrorWrapper` supplies wrapping; two of them supply both.

These are the in-tree counterparts to the standalone backends in `cgp-error-anyhow`, `cgp-error-eyre`,
and `cgp-error-std`. Those crates supply providers tied to one concrete error type, such as raising into
an `anyhow::Error`. The providers here stay abstract over the context's error type and capture
strategies that hold for any of them. A context usually wires a mix: a backend for the concrete error
type, plus these generic providers for the cross-cutting strategies.

## The seven providers, by role

The seven divide into three groups.

The **pure raisers** convert a source error into the abstract error:

- [`RaiseFrom`](raise_from.md) converts a source error through the standard `From` trait.
- [`ReturnError`](return_error.md) handles the case where the source error already *is* the context's
  error, so raising is the identity.
- [`RaiseInfallible`](raise_infallible.md) absorbs `core::convert::Infallible`, an error that can never
  be constructed.

The **string-formatting providers** implement both components by formatting through a string:

- [`DebugError`](debug_error.md) formats the source or detail with the `Debug` trait, then forwards to
  the context's own `String` handling.
- [`DisplayError`](display_error.md) does the same with the `Display` trait.

The remaining two act on the wrapping and raising components directly:

- [`DiscardDetail`](discard_detail.md) wraps by throwing the detail away and returning the error
  unchanged.
- [`PanicOnError`](panic_on_error.md) aborts with the source error's debug output rather than producing
  an error value.

## Wiring them

A context gains an error strategy by wiring one of these providers to `ErrorRaiserComponent` or
`ErrorWrapperComponent`, exactly like any other component, and the provider's `where` clause decides
when that wiring type-checks. Because both components dispatch on the source-error or detail type, a
context commonly wires several providers at once, one per source error type. The modern form is the
`open` statement of [`delegate_components!`](../../macros/delegate_components.md); the error family is
still defined with [`#[derive_delegate]`](../../attributes/derive_delegate.md), so a nested
[`UseDelegate`](../use_delegate.md) table is the form you will read in existing code.

## Related constructs

- [`CanRaiseError` and `CanWrapError`](../../components/can_raise_error.md) — the components these
  implement, through the `ErrorRaiser` and `ErrorWrapper` provider traits.
- [`HasErrorType`](../../components/has_error_type.md) — names the abstract `Self::Error` every provider
  here produces.
- [`UseDelegate`](../use_delegate.md) — the table that dispatches these providers per source-error type.
- [`delegate_components!`](../../macros/delegate_components.md) and
  [`check_components!`](../../macros/check_components.md) — wire and verify them.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — how the abstract error type, the
  raise and wrap capabilities, and these strategies fit together as interchangeable wiring decisions.

## Source

- The providers are in `cgp-error-extra`:
  [`impls/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-error-extra/src/impls).
- The `ErrorRaiser` and `ErrorWrapper` provider traits and the consumer traits they build on are in
  [`cgp-error`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-error/src).
- The concrete backends are in
  [`crates/standalone/error/`](https://github.com/contextgeneric/cgp/tree/main/crates/standalone/error).

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
