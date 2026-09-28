---
title: 'PipeHandlers — a pipeline of handlers'
description: 'The combinator that folds a Product! list of handlers into nested ComposeHandlers, so the input flows through each stage in turn.'
sidebar_label: 'PipeHandlers'
sidebar_position: 2
---

# `PipeHandlers`

Compose a type-level list of handlers into a single pipeline, threading each stage's output into the
next.

## Overview

`PipeHandlers<Providers>` generalizes [`ComposeHandlers`](compose_handlers.md) from two handlers to
a list of them. It takes a [`Product!`](../../macros/product.md) list of providers and folds the
whole pipeline into one nested `ComposeHandlers`, so the input flows through each stage in turn, on
one [**context**](/docs/reference/glossary#context), the type the implementation runs against. It is
the combinator to reach for when wiring a multi-stage transformation: list the stages in order and
let `PipeHandlers` build the composition. Like every CGP provider, it carries no runtime value; the
list rides in `PhantomData`.

## Usage

Import it from `cgp::extra::handler`; it is not in the prelude. It takes one type parameter, a
`Product!` list of handler providers:

```rust
use cgp::extra::handler::PipeHandlers;

delegate_components! {
    App {
        ComputerComponent: PipeHandlers<Product![Double, AddOne, Double]>,
    }
}
```

The list runs left to right: `Double`, then `AddOne` on its output, then `Double` again. Each
stage's output type must match the next stage's input type. Because the fold is generic over the
component key, one list of two or more stages serves as a `Computer`, `TryComputer`,
`AsyncComputer`, or `Handler`, whichever the wiring asks for, as long as every stage supports that
shape; the fold's `ComposeHandlers` has no `…Ref` impl. A one-element list is that element, and an
empty list provides nothing.

## Examples

A pipeline of field-reading computers over a context, where `Multiply<Field>` and `Add<Field>` read
a factor or an addend from a context field:

```rust
use cgp::prelude::*;
use cgp::extra::handler::{CanCompute, PipeHandlers};

#[cgp_new_provider]
impl<Context, Code, Field> Computer<Context, Code, u64> for Multiply<Field>
where
    Context: HasField<Field, Value = u64>,
{
    type Output = u64;

    fn compute(context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * context.get_field(PhantomData)
    }
}

#[cgp_new_provider]
impl<Context, Code, Field> Computer<Context, Code, u64> for Add<Field>
where
    Context: HasField<Field, Value = u64>,
{
    type Output = u64;

    fn compute(context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input + context.get_field(PhantomData)
    }
}

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

check_components! {
    MyContext {
        ComputerComponent: ((), u64),
    }
}

pub fn demo() {
    let context = MyContext { foo: 2, bar: 3, baz: 4 };
    assert_eq!(context.compute(PhantomData::<()>, 5), 52); // ((5 * 2) + 3) * 4
}
```

The list folds to `ComposeHandlers<Multiply<…>, ComposeHandlers<Add<…>, Multiply<…>>>`. `MyContext`
is a [value context](/docs/reference/glossary#value-context): each stage reads one of its fields.
Stages of mismatched shapes can be reconciled inline, as in
`PromoteAsync<Promote<Add<Symbol!("bar")>>>`, which lifts a plain `Computer` stage to the async
`Handler` shape a `Handler` pipeline asks for.

## When to use it

**Reach for `PipeHandlers` to compose three or more handlers**, or any time a list reads better than
nesting [`ComposeHandlers`](compose_handlers.md) by hand. For exactly two, `ComposeHandlers` is the
plainer choice.

For a pipeline where a step should short-circuit on a `Result` branch rather than always feed the
next stage, use [`PipeMonadic`](../monad/pipe_monadic.md), the monadic generalization;
`PipeMonadic<IdentMonadic, …>` reduces to `PipeHandlers`.

## Under the hood

`PipeHandlers` carries the list in `PhantomData` and holds no handler impls of its own:

```rust
pub struct PipeHandlers<Providers>(pub PhantomData<Providers>);
```

Instead it delegates every component key to whatever single provider the list folds down to,
computed by an internal `ComposeProviders` trait that walks the `Cons`/`Nil` list. A one-element
list folds to that provider unchanged, and a list `Cons<ProviderA, rest>` folds to
`ComposeHandlers<ProviderA, fold(rest)>`:

```rust
delegate_components! {
    <Component, Provider, Providers: ComposeProviders<Provider = Provider>>
    PipeHandlers<Providers> {
        Component: Provider,
    }
}
```

Because the delegation is generic over the `Component` key, the same pipeline answers whichever
handler shape the wiring asks for.

## Related constructs

- [`ComposeHandlers`](compose_handlers.md) — the two-handler combinator this folds a list into.
- [`ReturnInput`](return_input.md) — the identity stage, which leaves a pipeline unchanged.
- [`Product!`](../../macros/product.md) — the type-level list of stages.
- [`PipeMonadic`](../monad/pipe_monadic.md) — the monadic pipeline that adds short-circuiting.
- [`Computer`](../../components/handler/computer.md),
  [`Handler`](../../components/handler/handler.md) — the family a pipeline implements.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its composition.

## Source

- [`providers/pipe.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/pipe.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
