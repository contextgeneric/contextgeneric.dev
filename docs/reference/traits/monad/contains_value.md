---
title: 'ContainsValue — the value a monad threads'
sidebar_label: 'ContainsValue'
sidebar_position: 2
description: 'The monad trait naming the value beneath a step''s output that the monad threads forward: the Ok payload under ErrMonadic, the error under OkMonadic.'
---

# `ContainsValue`

Naming the value a monad threads forward out of a step's output.

:::info

### Generated machinery

**You are not expected to name `ContainsValue`.** The
[bind providers](../../providers/monad/index.md) use it while running a step, and the monad markers CGP
ships already implement it. The one case for naming it is defining a monad of your own; otherwise this
page is here to explain how a step unwraps its input.

:::

## Overview

A step in a [monadic pipeline](/docs/concepts/monadic-handlers) produces a wrapped output, a `Result`,
say, and the next step needs the value *inside* the wrapper for the branch that continues. Which half of
the wrapper that is depends on the monad, so it cannot be hard-coded.

`ContainsValue` is where each monad answers it.

It is one of four traits that give a monad marker its meaning, and it pairs with
[`LiftValue`](./lift_value.md): this one says how to get *out* of the output type, that one how to get
back *in*. The other two, [`MonadicBind`](./monadic_bind.md) and
[`MonadicTrans`](./monadic_trans.md), fold the pipeline rather than run a step of it.

**This is a plain trait, not a CGP component.** It lacks a generated provider trait and is
never wired.

## Definition

`ContainsValue` carries a single associated type and nothing else:

```rust
pub trait ContainsValue<Output> {
    type Value;
}
```

`Self` is the monad marker, `Output` is the full output type a step produces, and `Value` is what is
carried in the branch this monad threads forward. A continuation consumes that value, or a deeper monad
layer unwraps it further.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::monad::traits`:

```rust
use cgp::extra::monad::traits::ContainsValue;
```

You import it only when **defining a monad of your own**. The trait lacks a method: it is a
type-level projection from an output type to the value beneath the wrapper.

## Examples

The value each shipped marker reads beneath an output, including a two-layer stack, checked as type
equalities:

```rust
use cgp::extra::monad::monadic::err::ErrMonadic;
use cgp::extra::monad::monadic::ident::IdentMonadic;
use cgp::extra::monad::monadic::ok::{OkMonadic, OkMonadicTrans};
use cgp::extra::monad::traits::ContainsValue;

pub fn ident(value: <IdentMonadic as ContainsValue<u8>>::Value) -> u8 {
    value
}

// `ErrMonadic` continues on `Ok`, so the value is the `Ok` payload.
pub fn err(value: <ErrMonadic as ContainsValue<Result<u8, String>>>::Value) -> u8 {
    value
}

// `OkMonadic` continues on `Err`, so the value is the error.
pub fn ok(value: <OkMonadic as ContainsValue<Result<u8, String>>>::Value) -> String {
    value
}

// The outer layer is `ErrMonadic`'s, the inner one the `Ok` layer's.
pub fn stacked(
    value: <OkMonadicTrans<ErrMonadic> as ContainsValue<Result<Result<u8, String>, bool>>>::Value,
) -> String {
    value
}
```

Under `ErrMonadic` the value is the `Ok` payload and under `OkMonadic` the error, the mirror image
that is the whole design. The transformer form peels one layer after its base: `ErrMonadic` unwraps
the outer `Result` to `Result<u8, String>`, and the `Ok` layer then takes its error, `String`. An
*n*-layer stack unwraps *n* layers the same way, without code specific to any depth.

## When to use it

**Reach for the [monad providers](../../providers/monad/index.md), not this trait.** A pipeline is
built by wiring [`PipeMonadic`](../../providers/monad/pipe_monadic.md) with a marker and a handler
list; this is what the bind providers bound on internally, to relate each continuation's output to
the step's.

The one real reason to name it is **defining a new monad**: short-circuiting over an `Option`, or over a
custom two-branch enum. Implement it alongside [`LiftValue`](./lift_value.md), since the two are the pair
that runs a step and neither is useful without the other, and copy the mirror-image
`OkMonadic`/`ErrMonadic` pair as the model.

If you do not need short-circuiting at all, the
[handler combinators](../../providers/handler/index.md) chain steps without a branch and involve none
of this.

## Under the hood

The bind providers bound on `ContainsValue` in their
[`Computer`](../../components/handler/computer.md) and `AsyncComputer` impls. `BindErr`'s, the step
`ErrMonadic` builds, is:

```rust
#[cgp_provider]
impl<Context, Code, T1, T2, E, M, Cont> Computer<Context, Code, Result<T1, E>> for BindErr<M, Cont>
where
    Cont: Computer<Context, Code, T1>,
    M: ContainsValue<Cont::Output, Value = Result<T2, E>> + LiftValue<Result<T2, E>, Cont::Output>,
{
    type Output = M::Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: Result<T1, E>) -> Self::Output {
        match input {
            Ok(value) => M::lift_output(Cont::compute(context, code, value)),
            Err(err) => M::lift_value(Err(err)),
        }
    }
}
```

The step matches the incoming `Result` itself; `ContainsValue` is not what unwraps it. What this
trait does is constrain the **continuation's output**: `M` is the monad the step sits inside
(`IdentMonadic` for a single-layer pipeline), and
`M: ContainsValue<Cont::Output, Value = Result<T2, E>>` requires that what `M` sees beneath the
continuation's output is a `Result` with the same error type `E`. That is what lets the
short-circuit branch lift its `Err(err)` into the same output type the continue branch produces.

[`LiftValue`](./lift_value.md) is the other half of the bound, and supplies the two functions the
branches call. Because the transformer forms implement both by delegating to the base monad after
handling their own layer, the constraint composes without any depth-specific code.

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

- [`LiftValue`](./lift_value.md): the other half of running a step: getting back into the output type.
- [`MonadicBind`](./monadic_bind.md) and [`MonadicTrans`](./monadic_trans.md): the two traits that fold
  the pipeline rather than run a step.
- [Monad providers](../../providers/monad/index.md): `PipeMonadic`, `BindOk`, `BindErr`, and the
  markers; what you actually wire.
- [`Computer`](../../components/handler/computer.md): the component family the bind providers implement.
- [`TryComputer`](../../components/handler/try_computer.md) and [`Handler`](../../components/handler/handler.md): the fallible
  members a single step usually is.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers): why a pipeline short-circuits and how the monads
  compose.

## Source

- [`value.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/traits/value.rs):
  `ContainsValue`
- Per-marker impls: [`cgp-monad/src/monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
