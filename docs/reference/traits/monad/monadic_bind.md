---
title: 'MonadicBind — build one bind step'
sidebar_label: 'MonadicBind'
sidebar_position: 1
description: 'The monad trait that turns a continuation provider into the bind step running it, BindErr for ErrMonadic and BindOk for OkMonadic.'
---

# `MonadicBind`

Turning a continuation into one bind step of a monadic pipeline.

:::info

### Generated machinery

**You are not expected to name `MonadicBind`.**
[`PipeMonadic`](../../providers/monad/pipe_monadic.md) bounds on it while folding a pipeline, and the monad
markers CGP ships already implement it. The one case for naming it is defining a monad of your own;
otherwise this page is here to explain how a pipeline is assembled.

:::

## Overview

A [monadic pipeline](/docs/concepts/monadic-handlers) chains steps where each may either continue or
short-circuit, the familiar `?`-style behaviour, expressed as composable providers rather than as
syntax. Folding such a pipeline means asking the monad, once per step: *given everything built so far,
what provider runs one bind?*

`MonadicBind` is that question.

It is one of four traits that give a monad marker its meaning, alongside
[`ContainsValue`](./contains_value.md), [`LiftValue`](./lift_value.md), and
[`MonadicTrans`](./monadic_trans.md). Keeping them apart lets one marker serve all four roles.

**This is a plain trait, not a CGP component.** It lacks a generated provider trait and a
`…Component` marker, and is never wired through
[`delegate_components!`](../../macros/delegate_components.md): the [monad
providers](../../providers/monad/index.md) consume it as an ordinary trait bound while folding a
pipeline at compile time.

## Definition

`MonadicBind` carries a single associated type and nothing else:

```rust
pub trait MonadicBind<Provider> {
    type Provider;
}
```

`Self` is the monad marker. The `Provider` parameter is the **continuation**, the handler that should
run on the continue branch, and the `Provider` associated type is the **bind provider** that wraps it.
For the base monads this resolves to a [`BindOk` or `BindErr`](../../providers/monad/index.md) provider;
for `IdentMonadic` it is the continuation unchanged, which makes the identity monad free.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::monad::traits`:

```rust
use cgp::extra::monad::traits::MonadicBind;
```

In practice you import it only when **defining a monad of your own**. Using the existing ones means
naming [`PipeMonadic`](../../providers/monad/pipe_monadic.md) and a marker in a wiring entry,
without naming a trait.

The trait lacks a method: it is a type-level function from a continuation to the provider that
binds it.

## Examples

The bind provider each shipped marker wraps a continuation in, checked as type equalities:

```rust
use cgp::prelude::*;
use cgp::extra::monad::monadic::err::{BindErr, ErrMonadic};
use cgp::extra::monad::monadic::ident::IdentMonadic;
use cgp::extra::monad::monadic::ok::{BindOk, OkMonadic};
use cgp::extra::monad::traits::MonadicBind;

// A continuation provider; any type stands in for one here.
pub struct Next;

// Each function compiles only if the projection is the type on its right.
pub fn ident(
    p: PhantomData<<IdentMonadic as MonadicBind<Next>>::Provider>,
) -> PhantomData<Next> {
    p
}

pub fn err(
    p: PhantomData<<ErrMonadic as MonadicBind<Next>>::Provider>,
) -> PhantomData<BindErr<IdentMonadic, Next>> {
    p
}

pub fn ok(
    p: PhantomData<<OkMonadic as MonadicBind<Next>>::Provider>,
) -> PhantomData<BindOk<IdentMonadic, Next>> {
    p
}
```

**`IdentMonadic` implements it as the identity**: the continuation comes back unchanged, so nothing
branches, and a pipeline under `IdentMonadic` is plain composition. **`ErrMonadic` and `OkMonadic`
wrap the continuation in a bind provider**, and the two are mirror images over a `Result`:

| | continue branch | short-circuits on | bind provider |
|---|---|---|---|
| `ErrMonadic` | `Ok` | `Err` | `BindErr` |
| `OkMonadic` | `Err` | `Ok` | `BindOk` |

Each bind provider is named for the branch it stops on. So `ErrMonadic` gives the ordinary `?`
behaviour, and `OkMonadic` the inverted one that runs until something succeeds, useful for a
fallback chain. **The transformer forms delegate to their base**: `ErrMonadicTrans<M>` asks `M` to
bind a `BindErr<M, Provider>`, which is how a stack reaches arbitrary depth.

## When to use it

**Reach for the [monad providers](../../providers/monad/index.md), not this trait.** Building a pipeline
means wiring `PipeMonadic` with a monad marker and a handler list, and that provider bounds on this
trait internally.

The one real reason to name it is **defining a new monad**, where you implement all four traits for a new
marker. Implementing this one alone yields a marker that folds a pipeline and then fails to resolve when a
step tries to run, because the running half is [`ContainsValue`](./contains_value.md) and
[`LiftValue`](./lift_value.md).

And the alternatives to prefer when you do *not* need short-circuiting:

- **[Handler combinators](../../providers/handler/index.md)**: `ComposeHandlers` and `PipeHandlers`
  chain handlers without a branch. If every step runs unconditionally, do not involve a monad at all.
- **Ordinary `?` in one provider body**: if the whole chain lives in a single implementation, Rust's own
  operator is clearer than any composition.

## Under the hood

[`PipeMonadic`](../../providers/monad/pipe_monadic.md) folds its handler list with a private helper
trait, and this trait is what the fold asks at each step:

```rust
impl<M, ProviderA, ProviderB, RestProviders, OutProviders> BindProviders<M>
    for Cons<ProviderA, Cons<ProviderB, RestProviders>>
where
    Cons<ProviderB, RestProviders>: BindProviders<M, Provider = OutProviders>,
    M: MonadicBind<OutProviders>,
{
    type Provider = ComposeHandlers<ProviderA, M::Provider>;
}

impl<M, Provider> BindProviders<M> for Cons<Provider, Nil> {
    type Provider = Provider;
}
```

The fold recurses into the rest of the list first, so `OutProviders` at each stage is everything
that follows the current step, and the monad binds it: the first handler runs plainly and its output
is handed to a bind step over the rest. By the time the list is exhausted the result is one nested
provider, built from [`ComposeHandlers`](../../providers/handler/compose_handlers.md) and the bind
providers.

For the fallible components, `PipeMonadic` first stacks the chosen monad over `ErrMonadic` with
[`MonadicTrans`](./monadic_trans.md), so a stacked monad is resolved *before* any binding does. The
pair is therefore the list-folding half of the four traits, while
[`ContainsValue`](./contains_value.md) and [`LiftValue`](./lift_value.md) are the step-running half.

The whole fold happens during trait resolution. A monadic pipeline is not a runtime structure, and
the provider it resolves to implements the [`Computer`](../../components/handler/computer.md) family
like any other handler.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::monad::traits`.

**It is not a component and cannot be wired.** It lacks a `…Component` marker, so it never appears
in a [`delegate_components!`](../../macros/delegate_components.md) block. What gets wired is the
provider that consumes it.

**`OkMonadic` short-circuits on `Ok`, not on `Err`.** The naming reads as "the monad *for* `Ok`" and means
"the monad whose continue branch is `Err`". [`ErrMonadic`](../../providers/monad/index.md) is the one
that behaves like `?`. Getting these the wrong way round produces a pipeline that runs exactly when you
expected it to stop.

**A new monad needs all four traits, plus the transformer form to stack.** Implementing three of them
yields a marker that works in some positions and fails to resolve in others, with an error naming the
missing trait rather than the gap.

## Related constructs

- [`ContainsValue`](./contains_value.md) and [`LiftValue`](./lift_value.md): the two traits that run one
  bind step, where this one only builds it.
- [`MonadicTrans`](./monadic_trans.md): stacking one monad on another during the same fold.
- [Monad providers](../../providers/monad/index.md): `PipeMonadic`, `BindOk`, `BindErr`, and the monad
  markers; what you actually wire.
- [Handler combinators](../../providers/handler/index.md): composition without a short-circuit
  branch.
- [`Computer`](../../components/handler/computer.md): the component family a monadic pipeline implements.
- [`Product!`](../../macros/product.md): the type-level list a pipeline's steps are given in.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers): why a pipeline short-circuits and how the monads
  compose.

## Source

- [`bind.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/traits/bind.rs):
  `MonadicBind`
- Per-marker impls: [`cgp-monad/src/monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
