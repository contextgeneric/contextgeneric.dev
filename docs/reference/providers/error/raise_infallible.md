---
sidebar_label: 'RaiseInfallible'
sidebar_position: 3
---

# `RaiseInfallible`

Absorb `core::convert::Infallible`, the error type that can never be constructed.

## Overview

`RaiseInfallible` is the `ErrorRaiser` provider for `core::convert::Infallible`, an error type with no
values. It lets generic code that is parameterized over a fallible operation be wired uniformly on a
**context**, the type a capability runs against, even when the operation chosen for that context cannot
fail. Because an `Infallible` value cannot exist, the raising method is never actually called at
runtime. Like every CGP provider, `RaiseInfallible` carries no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorRaiserComponent` from
`cgp::core::error`. It takes no type parameter and accepts only `Infallible` as the source:

```rust
use core::convert::Infallible;
use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::{RaiseFrom, RaiseInfallible};

delegate_components! {
    App {
        open ErrorRaiserComponent;

        @ErrorRaiserComponent.Infallible: RaiseInfallible,
        @ErrorRaiserComponent.String: RaiseFrom,
    }
}
```

The `Infallible` entry satisfies the raiser bound for a step that cannot fail, while other entries handle
the source types that can.

## When to use it

**Reach for `RaiseInfallible` when generic code must raise an `Infallible` on a context, even though the
value can never occur.** This happens when a component is generic over an operation and one wiring of it
picks an operation that never errors, so the raiser slot for `Infallible` still has to be filled.

For any other source type, use one of the other raisers: [`RaiseFrom`](raise_from.md),
[`ReturnError`](return_error.md), [`DebugError`](debug_error.md), or [`DisplayError`](display_error.md).

## Under the hood

`RaiseInfallible` implements `ErrorRaiser<Context, Infallible>` for any context with an error type,
producing the abstract error by matching on the uninhabited value:

```rust
#[cgp_new_provider]
impl<Context> ErrorRaiser<Context, Infallible> for RaiseInfallible
where
    Context: HasErrorType,
{
    fn raise_error(e: Infallible) -> Context::Error {
        match e {}
    }
}
```

Because an `Infallible` value cannot be constructed, the empty `match` is total and the function has no
reachable body. The generated [`IsProviderFor`](../../traits/is_provider_for.md) impl carries the same
`HasErrorType` bound.

## Related constructs

- [`CanRaiseError`](../../components/can_raise_error.md) — the component `RaiseInfallible` supplies.
- [`RaiseFrom`](raise_from.md), [`ReturnError`](return_error.md) — the other pure raisers.
- [`UseDelegate`](../use_delegate.md) — dispatches raisers per source-error type, which is where the
  `Infallible` entry sits.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — raising a source into the abstract
  error type as a wiring decision.

## Source

- [`infallible.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/infallible.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
