---
title: 'MonadicTrans — stack one monad on another'
sidebar_label: 'MonadicTrans'
sidebar_position: 4
description: 'The monad trait applying one monad as a transformer over another, so a pipeline can peel a nested Result; PipeMonadic uses it for fallible steps.'
---

# `MonadicTrans`

Stacking one monad on top of another.

:::info

### Generated machinery

**You are not expected to name `MonadicTrans`.**
[`PipeMonadic`](../../providers/monad/pipe_monadic.md) resolves a stacked monad through it before any binding
happens, and the transformer markers CGP ships already implement it. The one case for naming it is giving
a monad of your own a transformer form; otherwise this page is here to explain how monads stack.

:::

## Overview

A [monadic pipeline](/docs/concepts/monadic-handlers) over a nested output, a `Result` inside a
`Result`, needs to peel more than one layer, and hard-coding a depth would give up the composability
the design exists for. `MonadicTrans` is how a monad is expressed as a *transformer* over another.

It is one of four traits that give a monad marker its meaning, and it belongs with
[`MonadicBind`](./monadic_bind.md) as the pipeline-folding half: those two decide the *shape* of the
composed provider, while [`ContainsValue`](./contains_value.md) and [`LiftValue`](./lift_value.md) run
each step.

**This is a plain trait, not a CGP component.** It lacks a generated provider trait and is
never wired.

## Definition

`MonadicTrans` carries a single associated type and nothing else:

```rust
pub trait MonadicTrans<M> {
    type M;
}
```

`Self` is the monad being applied, the `M` parameter is the base monad being transformed, and the
associated `M`, which shares the parameter's name, is the resulting stack. Applying `OkMonadic` to
`ErrMonadic` gives `OkMonadicTrans<ErrMonadic>`, for a pipeline that operates over a nested result:
the err monad handles the outer `Result` of each output, and the ok layer the `Result` inside it.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::monad::traits`:

```rust
use cgp::extra::monad::traits::MonadicTrans;
```

You import it only when **defining a monad of your own**, and specifically when giving it a
transformer form so it can stack. The trait lacks a method: it is a type-level function from a base
monad to a composed one.

## Examples

What applying each shipped marker as a transformer produces, checked as type equalities:

```rust
use cgp::prelude::*;
use cgp::extra::monad::monadic::err::{ErrMonadic, ErrMonadicTrans};
use cgp::extra::monad::monadic::ident::IdentMonadic;
use cgp::extra::monad::monadic::ok::{OkMonadic, OkMonadicTrans};
use cgp::extra::monad::traits::MonadicTrans;

// `IdentMonadic` leaves the base unchanged.
pub fn ident(
    m: PhantomData<<IdentMonadic as MonadicTrans<ErrMonadic>>::M>,
) -> PhantomData<ErrMonadic> {
    m
}

// `OkMonadic` over `ErrMonadic`, the stack `PipeMonadic` builds for a fallible pipeline.
pub fn ok_over_err(
    m: PhantomData<<OkMonadic as MonadicTrans<ErrMonadic>>::M>,
) -> PhantomData<OkMonadicTrans<ErrMonadic>> {
    m
}

// Transformers compose: the outer one wraps whatever the inner one produces.
pub fn nested(
    m: PhantomData<<ErrMonadicTrans<OkMonadic> as MonadicTrans<IdentMonadic>>::M>,
) -> PhantomData<ErrMonadicTrans<OkMonadicTrans<IdentMonadic>>> {
    m
}
```

`IdentMonadic` leaves the base unchanged, so it is the neutral element of a stack as well as of a
pipeline. `OkMonadic` over `ErrMonadic` is the stack
[`PipeMonadic`](../../providers/monad/pipe_monadic.md) builds for a fallible pipeline: the err layer
handles the outer `Result` of each output and the ok layer the `Result` inside it. And the
transformer forms compose, so a stack resolves layer by layer. **Layer order is meaningful**:
`OkMonadicTrans<ErrMonadic>` and `ErrMonadicTrans<OkMonadic>` unwrap their `Result` layers in
opposite orders.

## When to use it

**Reach for the [monad providers](../../providers/monad/index.md), not this trait.** Wiring
[`PipeMonadic`](../../providers/monad/pipe_monadic.md) with a marker, including a stacked one, is how a
pipeline is built.

The reasons to name it are narrow.

- **Defining a new monad that should stack.** The three other traits give a marker meaning on its own;
  this one lets it sit over another. A marker without it works as a base monad and cannot be a
  transformer.
- **Reading a stacked marker in an error.** A fallible pipeline's types carry an
  `OkMonadicTrans<ErrMonadic>`-shaped stack that `PipeMonadic` built with this trait, and knowing it
  is the composition step rather than the running step helps read the bound.

If your pipeline runs over a single-layer output, you do not need a transformer at all. Use the base
marker directly.

## Under the hood

`MonadicTrans` is applied to the **monads** rather than to the handlers, and it runs *before* any
binding does. Its one consumer is [`PipeMonadic`](../../providers/monad/pipe_monadic.md)'s wiring
for the fallible components, which stacks the chosen monad over `ErrMonadic` and promotes each
handler first:

```rust
delegate_components! {
    <
        Provider,
        M1: MonadicTrans<ErrMonadic, M = M2>,
        M2,
        ProvidersA: MapFields<TryPromoteProviders, Mapped = ProvidersB>,
        ProvidersB: BindProviders<M2, Provider = Provider>,
    >
    PipeMonadic<M1, ProvidersA> {
        TryComputerComponent: TryPromote<Provider>,
        HandlerComponent: TryPromote<Provider>,
    }
}
```

A `TryComputer` step returns `Result<Output, Error>`, so under a user's monad `M1` its outputs carry
one more `Result` layer, the context's error; stacking `M1` over `ErrMonadic` lets the error layer
short-circuit on that while `M1` handles the layer inside. The `Computer` and `AsyncComputer`
entries use the chosen monad directly.

Because the transformer forms implement [`ContainsValue`](./contains_value.md) and
[`LiftValue`](./lift_value.md) by delegating to the base monad after handling their own layer, a
two-layer stack unwraps two `Result` layers in order and re-wraps them in reverse, and the same code
serves any depth. The resolved stack is another marker, and everything downstream treats it as
one. The whole fold happens during trait resolution, so a stacked monadic pipeline is not a runtime
structure.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::monad::traits`.

**It is not a component and cannot be wired.** What gets wired is the provider that consumes it.

**A stack must be written in layer order.** `OkMonadicTrans<ErrMonadic>` and
`ErrMonadicTrans<OkMonadic>` are different monads, and swapping them produces a pipeline that
short-circuits on the wrong layer.

**The associated type is also called `M`.** The trait's parameter and its associated type share a name,
which reads oddly in a projection like `<Self as MonadicTrans<M>>::M` and is easy to misread when
debugging a bound.

**A base marker without this impl cannot be transformed**, so a monad of your own used with a
fallible pipeline needs it even if you never stack it by hand.

**A new monad needs all four traits**, and this one only if it should stack.

## Related constructs

- [`MonadicBind`](./monadic_bind.md): the other folding trait, which turns a continuation into a bind
  step.
- [`ContainsValue`](./contains_value.md) and [`LiftValue`](./lift_value.md): the two traits that run one
  step, where these two build the pipeline.
- [Monad providers](../../providers/monad/index.md): `PipeMonadic`, `BindOk`, `BindErr`, and the
  markers, including the transformer forms.
- [`Computer`](../../components/handler/computer.md): the component family a monadic pipeline implements.
- [`Product!`](../../macros/product.md): the type-level list a pipeline's steps are given in.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers): why a pipeline short-circuits and how the monads
  compose.

## Source

- [`monadic_trans.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/traits/monadic_trans.rs):
  `MonadicTrans`
- Per-marker impls: [`cgp-monad/src/monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
