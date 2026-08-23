---
sidebar_label: 'ComposeHandlers'
sidebar_position: 1
---

# `ComposeHandlers`

Run two handlers back to back, feeding the output of the first as the input of the second.

## Overview

`ComposeHandlers<ProviderA, ProviderB>` is the fundamental sequencing combinator. It runs `ProviderA`,
then runs `ProviderB` on `ProviderA`'s output, under one **context**, the type a capability runs
against, and one `Code` tag. It implements every member of the handler family by threading the
intermediary value through both providers. Only the value flowing between them changes type. Like every
CGP provider, it carries no runtime value; the two inner providers ride in `PhantomData`.

## Usage

Import it from `cgp::extra::handler`. It takes two provider type parameters, the first handler and the
second:

```rust
use cgp::extra::handler::ComposeHandlers;

delegate_components! {
    App {
        ComputerComponent: ComposeHandlers<Double, AddOne>,
    }
}
```

The second provider's input type is pinned to the first's output type, so the two must line up:
`ProviderB` computes over `ProviderA::Output`, and the composite output is `ProviderB::Output`.

For composing more than two handlers, reach for [`PipeHandlers`](pipe_handlers.md), which takes a list
and folds it into nested `ComposeHandlers`.

## Examples

`ComposeHandlers` chains two field-reading computers over a context. Given a `Double` computer and an
`AddOne` computer, both over `u64`:

```rust
use cgp::extra::handler::{CanCompute, ComposeHandlers};

delegate_components! {
    App {
        ComputerComponent: ComposeHandlers<Double, AddOne>,
    }
}
```

Computing over an input of `5` runs `Double` first, then `AddOne` on its output, giving `(5 * 2) + 1`.
The same composite serves as a `TryComputer`, `AsyncComputer`, or `Handler` if both stages support that
shape.

## When to use it

**Reach for `ComposeHandlers` to sequence exactly two handlers.** For three or more, use
[`PipeHandlers`](pipe_handlers.md), which reads better as a list and folds to the same nested
composition. For a step that should short-circuit on a `Result` branch rather than always run the next
stage, use the [monad providers](../monad/pipe_monadic.md) instead.

## Under the hood

`ComposeHandlers` carries the two inner providers in `PhantomData`:

```rust
pub struct ComposeHandlers<ProviderA, ProviderB>(pub PhantomData<(ProviderA, ProviderB)>);
```

Its `Computer` impl requires `ProviderA: Computer<Context, Code, Input>` and
`ProviderB: Computer<Context, Code, ProviderA::Output>`, and sets the composite `Output` to
`ProviderB::Output`:

```rust
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

The same shape is implemented for `TryComputer`, `AsyncComputer`, and `Handler`. The fallible variants
use `?` to short-circuit on the first provider's error, so they require the context to have an error
type; the async variants `.await` each step.

## Related constructs

- [`PipeHandlers`](pipe_handlers.md) — generalizes this to a list, folding to nested `ComposeHandlers`.
- [`ReturnInput`](return_input.md) — the identity that composes with any handler without changing it.
- [`Computer`](../../components/handler/computer.md), [`Handler`](../../components/handler/handler.md) — the family it
  implements.
- [`PipeMonadic`](../monad/pipe_monadic.md) — composition that short-circuits through a monad.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family these combinators build.

## Source

- [`providers/compose.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/compose.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
