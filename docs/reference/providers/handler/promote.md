---
title: 'Promote — lift a handler one step up'
description: 'The single-step lift that turns a Producer into a Computer, a Computer into a TryComputer, and an AsyncComputer into a Handler, adding no behavior.'
sidebar_label: 'Promote'
sidebar_position: 4
---

# `Promote`

Lift a handler one step up the family, from a producer to a computer or from infallible to fallible,
without adding behavior of its own.

## Overview

`Promote<Provider>` re-exposes one inner provider under a more capable member of the handler family.
It treats a less capable provider as a more capable one without introducing error or async behavior
itself, so the [**context**](/docs/reference/glossary#context), the type the implementation runs
against, sees the promoted shape while the inner provider does the same work. It is usually reached
through a promotion bundle such as [`PromoteComputer`](promote_computer.md), which the handler
macros wire for a generated provider, and it is written by hand when a context wires one handler
slot at a time. Like every CGP provider, it carries no runtime value; the inner provider rides in
`PhantomData`.

## Usage

Import it from `cgp::extra::handler`; it is not in the prelude. It takes one type parameter, the
inner provider:

```rust
use cgp::extra::handler::Promote;

// Fill a TryComputer slot from a plain Computer, wrapping its result in Ok.
delegate_components! {
    App {
        TryComputerComponent: Promote<Double>,
    }
}
```

Each lift takes exactly one step. A `Computer` reaches a `Handler` in two, as
`PromoteAsync<Promote<Double>>`: `Promote` makes the `TryComputer`, and
[`PromoteAsync`](promote_async.md) makes the `Handler` from it.

## Examples

A context fills three slots from two hand-written providers, one lift each and one chain of two:

```rust
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanCompute, CanHandle, CanTryCompute, Promote, PromoteAsync};

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

#[cgp_new_provider]
impl<Context, Code> Producer<Context, Code> for DefaultPort {
    type Output = u16;

    fn produce(_context: &Context, _code: PhantomData<Code>) -> u16 {
        8080
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ComputerComponent: Promote<DefaultPort>,
        TryComputerComponent: Promote<Double>,
        HandlerComponent: PromoteAsync<Promote<Double>>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), ()),
        [TryComputerComponent, HandlerComponent]: ((), u64),
    }
}

pub async fn demo() {
    let code = PhantomData::<()>;

    assert_eq!(App.compute(code, ()), 8080);
    assert_eq!(App.try_compute(code, 21), Ok(42));
    assert_eq!(App.handle(code, 21).await, Ok(42));
}
```

`Promote<DefaultPort>` answers `compute` by ignoring the input and producing `8080`,
`Promote<Double>` answers `try_compute` by wrapping `Double`'s output in `Ok`, and the chain answers
`handle`. The fallible members need the error type `App` wires. `App` is an [environmental
context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `Promote` when wiring a handler slot by hand and the slot needs exactly this lift**: a
producer filling a computer slot, or an infallible result wrapped in `Ok`. A provider written with
[`#[cgp_computer]`](../../macros/cgp_computer.md) or
[`#[cgp_producer]`](../../macros/cgp_producer.md) is already wired to a bundle, such as
[`PromoteComputer`](promote_computer.md), that fills in the whole family, so it needs no `Promote`
of its own. For the sync-to-async lift use [`PromoteAsync`](promote_async.md), for the borrow lift
[`PromoteRef`](promote_ref.md), and for the `Result` bridge [`TryPromote`](try_promote.md).

## Under the hood

`Promote<Provider>` carries the inner provider in `PhantomData` and gives three impls:

```rust
pub struct Promote<Provider>(pub PhantomData<Provider>);
```

- As a `Computer`, it requires the inner `Provider: Producer<Context, Code>` and ignores its own
  input, calling `Provider::produce`. This adapts a producer, which takes no input, to fill a
  computer slot that is handed an input it does not need.
- As a `TryComputer`, it requires `Provider: Computer` and wraps the infallible result in `Ok`.
- As a `Handler`, it requires `Provider: AsyncComputer` and wraps the awaited result in `Ok`.

The fallible impls require the context to have an error type. Each promotion adds the missing
behavior, either discarding an input or introducing an always-`Ok` result, without changing what the
inner provider computes.

## Related constructs

- [`PromoteAsync`](promote_async.md), [`PromoteRef`](promote_ref.md), [`TryPromote`](try_promote.md)
  — the other single-step lifts, along different axes.
- [`PromoteComputer`](promote_computer.md) and the other bundles — wire `Promote` into the family
  for a macro-generated provider.
- [`#[cgp_computer]`](../../macros/cgp_computer.md), [`#[cgp_producer]`](../../macros/cgp_producer.md) —
  generate providers that the bundles promote.
- [`Producer`](../../components/handler/producer.md),
  [`Computer`](../../components/handler/computer.md),
  [`Handler`](../../components/handler/handler.md) — the family members it lifts between.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and the orderings the promotions trade on.

## Source

- [`providers/promote.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
