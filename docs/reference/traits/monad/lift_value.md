---
title: 'LiftValue — wrap a value into an output'
sidebar_label: 'LiftValue'
sidebar_position: 3
description: 'The monad trait that puts a value into a step''s output on either branch: lift_value for a short-circuit, lift_output for a continuation''s result.'
---

# `LiftValue`

Putting a value back into a monad's output type, on either branch.

:::info

### Generated machinery

**You are not expected to name `LiftValue`.** The
[bind providers](../../providers/monad/index.md) use it while running a step, and the monad markers CGP
ships already implement it. The one case for naming it is defining a monad of your own; otherwise this
page is here to explain how a step produces its output, and why that takes two methods rather than
one.

:::

## Overview

A bind step in a [monadic pipeline](/docs/concepts/monadic-handlers) unwraps a value, runs a
continuation or short-circuits, and then has to produce the step's own output. Producing it means
wrapping something back up, and there are **two** things that might need wrapping, so this trait has two
methods rather than one.

It is the counterpart of [`ContainsValue`](./contains_value.md): that one says how to get *out* of an
output type, this one how to get back *in*. Together they run one step; the other two monad traits,
[`MonadicBind`](./monadic_bind.md) and [`MonadicTrans`](./monadic_trans.md), fold the pipeline instead.

**This is a plain trait, not a CGP component.** It lacks a generated provider trait and is
never wired.

## Definition

`LiftValue` carries an associated type and two methods:

```rust
pub trait LiftValue<Value, Output> {
    type Output;

    fn lift_value(value: Value) -> Self::Output;
    fn lift_output(output: Output) -> Self::Output;
}
```

`Self` is the monad marker, `Value` is the bare value, and the `Output` parameter is the inner output
being forwarded. The associated `Output`, which shares the parameter's name and is not the same thing,
is the step's final output type. `lift_value` wraps a bare value from the continue or short-circuit
branch into that final output; `lift_output` re-wraps a value already in the inner shape, forwarding a
continuation's computed result.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::monad::traits`:

```rust
use cgp::extra::monad::traits::LiftValue;
```

You import it only when **defining a monad of your own**. `Self` is the monad marker, `Value` is the bare
value, `Output` is the inner output being forwarded, and the associated `Output` is the step's final
output type.

## Examples

Each shipped marker's two lifting functions, called directly:

```rust
use cgp::extra::monad::monadic::err::ErrMonadic;
use cgp::extra::monad::monadic::ident::IdentMonadic;
use cgp::extra::monad::monadic::ok::OkMonadic;
use cgp::extra::monad::traits::LiftValue;

pub fn demo() {
    assert_eq!(<IdentMonadic as LiftValue<u8, u8>>::lift_value(1), 1);

    // A bare value enters an `ErrMonadic` output as `Ok`, and an `OkMonadic` one as `Err`.
    let lifted: Result<u8, String> =
        <ErrMonadic as LiftValue<u8, Result<u8, String>>>::lift_value(1);
    assert_eq!(lifted, Ok(1));

    let lifted: Result<u8, String> =
        <OkMonadic as LiftValue<String, Result<u8, String>>>::lift_value("no".to_owned());
    assert_eq!(lifted, Err("no".to_owned()));

    // An output already in the inner shape is forwarded unchanged.
    let forwarded =
        <ErrMonadic as LiftValue<u8, Result<u8, String>>>::lift_output(Err("stop".to_owned()));
    assert_eq!(forwarded, Err("stop".to_owned()));
}
```

Under `ErrMonadic` a bare value enters the output as `Ok(value)` and under `OkMonadic` as
`Err(value)`, while `lift_output` forwards an output already in the inner shape unchanged. **Both
methods matter, and conflating them changes behaviour silently**: a bind step lifts the
short-circuit value with `lift_value` and forwards the continuation's result with `lift_output`, as
[Under the hood](#under-the-hood) shows.

## When to use it

**Reach for the [monad providers](../../providers/monad/index.md), not this trait.** A pipeline is built
by wiring [`PipeMonadic`](../../providers/monad/pipe_monadic.md) with a marker and a handler list.

The one real reason to name it is **defining a new monad**. Implement it alongside
[`ContainsValue`](./contains_value.md), since the two are the pair that runs a step, and take the
`OkMonadic`/`ErrMonadic` pair as the model, including their separate treatment of the two methods, which
is the part a first implementation usually gets wrong.

If short-circuiting is not what you need, the [handler combinators](../../providers/handler/index.md)
chain steps without a branch and involve none of this.

## Under the hood

The bind providers use `LiftValue` in their [`Computer`](../../components/handler/computer.md) and
`AsyncComputer` impls, beside [`ContainsValue`](./contains_value.md). `BindErr`'s, the step
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

The step matches the incoming `Result`, then lifts with `M`, the monad it sits inside:

- on the **short-circuit** branch, the value never reached the continuation, so `Err(err)` is
  wrapped with `lift_value`;
- on the **continue** branch, the continuation has already produced an output in the inner shape, so
  it is forwarded with `lift_output`.

That is the whole reason for two methods. A monad whose two methods agree, as `IdentMonadic`'s do, is
one where the distinction happens to be vacuous, not one where it does not exist.

The transformer forms implement both by delegating to the base monad after handling their own layer
(`ErrMonadicTrans<M>`'s `lift_value` is `M::lift_value(Ok(value))`), which lets a stack lift through
*n* layers without depth-specific code.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::monad::traits`.

**It is not a component and cannot be wired.** What gets wired is the provider that consumes it.

**`lift_value` and `lift_output` are not the same method.** Implementing the second as the first, or
forgetting the distinction, collapses the two branches and silently changes what a pipeline does on a
short-circuit. This is the most likely defect in a hand-written monad.

**`OkMonadic` lifts with `Err`.** Its continue branch is the error half, which follows from
[`ContainsValue`](./contains_value.md) and reads backwards until you have internalized the naming.

**The associated `Output` is not the `Output` parameter.** The parameter is the inner output being
forwarded; the associated type is the step's own. They coincide for some monads and not for others.

**A new monad needs all four traits**, plus the transformer form to stack.

## Related constructs

- [`ContainsValue`](./contains_value.md): the other half of running a step: getting out of the output
  type.
- [`MonadicBind`](./monadic_bind.md) and [`MonadicTrans`](./monadic_trans.md): the two traits that fold
  the pipeline rather than run a step.
- [Monad providers](../../providers/monad/index.md): `PipeMonadic`, `BindOk`, `BindErr`, and the
  markers; what you actually wire.
- [`Computer`](../../components/handler/computer.md): the component family the bind providers implement.
- [Handler combinators](../../providers/handler/index.md): composition without a short-circuit
  branch.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers): why a pipeline short-circuits and how the monads
  compose.

## Source

- [`lift.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/traits/lift.rs):
  `LiftValue`
- Per-marker impls: [`cgp-monad/src/monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
