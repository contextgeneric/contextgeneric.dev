---
sidebar_label: 'ContainsValue'
---

# `ContainsValue`

Naming the value a monad threads forward out of a step's output.

:::info

### Generated machinery

**You are not expected to name `ContainsValue`.** The
[bind providers](../providers/monad/index.md) use it while running a step, and the monad markers CGP
ships already implement it. The one case for naming it is defining a monad of your own; otherwise this
page is here to explain how a step unwraps its input.

:::

## Overview

A step in a [monadic pipeline](/docs/concepts/monadic-handlers) produces a wrapped output — a `Result`,
say — and the next step needs the value *inside* the wrapper for the branch that continues. Which half of
the wrapper that is depends on the monad, so it cannot be hard-coded.

`ContainsValue` is where each monad answers it:

```rust
pub trait ContainsValue<Output> {
    type Value;
}
```

`Self` is the monad marker, `Output` is the full output type a step produces, and `Value` is what is
carried in the branch this monad threads forward. A continuation consumes that value, or a deeper monad
layer unwraps it further.

It is one of four traits that give a monad marker its meaning, and it pairs with
[`LiftValue`](./lift_value.md): this one says how to get *out* of the output type, that one how to get
back *in*. The other two, [`MonadicBind`](./monadic_bind.md) and
[`MonadicTrans`](./monadic_trans.md), fold the pipeline rather than run a step of it.

**This is a plain capability trait, not a CGP component.** It has no generated provider trait and is
never wired.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::monad::traits`:

```rust
use cgp::extra::monad::traits::ContainsValue;
```

You import it only when **defining a monad of your own**. There is no method — the trait is a type-level
projection from an output type to the value beneath the wrapper.

## Examples

What each marker's impl says is what decides how a pipeline behaves.

**`IdentMonadic` is the identity**: `ContainsValue<T>::Value` is `T`. Nothing is wrapped, so nothing is
unwrapped, and every value threads forward.

**`ErrMonadic` and `OkMonadic` are mirror images over a `Result`**, and the mirroring is the whole design:

| Monad | `ContainsValue<Result<T, E>>::Value` | continue branch |
|---|---|---|
| `ErrMonadic` | `T` | the `Ok` payload threads forward |
| `OkMonadic` | `E` | the `Err` payload threads forward |

So under `ErrMonadic` a step's continuation receives the `Ok` payload — the ordinary `?` behaviour — and
under `OkMonadic` it receives the `Err` payload, which is the inverted, run-until-something-succeeds
behaviour.

**The transformer forms peel one layer.** `OkMonadicTrans<M>` and `ErrMonadicTrans<M>` remove their own
`Result` layer and hand the rest to `M`, requiring `M: ContainsValue<V, Value = Result<…>>`. A two-layer
stack therefore unwraps two `Result` layers in order, and an *n*-layer stack unwraps *n*, with no code
specific to any depth.

## When to reach for it, and when not

**Reach for the [monad providers](../providers/monad/index.md), not this trait.** A pipeline is built
by wiring [`PipeMonadic`](../providers/monad/pipe_monadic.md) with a marker and a handler list; this is what
the bind providers bound on internally.

The one real reason to name it is **defining a new monad** — short-circuiting over an `Option`, or over a
custom two-branch enum. Implement it alongside [`LiftValue`](./lift_value.md), since the two are the pair
that runs a step and neither is useful without the other, and copy the mirror-image
`OkMonadic`/`ErrMonadic` pair as the model.

If you do not need short-circuiting at all, the
[handler combinators](../providers/handler/index.md) chain steps without a branch and involve none
of this.

## Under the hood

The [`BindOk` and `BindErr`](../providers/monad/index.md) providers use `ContainsValue` in their
[`Computer`](../components/handler/computer.md) and `AsyncComputer` impls. Running one bind step means: take the
step's output, ask the monad what value sits beneath its wrapper, and hand that to the continuation. This
trait is the second half of that sentence.

[`LiftValue`](./lift_value.md) is then what puts a result back into the output type — `lift_value` for the
branch that short-circuits, `lift_output` for the branch that forwarded to the continuation. So the two
traits bracket one step: unwrap, run, re-wrap.

Because the transformer forms implement it by delegating to the base monad, the unwrapping composes
without any depth-specific code.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::monad::traits`.

**It is not a component and cannot be wired.** What gets wired is the provider that consumes it.

**`OkMonadic`'s `Value` is the error half.** Under `OkMonadic`, `ContainsValue<Result<T, E>>::Value` is
`E`, not `T`. This is the single most confusing consequence of the naming, and it is deliberate:
`OkMonadic` is the monad that *stops* on `Ok`.

**A stack must be written in layer order.** `OkMonadicTrans<ErrMonadic>` and `ErrMonadicTrans<OkMonadic>`
unwrap their `Result` layers in opposite orders and are not interchangeable.

**A new monad needs all four traits.** Implementing this one without [`LiftValue`](./lift_value.md)
yields a marker that can say what a value is and cannot put one back, so a pipeline fails to resolve at
the first step rather than at the definition.

## Related constructs

- [`LiftValue`](./lift_value.md) — the other half of running a step: getting back into the output type.
- [`MonadicBind`](./monadic_bind.md) and [`MonadicTrans`](./monadic_trans.md) — the two traits that fold
  the pipeline rather than run a step.
- [Monad providers](../providers/monad/index.md) — `PipeMonadic`, `BindOk`, `BindErr`, and the
  markers; what you actually wire.
- [`Computer`](../components/handler/computer.md) — the component family the bind providers implement.
- [`TryComputer`](../components/handler/try_computer.md) and [`Handler`](../components/handler/handler.md) — the fallible
  members a single step usually is.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — why a pipeline short-circuits and how the monads
  compose.

## Source

- [`value.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/traits/value.rs)
  — `ContainsValue`
- Per-marker impls: [`cgp-monad/src/monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
