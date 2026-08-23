---
sidebar_label: 'PipeHandlers'
sidebar_position: 2
---

# `PipeHandlers`

Compose a type-level list of handlers into a single pipeline, threading each stage's output into the
next.

## Overview

`PipeHandlers<Providers>` generalizes [`ComposeHandlers`](compose_handlers.md) from two handlers to a
list of them. It takes a [`Product!`](../../macros/product.md) list of providers and folds the whole
pipeline into one nested `ComposeHandlers`, so the input flows through each stage in turn, on one
**context**, the type a capability runs against. It is the combinator to reach for when wiring a
multi-stage transformation: list the stages in order and let `PipeHandlers` build the composition. Like
every CGP provider, it carries no runtime value; the list rides in `PhantomData`.

## Usage

Import it from `cgp::extra::handler`. It takes one type parameter, a `Product!` list of handler
providers:

```rust
use cgp::extra::handler::{CanCompute, PipeHandlers};

delegate_components! {
    App {
        ComputerComponent:
            PipeHandlers<Product![ParseInput, Validate, Normalize]>,
    }
}
```

The list runs left to right: `ParseInput`, then `Validate` on its output, then `Normalize`. Each
stage's output type must match the next stage's input type. Because the fold is generic over the
component, one list serves as a `Computer`, `TryComputer`, `AsyncComputer`, or `Handler`, whichever the
wiring asks for, as long as every stage supports that shape.

## Examples

A pipeline of field-reading computers over a context. Suppose `Multiply<Field>` and `Add<Field>` are
`Computer` providers over `u64` that read a factor or an addend from a context field:

```rust
use cgp::prelude::*;
use cgp::extra::handler::{CanCompute, PipeHandlers};

#[derive(HasField)]
pub struct MyContext {
    pub foo: u64,
    pub bar: u64,
    pub baz: u64,
}

delegate_components! {
    MyContext {
        ComputerComponent:
            PipeHandlers<Product![
                Multiply<Symbol!("foo")>,
                Add<Symbol!("bar")>,
                Multiply<Symbol!("baz")>,
            ]>,
    }
}
```

Wiring `ComputerComponent` to this list composes the three into
`ComposeHandlers<Multiply<…>, ComposeHandlers<Add<…>, Multiply<…>>>`. Computing over an input of `5` on
a context with `foo = 2`, `bar = 3`, `baz = 4` gives `((5 * 2) + 3) * 4`. Stages of mismatched shapes
can be reconciled inline, as in `PromoteAsync<Promote<Add<Symbol!("bar")>>>`, which lifts a plain
`Computer` stage up to the async `Handler` shape a pipeline expects.

## When to reach for it, and when not

**Reach for `PipeHandlers` to compose three or more handlers**, or any time a list reads better than
nesting [`ComposeHandlers`](compose_handlers.md) by hand. For exactly two, `ComposeHandlers` is the
plainer choice.

For a pipeline where a step should short-circuit on a `Result` branch rather than always feed the next
stage, use [`PipeMonadic`](../monad/pipe_monadic.md), the monadic generalization that
`PipeHandlers<IdentMonadic>` reduces to.

## Under the hood

`PipeHandlers` carries the list in `PhantomData` and holds no handler impls of its own:

```rust
pub struct PipeHandlers<Providers>(pub PhantomData<Providers>);
```

Instead it delegates every handler component to whatever single provider the list folds down to,
computed by an internal `ComposeProviders` trait that walks the `Cons`/`Nil` list. A one-element list
folds to that provider unchanged; a list `Cons<ProviderA, rest>` folds to
`ComposeHandlers<ProviderA, fold(rest)>`:

```rust
delegate_components! {
    <Component, Provider, Providers: ComposeProviders<Provider = Provider>>
    PipeHandlers<Providers> {
        Component: Provider,
    }
}
```

Because the delegation is generic over the `Component` key, the same pipeline answers whichever handler
shape the wiring asks for.

## Related constructs

- [`ComposeHandlers`](compose_handlers.md) — the two-handler combinator this folds a list into.
- [`ReturnInput`](return_input.md) — the identity stage, the neutral element of the fold.
- [`Product!`](../../macros/product.md) — the type-level list of stages.
- [`PipeMonadic`](../monad/pipe_monadic.md) — the monadic pipeline that adds short-circuiting.
- [`Computer`](../../components/handler/computer.md), [`Handler`](../../components/handler/handler.md) — the family a
  pipeline implements.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its composition.

## Source

- [`providers/pipe.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/pipe.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
