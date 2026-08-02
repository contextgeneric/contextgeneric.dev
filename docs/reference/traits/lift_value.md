---
sidebar_label: 'LiftValue'
---

# `LiftValue`

Putting a value back into a monad's output type, on either branch.

:::info

### Generated machinery

**You are not expected to name `LiftValue`.** The
[bind providers](../providers/monad_providers.md) use it while running a step, and the monad markers CGP
ships already implement it. The one case for naming it is defining a monad of your own; otherwise this
page is here to explain how a step produces its output — and why that takes two methods rather than
one.

:::

## What it's for

A bind step in a [monadic pipeline](/docs/concepts/monadic-handlers) unwraps a value, runs a
continuation or short-circuits, and then has to produce the step's own output. Producing it means
wrapping something back up — and there are **two** things that might need wrapping, which is why this
trait has two methods rather than one:

```rust
pub trait LiftValue<Value, Output> {
    type Output;

    fn lift_value(value: Value) -> Self::Output;
    fn lift_output(output: Output) -> Self::Output;
}
```

`lift_value` wraps a bare value from the continue or short-circuit branch into the final output.
`lift_output` re-wraps a value already in the inner shape, forwarding a continuation's computed result.

It is the counterpart of [`ContainsValue`](./contains_value.md): that one says how to get *out* of an
output type, this one how to get back *in*. Together they run one step; the other two monad traits,
[`MonadicBind`](./monadic_bind.md) and [`MonadicTrans`](./monadic_trans.md), fold the pipeline instead.

**This is a plain capability trait, not a CGP component.** It has no generated provider trait and is
never wired.

## Using it

**It is not in the prelude.** Import it from `cgp::extra::monad::traits`:

```rust
use cgp::extra::monad::traits::LiftValue;
```

You import it only when **defining a monad of your own**. `Self` is the monad marker, `Value` is the bare
value, `Output` is the inner output being forwarded, and the associated `Output` is the step's final
output type.

## Examples

What each marker's impl says is what decides how a pipeline behaves.

**`IdentMonadic` is the identity in both directions**: `LiftValue<T, T>` returns `T` from both methods.
Nothing is wrapped, so a pipeline under it is plain function composition.

**`ErrMonadic` and `OkMonadic` mirror each other** over a `Result`, and the `lift_value` constructor is
where the mirroring shows:

| Monad | `lift_value` wraps with | continue branch |
|---|---|---|
| `ErrMonadic` | `Ok` | `Ok` |
| `OkMonadic` | `Err` | `Err` |

So under `ErrMonadic` a bare value re-enters the pipeline as `Ok(value)` — the ordinary `?` behaviour —
and under `OkMonadic` as `Err(value)`.

**Both methods matter, and conflating them changes behaviour silently.** `lift_value` is used where a
bare value is being wrapped for the first time; `lift_output` where a continuation has already produced
something in the inner shape and the step is passing it outward. Implementing the second as the first
collapses the two branches.

## When to reach for it, and when not

**Reach for the [monad providers](../providers/monad_providers.md), not this trait.** A pipeline is built
by wiring [`PipeMonadic`](../providers/monad_providers.md) with a marker and a handler list.

The one real reason to name it is **defining a new monad**. Implement it alongside
[`ContainsValue`](./contains_value.md), since the two are the pair that runs a step, and take the
`OkMonadic`/`ErrMonadic` pair as the model — including their separate treatment of the two methods, which
is the part a first implementation usually gets wrong.

If short-circuiting is not what you need, the [handler combinators](../providers/handler_combinators.md)
chain steps without a branch and involve none of this.

## Under the hood

:::note

### Advanced

This section shows where a bind step uses it.

:::

The [`BindOk` and `BindErr`](../providers/monad_providers.md) providers use `LiftValue` in their
[`Computer`](../components/computer.md) and `AsyncComputer` impls, as the closing half of a step whose
opening half is [`ContainsValue`](./contains_value.md). The step unwraps the incoming output, decides
whether to short-circuit, and then lifts:

- on the **short-circuit** branch, the value never reached the continuation, so it is wrapped with
  `lift_value`;
- on the **continue** branch, the continuation has already produced an inner output, so it is forwarded
  with `lift_output`.

That is the whole reason for two methods. A monad whose two methods agree — as `IdentMonadic`'s do — is
one where the distinction happens to be vacuous, not one where it does not exist.

The transformer forms implement both by delegating to the base monad after handling their own layer,
which is what lets a stack lift through *n* layers with no depth-specific code.

## Gotchas

**It is not in the prelude.** Import from `cgp::extra::monad::traits`.

**It is not a component and cannot be wired.** What gets wired is the provider that consumes it.

**`lift_value` and `lift_output` are not the same method.** Implementing the second as the first — or
forgetting the distinction — collapses the two branches and silently changes what a pipeline does on a
short-circuit. This is the most likely defect in a hand-written monad.

**`OkMonadic` lifts with `Err`.** Its continue branch is the error half, which follows from
[`ContainsValue`](./contains_value.md) and reads backwards until you have internalized the naming.

**The associated `Output` is not the `Output` parameter.** The parameter is the inner output being
forwarded; the associated type is the step's own. They coincide for some monads and not for others.

**A new monad needs all four traits**, plus the transformer form to stack.

## Related constructs

- [`ContainsValue`](./contains_value.md) — the other half of running a step: getting out of the output
  type.
- [`MonadicBind`](./monadic_bind.md) and [`MonadicTrans`](./monadic_trans.md) — the two traits that fold
  the pipeline rather than run a step.
- [Monad providers](../providers/monad_providers.md) — `PipeMonadic`, `BindOk`, `BindErr`, and the
  markers; what you actually wire.
- [`Computer`](../components/computer.md) — the component family the bind providers implement.
- [Handler combinators](../providers/handler_combinators.md) — composition without a short-circuit
  branch.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — why a pipeline short-circuits and how the monads
  compose.

## Source

- [`lift.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/traits/lift.rs)
  — `LiftValue`
- Per-marker impls: [`cgp-monad/src/monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
