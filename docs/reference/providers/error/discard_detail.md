---
sidebar_label: 'DiscardDetail'
sidebar_position: 6
---

# `DiscardDetail`

Wrap an error by throwing the detail away and returning the error unchanged.

## Overview

`DiscardDetail` is the `ErrorWrapper` provider that ignores whatever detail is attached and returns the
error as it was. It satisfies the `CanWrapError` capability of a **context**, the type a capability runs
against, without enriching the error. This is useful when a context's error type cannot carry extra
context, or when the wrapping detail is deliberately not kept. It is the wrapping counterpart of a no-op:
the error propagates unchanged. Like every CGP provider, `DiscardDetail` carries no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorWrapperComponent` from
`cgp::core::error`. It takes no type parameter and accepts any detail type:

```rust
use cgp::core::error::ErrorWrapperComponent;
use cgp::extra::error::DiscardDetail;

delegate_components! {
    App {
        ErrorWrapperComponent: DiscardDetail,
    }
}
```

Wired this way, any call to `Context::wrap_error(error, detail)` on `App` returns `error` and drops
`detail`, for every detail type.

## When to use it

**Reach for `DiscardDetail` when a context's error type cannot hold extra detail, or when a call site
attaches detail that this context has no use for.** It keeps the `CanWrapError` capability satisfiable
without storing anything.

Reach for [`DebugError`](debug_error.md) or [`DisplayError`](display_error.md) when the detail should be
kept as a formatted string, or for a backend provider when the concrete error type can carry structured
context.

## Under the hood

`DiscardDetail` implements `ErrorWrapper<Context, Detail>` for any context and any detail type:

```rust
#[cgp_new_provider]
impl<Context, Detail> ErrorWrapper<Context, Detail> for DiscardDetail
where
    Context: HasErrorType,
{
    fn wrap_error(error: Context::Error, _detail: Detail) -> Context::Error {
        error
    }
}
```

The detail is bound only so the method can accept it; the body returns the error untouched. The
generated [`IsProviderFor`](../../traits/is_provider_for.md) impl carries the same `HasErrorType` bound.

## Related constructs

- [`CanWrapError`](../../components/can_wrap_error.md) — the component `DiscardDetail` supplies, through
  the `ErrorWrapper` provider trait.
- [`DebugError`](debug_error.md), [`DisplayError`](display_error.md) — wrap by keeping the detail as a
  formatted string instead of discarding it.
- [`UseDelegate`](../use_delegate.md) — dispatches wrappers per detail type.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — attaching detail as a wiring
  decision independent of the error type and its construction.

## Source

- [`discard_detail.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/discard_detail.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
