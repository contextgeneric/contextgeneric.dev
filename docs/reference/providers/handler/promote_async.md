---
title: 'PromoteAsync — run a sync handler as async'
description: 'The single-step lift that runs a synchronous Computer or TryComputer inside an async method, to serve an AsyncComputer or Handler slot.'
sidebar_label: 'PromoteAsync'
sidebar_position: 5
---

# `PromoteAsync`

Lift a synchronous handler into an asynchronous one.

## Overview

`PromoteAsync<Provider>` runs a synchronous inner provider inside an async method, so a synchronous
handler serves an asynchronous slot on a [**context**](/docs/reference/glossary#context), the type
the implementation runs against. The returned future does all its work when first polled and awaits
nothing, so no actual asynchrony is added. It is usually reached through a promotion bundle such as
[`PromoteComputer`](promote_computer.md), and it is written by hand when a context wires one slot at
a time. Like every CGP provider, it carries no runtime value; the inner provider rides in
`PhantomData`.

## Usage

Import it from `cgp::extra::handler`; it is not in the prelude. It takes one type parameter, the
inner provider:

```rust
use cgp::extra::handler::PromoteAsync;

// Serve an AsyncComputer slot from a synchronous Computer.
delegate_components! {
    App {
        AsyncComputerComponent: PromoteAsync<Double>,
    }
}
```

## Examples

A context serves both async members from hand-written synchronous providers:

```rust
use core::num::ParseIntError;
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DebugError;
use cgp::extra::error::RaiseFrom;
use cgp::extra::handler::{CanComputeAsync, CanHandle, PromoteAsync};

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

#[cgp_impl(new ParseU64)]
#[uses(CanRaiseError<ParseIntError>)]
#[use_type(HasErrorType.Error)]
impl<Code> TryComputer<Code, String> {
    type Output = u64;

    fn try_compute(&self, _code: PhantomData<Code>, input: String) -> Result<u64, Error> {
        input.parse().map_err(Self::raise_error)
    }
}

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,

        AsyncComputerComponent: PromoteAsync<Double>,
        HandlerComponent: PromoteAsync<ParseU64>,
    }
}

check_components! {
    App {
        AsyncComputerComponent: ((), u64),
        HandlerComponent: ((), String),
    }
}

pub async fn demo() {
    let code = PhantomData::<()>;

    assert_eq!(App.compute_async(code, 21).await, 42);
    assert_eq!(App.handle(code, "12".to_owned()).await, Ok(12));
}
```

`PromoteAsync<Double>` answers `compute_async` from the synchronous `Double`, and
`PromoteAsync<ParseU64>` answers `handle` from the fallible synchronous `ParseU64`, passing its
`Result` through. `App` is an [environmental
context](/docs/reference/glossary#environmental-context) that also wires the error type and raisers
`ParseU64` needs.

## When to use it

**Reach for `PromoteAsync` when wiring an async slot by hand from a synchronous provider.** A
provider written with [`#[cgp_computer]`](../../macros/cgp_computer.md) is already wired to a bundle
that routes its async slots here. For the infallible-to-fallible or producer lift use
[`Promote`](promote.md), and for the borrow lift [`PromoteRef`](promote_ref.md).

## Under the hood

`PromoteAsync<Provider>` carries the inner provider in `PhantomData`:

```rust
pub struct PromoteAsync<Provider>(pub PhantomData<Provider>);
```

- As an `AsyncComputer`, it requires `Provider: Computer` and runs it synchronously inside the async
  method, returning a future that awaits nothing.
- As a `Handler`, it requires `Provider: TryComputer` and returns that fallible synchronous result,
  so a synchronous fallible computer becomes an async fallible handler. This impl requires the
  context to have an error type.

## Related constructs

- [`Promote`](promote.md), [`PromoteRef`](promote_ref.md), [`TryPromote`](try_promote.md) — the
  other single-step lifts.
- [`PromoteComputer`](promote_computer.md) and the other bundles — wire `PromoteAsync` into the
  family for a macro-generated provider.
- [`Computer`](../../components/handler/computer.md),
  [`TryComputer`](../../components/handler/try_computer.md),
  [`Handler`](../../components/handler/handler.md) — the family members it lifts between.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and the sync-to-async ordering this lift trades
  on.

## Source

- [`providers/promote_async.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_async.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
