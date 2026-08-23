---
sidebar_label: 'RaiseFrom'
sidebar_position: 1
---

# `RaiseFrom`

Raise a source error by converting it into the context's error type through the standard `From` trait.

## Overview

`RaiseFrom` is the `ErrorRaiser` provider for the common case: the **context**, the type a capability
runs against, already knows how to build its abstract `Error` from the source error through `From`.
Wiring it means "convert every source error the abstract error has a `From` impl for". It is the default
choice whenever that `From` impl exists, which covers most error raising in practice.

Because the bound is on the abstract error rather than on one source type, a single wiring of `RaiseFrom`
covers every source error the context's error can absorb. Like every CGP provider, `RaiseFrom` carries
no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorRaiserComponent` from
`cgp::core::error`. It takes no type parameter:

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
`source` whose type the `App` error implements `From` for.

## Examples

`RaiseFrom` is most often one entry in a per-source dispatch table, alongside the formatting providers
for the source types the error cannot absorb directly. Here a raised `String` is converted straight
into the abstract error, while a `ParseIntError` is formatted first:

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

[`DebugError`](debug_error.md) formats the `ParseIntError` into a `String` and forwards it back through the
`String` entry, which `RaiseFrom` handles.

## When to reach for it, and when not

**Reach for `RaiseFrom` whenever the abstract error already has a `From` impl for the source.** It is the
plainest raiser and the one to try first.

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

The bound `Context::Error: From<E>` is what makes one wiring cover many source types. The generated
[`IsProviderFor`](../../traits/is_provider_for.md) impl carries the same clause, so a
[check](../../macros/check_components.md) reports a missing `From` impl at the wiring site.

## Related constructs

- [`CanRaiseError`](../../components/can_raise_error.md) — the component `RaiseFrom` supplies, through
  the `ErrorRaiser` provider trait.
- [`ReturnError`](return_error.md), [`RaiseInfallible`](raise_infallible.md) — the other pure raisers.
- [`DebugError`](debug_error.md), [`DisplayError`](display_error.md) — format a non-convertible source
  into a `String` that `RaiseFrom` can then convert.
- [`UseDelegate`](../use_delegate.md) — dispatches `RaiseFrom` per source-error type.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — the error type and its construction
  as separate wiring decisions.

## Source

- [`raise_from.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/raise_from.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
