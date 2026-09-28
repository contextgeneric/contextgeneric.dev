---
title: 'RaiseInfallible — raise an Infallible error'
description: 'The error raiser for core::convert::Infallible, which fills the raiser slot for a step that cannot fail; its method is never called.'
sidebar_label: 'RaiseInfallible'
sidebar_position: 3
---

# `RaiseInfallible`

Absorb `core::convert::Infallible`, the error type that can never be constructed.

## Overview

`RaiseInfallible` is the `ErrorRaiser` provider for `core::convert::Infallible`, an error type with no
values. It lets generic code that raises the error of a fallible operation be wired uniformly on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against, even when
the operation chosen for that context cannot fail. Because an `Infallible` value cannot exist, the
raising method is never actually called at runtime. Like every CGP provider, `RaiseInfallible` carries
no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorRaiserComponent` from
`cgp::core::error`; neither is in the prelude. It takes no type parameter and accepts only
`Infallible` as the source:

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

The `Infallible` entry satisfies the raiser bound for a step that cannot fail, while other entries
handle the source types that can.

## Examples

A provider raises the error of a step that cannot fail, the same way it would raise a real one, and
the context fills that raiser slot with `RaiseInfallible`:

```rust
use core::convert::Infallible;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::{RaiseFrom, RaiseInfallible};

#[cgp_component(Runner)]
#[use_type(HasErrorType.Error)]
pub trait CanRun {
    fn run(&self) -> Result<(), Error>;
}

#[cgp_impl(new RunInfallibly)]
#[uses(CanRaiseError<Infallible>)]
#[use_type(HasErrorType.Error)]
impl Runner {
    fn run(&self) -> Result<(), Error> {
        // A step that cannot fail, raised the same way a fallible step would be.
        let outcome: Result<(), Infallible> = Ok(());
        outcome.map_err(Self::raise_error)?;
        Ok(())
    }
}

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        RunnerComponent: RunInfallibly,

        @ErrorRaiserComponent.Infallible: RaiseInfallible,
        @ErrorRaiserComponent.String: RaiseFrom,
    }
}

check_components! {
    App {
        RunnerComponent,
    }
}

pub fn demo() {
    assert_eq!(App.run(), Ok(()));
}
```

`RunInfallibly` requires `CanRaiseError<Infallible>`, and `App`, an
[environmental context](/docs/reference/glossary#environmental-context), meets it with
`RaiseInfallible`. The check is what proves the wiring resolves, since the raise is never taken at
run time.

## When to use it

**Reach for `RaiseInfallible` when generic code must raise an `Infallible` on a context, even though
the value can never occur.** This happens when a component is generic over an operation and one wiring
of it picks an operation that never errors, so the raiser slot for `Infallible` still has to be
filled.

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

Because an `Infallible` value cannot be constructed, the empty `match` is total and the function has
no reachable body. The generated [`IsProviderFor`](../../traits/wiring/is_provider_for.md) impl
carries the same `HasErrorType` bound.

## Related constructs

- [`CanRaiseError`](../../components/can_raise_error.md) — the component `RaiseInfallible` supplies.
- [`RaiseFrom`](raise_from.md), [`ReturnError`](return_error.md) — the other pure raisers.
- [`delegate_components!`](../../macros/delegate_components.md) — its `open` statement dispatches
  raisers per source type, which is where the `Infallible` entry sits.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — raising a source into the abstract
  error type as a wiring decision.

## Source

- [`infallible.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/infallible.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
