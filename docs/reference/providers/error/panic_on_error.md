---
sidebar_label: 'PanicOnError'
sidebar_position: 7
---

# `PanicOnError`

Abort with the source error's debug output instead of producing an abstract error value.

## Overview

`PanicOnError` is the `ErrorRaiser` provider that panics rather than returning an error. When a
**context**, the type a capability runs against, treats an error as a programming fault that should stop
the program, `PanicOnError` raises by calling `panic!` with the source error's debug representation. Its
signature promises to return the abstract error, but the body never does, because `panic!` diverges.
This suits tests and fail-fast tooling. Like every CGP provider, `PanicOnError` carries no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorRaiserComponent` from
`cgp::core::error`. It takes no type parameter and accepts any `Debug` source:

```rust
use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::PanicOnError;

delegate_components! {
    TestApp {
        ErrorRaiserComponent: PanicOnError,
    }
}
```

Any call to `Context::raise_error(source)` on `TestApp` panics with `format!("{source:?}")` rather than
returning.

## When to reach for it, and when not

**Reach for `PanicOnError` where an error means a bug that should abort rather than be handled**, such as
a test harness or a fail-fast tool. It removes the need to give the context a real error type or a
handling strategy.

Do not wire it in production code that should recover from errors. Use [`RaiseFrom`](raise_from.md) or a
backend provider there, so a raised error becomes a value the caller can handle.

## Under the hood

`PanicOnError` implements `ErrorRaiser<Context, E>` for any `Debug` source over any context with an
error type:

```rust
#[cgp_new_provider]
impl<Context, E> ErrorRaiser<Context, E> for PanicOnError
where
    Context: HasErrorType,
    E: Debug,
{
    fn raise_error(e: E) -> Context::Error {
        panic!("{e:?}")
    }
}
```

The return type is `Context::Error`, but `panic!` diverges, so the body never produces one. The
generated [`IsProviderFor`](../../traits/is_provider_for.md) impl carries the same bounds.

## Related constructs

- [`CanRaiseError`](../../components/can_raise_error.md) — the component `PanicOnError` supplies.
- [`RaiseFrom`](raise_from.md), [`ReturnError`](return_error.md) — the raisers to use when an error
  should become a value rather than abort.
- [`UseDelegate`](../use_delegate.md) — dispatches raisers per source-error type.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — the raising strategy as a wiring
  decision, here choosing to abort.

## Source

- [`panic_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/panic_error.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
