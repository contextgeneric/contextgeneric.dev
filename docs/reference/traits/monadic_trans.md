---
sidebar_label: 'MonadicTrans'
---

# `MonadicTrans`

Stacking one monad on top of another.

:::info

### Generated machinery

**You are not expected to name `MonadicTrans`.**
[`PipeMonadic`](../providers/monad/pipe_monadic.md) resolves a stacked monad through it before any binding
happens, and the transformer markers CGP ships already implement it. The one case for naming it is giving
a monad of your own a transformer form; otherwise this page is here to explain how monads stack.

:::

## Overview

A [monadic pipeline](/docs/concepts/monadic-handlers) over a nested output — a `Result` inside a
`Result` — needs to peel more than one layer, and hard-coding a depth would give up the composability
the design exists for. `MonadicTrans` is how a monad is expressed as a *transformer* over another:

```rust
pub trait MonadicTrans<M> {
    type M;
}
```

`Self` is the monad being applied, `M` is the base monad being transformed, and the associated `M` is the
resulting stack. That is what lets `OkMonadic` be written as `OkMonadicTrans<ErrMonadic>` when a pipeline
operates over a nested result, layering one behaviour on top of another.

It is one of four traits that give a monad marker its meaning, and it belongs with
[`MonadicBind`](./monadic_bind.md) as the pipeline-folding half: those two decide the *shape* of the
composed provider, while [`ContainsValue`](./contains_value.md) and [`LiftValue`](./lift_value.md) run
each step.

**This is a plain capability trait, not a CGP component.** It has no generated provider trait and is
never wired.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::monad::traits`:

```rust
use cgp::extra::monad::traits::MonadicTrans;
```

You import it only when **defining a monad of your own**, and specifically when giving it a transformer
form so it can stack. There is no method; the trait is a type-level function from a base monad to a
composed one.

## Examples

What each marker's impl says decides how deeply a pipeline can reach.

**`IdentMonadic` returns `M` unchanged**, so applying it as a transformer changes nothing — which is what
makes it the neutral element of a stack as well as of a pipeline.

**The transformer forms compose.** `OkMonadicTrans<M>` and `ErrMonadicTrans<M>` implement the running
traits by peeling their own `Result` layer and handing the rest to `M`, and their `MonadicTrans` impls
compose in the same order, so a stack like `OkMonadicTrans<ErrMonadic>` resolves layer by layer:

```rust
// conceptually: unwrap the outer Result the OkMonadic way,
// then hand the inner Result to ErrMonadic
type Stacked = OkMonadicTrans<ErrMonadic>;
```

An *n*-layer stack unwraps *n* layers with no code specific to any depth, which is the whole payoff.

**Layer order is meaningful.** `OkMonadicTrans<ErrMonadic>` and `ErrMonadicTrans<OkMonadic>` unwrap their
`Result` layers in opposite orders and are not interchangeable.

## When to use it

**Reach for the [monad providers](../providers/monad/index.md), not this trait.** Wiring
[`PipeMonadic`](../providers/monad/pipe_monadic.md) with a marker — including a stacked one — is how a
pipeline is built.

The reasons to name it are two, and both are narrow.

- **Defining a new monad that should stack.** The three other traits give a marker meaning on its own;
  this one is what lets it sit over another. A marker without it works as a base monad and cannot be a
  transformer.
- **Reading a stacked marker in an error.** A resolution failure over a nested result usually names this
  trait, and knowing it is the composition step rather than the running step is what tells you the
  problem is the stack's shape rather than a step's types.

If your pipeline runs over a single-layer output, you do not need a transformer at all — use the base
marker directly.

## Under the hood

`MonadicTrans` is applied to the **monads** rather than to the handlers, and it runs *before* any binding
does. [`PipeMonadic`](../providers/monad/pipe_monadic.md) resolves the stacked monad first, then walks the
handler list asking [`MonadicBind`](./monadic_bind.md) to turn each continuation into a bind step.

Because the transformer forms implement [`ContainsValue`](./contains_value.md) and
[`LiftValue`](./lift_value.md) by delegating to the base monad after handling their own layer, a
two-layer stack unwraps two `Result` layers in order and re-wraps them in reverse — and the same code
serves any depth. That is the sense in which stacking is composition rather than a special case: the
resolved stack is just another marker, and everything downstream treats it as one.

The whole fold happens during trait resolution, so a stacked monadic pipeline is not a runtime structure.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::monad::traits`.

**It is not a component and cannot be wired.** What gets wired is the provider that consumes it.

**A stack must be written in layer order.** `OkMonadicTrans<ErrMonadic>` and
`ErrMonadicTrans<OkMonadic>` are different monads, and swapping them produces a pipeline that
short-circuits on the wrong layer.

**The associated type is also called `M`.** The trait's parameter and its associated type share a name,
which reads oddly in a projection like `<Self as MonadicTrans<M>>::M` and is easy to misread when
debugging a bound.

**A base marker without this impl cannot be transformed.** The error names the missing `MonadicTrans`
bound rather than saying the marker is not stackable.

**A new monad needs all four traits**, and this one only if it should stack.

## Related constructs

- [`MonadicBind`](./monadic_bind.md) — the other folding trait, which turns a continuation into a bind
  step.
- [`ContainsValue`](./contains_value.md) and [`LiftValue`](./lift_value.md) — the two traits that run one
  step, where these two build the pipeline.
- [Monad providers](../providers/monad/index.md) — `PipeMonadic`, `BindOk`, `BindErr`, and the
  markers, including the transformer forms.
- [`Computer`](../components/handler/computer.md) — the component family a monadic pipeline implements.
- [`Product!`](../macros/product.md) — the type-level list a pipeline's steps are given in.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — why a pipeline short-circuits and how the monads
  compose.

## Source

- [`monadic_trans.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/traits/monadic_trans.rs)
  — `MonadicTrans`
- Per-marker impls: [`cgp-monad/src/monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
