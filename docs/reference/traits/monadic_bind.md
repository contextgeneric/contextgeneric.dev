---
sidebar_label: 'MonadicBind'
---

# `MonadicBind`

Turning a continuation into one bind step of a monadic pipeline.

## What it's for

A [monadic pipeline](/docs/concepts/monadic-handlers) chains steps where each may either continue or
short-circuit — the familiar `?`-style behaviour, expressed as composable providers rather than as
syntax. Folding such a pipeline means asking the monad, once per step: *given everything built so far,
what provider runs one bind?*

`MonadicBind` is that question:

```rust
pub trait MonadicBind<Provider> {
    type Provider;
}
```

`Self` is the monad marker. The parameter is the **continuation** — the handler that should run on the
continue branch — and the associated type is the **bind provider** that wraps it. For the base monads
this resolves to a [`BindOk` or `BindErr`](../providers/monad_providers.md) provider; for `IdentMonadic`
it is the continuation unchanged, which is what makes the identity monad free.

It is one of four traits that give a monad marker its meaning, alongside
[`ContainsValue`](./contains_value.md), [`LiftValue`](./lift_value.md), and
[`MonadicTrans`](./monadic_trans.md). Keeping them apart is what lets one marker serve all four roles.

**This is a plain capability trait, not a CGP component.** It has no generated provider trait, no
`…Component` marker, and is never wired through
[`delegate_components!`](../macros/delegate_components.md) — the
[monad providers](../providers/monad_providers.md) consume it as an ordinary trait bound while folding a
pipeline at compile time.

## Using it

**It is not in the prelude.** Import it from `cgp::extra::monad::traits`:

```rust
use cgp::extra::monad::traits::MonadicBind;
```

In practice you import it only when **defining a monad of your own**. Using the existing ones means
naming [`PipeMonadic`](../providers/monad_providers.md) and a marker in a wiring entry, with no trait in
sight.

There is no method. The whole trait is a type-level function from a continuation to the provider that
binds it.

## Examples

The trait is consumed rather than called, so the thing to read is what each marker's impl *says* —
because that is what decides how a pipeline behaves.

**`IdentMonadic` implements it as the identity**: `MonadicBind<Provider>::Provider` is `Provider`
unchanged. Nothing wraps the continuation, nothing branches, and a pipeline under `IdentMonadic` is
therefore just function composition.

**`ErrMonadic` and `OkMonadic` wrap the continuation in a bind provider**, and the two are mirror images
over a `Result`:

| | continue branch | short-circuits on | bind provider |
|---|---|---|---|
| `ErrMonadic` | `Ok` | `Err` | `BindOk` |
| `OkMonadic` | `Err` | `Ok` | `BindErr` |

So `ErrMonadic` gives the ordinary `?` behaviour, and `OkMonadic` the inverted one that runs until
something succeeds — useful for a fallback chain.

**The transformer forms delegate one layer down.** `OkMonadicTrans<M>` and `ErrMonadicTrans<M>` bind their
own `Result` layer and hand the rest to `M`, which is how a stack reaches arbitrary depth.

## When to reach for it, and when not

**Reach for the [monad providers](../providers/monad_providers.md), not this trait.** Building a pipeline
means wiring `PipeMonadic` with a monad marker and a handler list; this is what that provider bounds on
internally.

The one real reason to name it is **defining a new monad**, where you implement all four traits for a new
marker. Implementing this one alone yields a marker that folds a pipeline and then fails to resolve when a
step tries to run, because the running half is [`ContainsValue`](./contains_value.md) and
[`LiftValue`](./lift_value.md).

And the alternatives to prefer when you do *not* need short-circuiting:

- **[Handler combinators](../providers/handler_combinators.md)** — `ComposeHandlers` and `PipeHandlers`
  chain handlers without a branch. If every step runs unconditionally, do not involve a monad at all.
- **Ordinary `?` in one provider body** — if the whole chain lives in a single implementation, Rust's own
  operator is clearer than any composition.

## Under the hood

:::note

### Advanced

This section shows where the fold uses it.

:::

[`PipeMonadic`](../providers/monad_providers.md) walks the handler list and, for each step, asks the monad
to turn the continuation built so far into a bind step — which is this trait. Because the walk proceeds
from the end of the list backwards, `Provider` at each stage is everything that follows the current step,
and the result is a single nested provider by the time the list is exhausted.

[`MonadicTrans`](./monadic_trans.md) is what applies one monad as a transformer over another while that
fold happens, so a stacked monad is resolved *before* any binding does. The pair is therefore the
list-folding half of the four traits, while [`ContainsValue`](./contains_value.md) and
[`LiftValue`](./lift_value.md) are the step-running half.

The whole fold happens during trait resolution. A monadic pipeline is not a runtime structure, and the
provider it resolves to implements the [`Computer`](../components/computer.md) family like any other
handler.

## Gotchas

**It is not in the prelude.** Import from `cgp::extra::monad::traits`.

**It is not a component and cannot be wired.** There is no `…Component` marker, so it never appears in a
[`delegate_components!`](../macros/delegate_components.md) block. What gets wired is the provider that
consumes it.

**`OkMonadic` short-circuits on `Ok`, not on `Err`.** The naming reads as "the monad *for* `Ok`" and means
"the monad whose continue branch is `Err`". [`ErrMonadic`](../providers/monad_providers.md) is the one
that behaves like `?`. Getting these the wrong way round produces a pipeline that runs exactly when you
expected it to stop.

**A new monad needs all four traits, plus the transformer form to stack.** Implementing three of them
yields a marker that works in some positions and fails to resolve in others, with an error naming the
missing trait rather than the gap.

## Related constructs

- [`ContainsValue`](./contains_value.md) and [`LiftValue`](./lift_value.md) — the two traits that run one
  bind step, where this one only builds it.
- [`MonadicTrans`](./monadic_trans.md) — stacking one monad on another during the same fold.
- [Monad providers](../providers/monad_providers.md) — `PipeMonadic`, `BindOk`, `BindErr`, and the monad
  markers; what you actually wire.
- [Handler combinators](../providers/handler_combinators.md) — composition without a short-circuit
  branch.
- [`Computer`](../components/computer.md) — the component family a monadic pipeline implements.
- [`Product!`](../macros/product.md) — the type-level list a pipeline's steps are given in.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — why a pipeline short-circuits and how the monads
  compose.

## Source

- [`bind.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-monad/src/traits/bind.rs)
  — `MonadicBind`
- Per-marker impls: [`cgp-monad/src/monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
