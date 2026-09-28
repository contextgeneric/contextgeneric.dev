---
title: 'ComposeHandlers — two handlers in sequence'
description: 'The combinator that runs one handler and feeds its output to a second, as a Computer, TryComputer, AsyncComputer, or Handler.'
sidebar_label: 'ComposeHandlers'
sidebar_position: 1
---

# `ComposeHandlers`

Run two handlers back to back, feeding the output of the first as the input of the second.

## Overview

`ComposeHandlers<ProviderA, ProviderB>` is the fundamental sequencing combinator. It runs
`ProviderA`, then runs `ProviderB` on `ProviderA`'s output, under one
[**context**](/docs/reference/glossary#context), the type the implementation runs against, and one
`Code` tag. It implements the four owned-input members of the handler family, `Computer`,
`TryComputer`, `AsyncComputer`, and `Handler`, by threading the intermediate value through both
providers; it has no `…Ref` or `Producer` impl. Only the value flowing between the two changes type.
Like every CGP provider, it carries no runtime value; the two inner providers ride in `PhantomData`.

## Usage

Import it from `cgp::extra::handler`; it is not in the prelude. It takes two provider type
parameters, the first handler and the second:

```rust
use cgp::extra::handler::ComposeHandlers;

delegate_components! {
    App {
        ComputerComponent: ComposeHandlers<Double, AddOne>,
    }
}
```

The second provider's input type is pinned to the first's output type, so the two must line up:
`ProviderB` computes over `ProviderA::Output`, and the composite output is `ProviderB::Output`. For
composing more than two handlers, reach for [`PipeHandlers`](pipe_handlers.md), which takes a list
and folds it into nested `ComposeHandlers`.

## Examples

Two computers composed and wired as one:

```rust
use cgp::prelude::*;
use cgp::extra::handler::{CanCompute, ComposeHandlers};

#[cgp_computer]
pub fn double(value: u64) -> u64 {
    value * 2
}

#[cgp_computer]
pub fn add_one(value: u64) -> u64 {
    value + 1
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent: ComposeHandlers<Double, AddOne>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), u64),
    }
}

pub fn demo() {
    assert_eq!(App.compute(PhantomData::<()>, 5), 11); // (5 * 2) + 1
}
```

Computing over `5` runs `Double` first and `AddOne` on its output. `App` is an
[environmental context](/docs/reference/glossary#environmental-context), and the same composite
serves as a `TryComputer`, `AsyncComputer`, or `Handler` wherever both stages support that shape.

## When to use it

**Reach for `ComposeHandlers` to sequence exactly two handlers.** For three or more, use
[`PipeHandlers`](pipe_handlers.md), which reads better as a list and folds to the same nested
composition. For a step that should short-circuit on a `Result` branch rather than always run the
next stage, use the [monad providers](../monad/pipe_monadic.md) instead.

## Under the hood

`ComposeHandlers` carries the two inner providers in `PhantomData`:

```rust
pub struct ComposeHandlers<ProviderA, ProviderB>(pub PhantomData<(ProviderA, ProviderB)>);
```

Its `Computer` impl requires `ProviderA: Computer<Context, Code, Input>` and
`ProviderB: Computer<Context, Code, ProviderA::Output>`, and sets the composite `Output` to
`ProviderB::Output`:

```rust
#[cgp_provider]
impl<Context, Code, Input, ProviderA, ProviderB> Computer<Context, Code, Input>
    for ComposeHandlers<ProviderA, ProviderB>
where
    ProviderA: Computer<Context, Code, Input>,
    ProviderB: Computer<Context, Code, ProviderA::Output>,
{
    type Output = ProviderB::Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: Input) -> Self::Output {
        let intermediary = ProviderA::compute(context, code, input);
        ProviderB::compute(context, code, intermediary)
    }
}
```

The same shape is implemented for `TryComputer`, `AsyncComputer`, and `Handler`. The fallible
variants use `?` to short-circuit on the first provider's error, so they require the context to have
an error type; the async variants `.await` each step.

## Related constructs

- [`PipeHandlers`](pipe_handlers.md) — generalizes this to a list, folding to nested
  `ComposeHandlers`.
- [`ReturnInput`](return_input.md) — the identity that composes with any handler without changing
  it.
- [`Computer`](../../components/handler/computer.md),
  [`Handler`](../../components/handler/handler.md) — the family it implements.
- [`PipeMonadic`](../monad/pipe_monadic.md) — composition that short-circuits through a monad.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family these combinators build.

## Source

- [`providers/compose.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/compose.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
