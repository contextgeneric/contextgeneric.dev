---
sidebar_label: 'Producer'
sidebar_position: 4
---

# `Producer`

The input-free member of the handler family: a synchronous, infallible source of a value.

## Overview

`Producer` is for computations that take nothing to compute and simply yield a value. A handler that
supplies a default configuration, a constant, or a value read entirely from the **context** has no
`Input` to transform: it needs only the context and a phantom `Code` tag to know which value to produce.
The context is the type a capability runs against, which supplies the values an implementation needs as
its own fields. The rest of the [handler family](/docs/concepts/handlers) threads an `Input` through
every method; `Producer` is the case where that input is absent. It is the simplest component in the
family, synchronous and infallible, producing an `Output` directly.

Being input-free does not isolate a producer from the family. It makes it the natural *source* of values
that flow into the other handlers: a producer promotes into any computer or handler by ignoring whatever
input that variant supplies and returning the produced value regardless. This is how a constant or a
context-derived value enters a computation pipeline.

## Definition

`CanProduce` is defined as:

```rust
#[cgp_component(Producer)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
pub trait CanProduce<Code> {
    type Output;

    fn produce(&self, _code: PhantomData<Code>) -> Self::Output;
}
```

Its attributes:

- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `Producer` that implementations target and the wiring key `ProducerComponent`, while `CanProduce` stays the consumer trait callers use.
- [`#[prefix]`](../../macros/cgp_namespace.md) — registers the generated names into the `@cgp.extra.handler` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates a `UseDelegate` provider that dispatches on the `Code` type, so a context can route each `Code` to its own provider; the `open` statement is the modern sugar for the same dispatch.

## Usage

`Producer` and its consumer trait `CanProduce` are imported from `cgp::extra::handler`. The consumer
method takes only the context and a `Code` tag, with no `Input`:

```rust
fn produce(&self, _code: PhantomData<Code>) -> Self::Output;
```

A context gains the capability by wiring `ProducerComponent` to a provider. In everyday use the provider
comes from [`#[cgp_producer]`](../../macros/cgp_producer.md), which turns a zero-argument function such as
`fn magic_number() -> u64 { 42 }` into a `Producer` provider and wires `PromoteProducer<Self>` so the
same function also answers the input-taking components. Because there is no input to dispatch on, the
component dispatches on the `Code` tag alone, with no `Input`-keyed table. `Producer` has no
by-reference or async sibling, since there is no input to borrow.

## Examples

A producer wired into a context and invoked directly:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::{CanProduce, Producer, ProducerComponent};

#[cgp_new_provider]
impl<Context, Code> Producer<Context, Code> for MagicNumber {
    type Output = u64;

    fn produce(_context: &Context, _code: PhantomData<Code>) -> u64 {
        42
    }
}

pub struct App;

delegate_components! {
    App {
        ProducerComponent: MagicNumber,
    }
}

fn run(app: &App) -> u64 {
    app.produce(PhantomData::<()>) // returns 42
}
```

`MagicNumber` produces `42` from the `Code` tag alone, and `App` delegates `ProducerComponent` to it,
giving `App` the `CanProduce<(), Output = u64>` capability. The example is **parameter-targeted**: the
context decides which provider answers, and no value is operated on. In practice the provider is written
with [`#[cgp_producer]`](../../macros/cgp_producer.md), which also wires the promotion so the same function
answers the input-taking components while ignoring their input.

## When to reach for it, and when not

**Reach for `Producer` for a value that a computation needs but does not compute from an input**, such
as a constant, a default, or a value read from the context. It is the family's source, and promotion
lets one producer feed the input-taking members: delegating a context's handler surface to
`PromoteProducer<Self>` makes one producer answer `CanCompute`, `CanTryCompute`, `CanComputeAsync`,
`CanHandle`, and their by-reference forms.

Reach for [`Computer`](./computer.md) instead when the value is computed *from* an input, and for
[`TryComputer`](./try_computer.md) or [`Handler`](./handler.md) when producing it can fail or must await.

## Related constructs

- [`Computer`](./computer.md) — the synchronous input-taking member; a producer promotes into it.
- [`TryComputer`](./try_computer.md) — the fallible member of the family.
- [`Handler`](./handler.md) — the general async-and-fallible member.
- [`#[cgp_producer]`](../../macros/cgp_producer.md) — builds a `Producer` provider from a zero-argument
  function.
- [Handler combinators](../../providers/handler/index.md) — the `Promote` provider and the `PromoteProducer`
  table that lift a producer into the rest of the family.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.

## Source

- `Producer`:
  [`produce.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/produce.rs)
- The `Promote` combinator and the `PromoteProducer` table:
  [`cgp-handler/src/providers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-handler/src/)
- Re-exported through `cgp::extra::handler`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
