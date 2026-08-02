---
sidebar_label: 'Monad traits'
---

# Monad traits

`MonadicTrans`, `MonadicBind`, `LiftValue`, and `ContainsValue`.

## What it's for

A [monadic pipeline](/docs/concepts/monadic-handlers) chains a list of steps where each step may either continue
or short-circuit — the familiar `?`-style behaviour, expressed as composable providers rather than as syntax. To
build one, the machinery needs to know three things about the monad it is running under: which branch of an
output continues, which branch stops, and how values move between them.

A monad in CGP is a zero-sized marker — `IdentMonadic`, `OkMonadic`, `ErrMonadic`, and their transformer forms —
and these four traits are what give that marker meaning. Each answers one of the separable questions:

- **`MonadicBind`** — given a continuation, what provider runs one bind step?
- **`ContainsValue`** — for a given output type, what value sits beneath this monad's wrapper?
- **`LiftValue`** — how does a value get back *into* that output type?
- **`MonadicTrans`** — how does this monad stack on top of another?

Keeping them apart is what lets one marker serve all four roles, and what lets monads stack by composing their
implementations.

**These are plain capability traits, not CGP components.** They have no generated provider trait, no
`…Component` marker, and are never wired through
[`delegate_components!`](../macros/delegate_components.md) — the [monad providers](../providers/monad_providers.md)
consume them as ordinary trait bounds while folding a pipeline at compile time.

## Using it

**None of the four is in the prelude.** Import them from `cgp::extra::monad::traits`. In practice you will import
them only when *defining a monad of your own*; using the existing ones means naming
[`PipeMonadic`](../providers/monad_providers.md) and a marker in a wiring entry, with no trait in sight.

### `MonadicBind`

```rust
pub trait MonadicBind<Provider> {
    type Provider;
}
```

The parameter is the continuation — the handler that should run on the continue branch — and the associated type
is the bind provider that wraps it. For the base monads this resolves to a `BindOk` or `BindErr` provider; for
`IdentMonadic` it is the continuation unchanged, which is what makes the identity monad free.

### `ContainsValue`

```rust
pub trait ContainsValue<Output> {
    type Value;
}
```

`Output` is the full output type a step produces; `Value` is what is carried in the branch this monad threads
forward. A continuation consumes that value, or a deeper monad layer unwraps it further.

### `LiftValue`

```rust
pub trait LiftValue<Value, Output> {
    type Output;

    fn lift_value(value: Value) -> Self::Output;
    fn lift_output(output: Output) -> Self::Output;
}
```

Two methods for the two branches a bind step takes. `lift_value` wraps a bare value from the continue or
short-circuit branch into the final output; `lift_output` re-wraps a value already in the inner shape, forwarding
a continuation's computed result.

### `MonadicTrans`

```rust
pub trait MonadicTrans<M> {
    type M;
}
```

`M` is the base monad being transformed and the associated `M` is the resulting stack. This is what lets
`OkMonadic` be written as `OkMonadicTrans<ErrMonadic>` when a pipeline operates over a nested result, layering
one behaviour on top of another.

## Examples

The traits are consumed rather than called, so the thing to read is what each marker's impls *say* — because that
is what decides how a pipeline behaves.

**`IdentMonadic` implements all four as identities**, which is why it threads every value forward and never
short-circuits. `MonadicTrans<M>` returns `M` unchanged, `MonadicBind<Provider>` returns `Provider` unchanged,
`ContainsValue<T>::Value` is `T`, and `LiftValue<T, T>` is the identity in both directions. A pipeline under
`IdentMonadic` is therefore just function composition.

**`ErrMonadic` and `OkMonadic` are mirror images over a `Result`**, and the mirroring is the whole design:

| | continue branch | short-circuits on |
|---|---|---|
| `ErrMonadic` | `Ok` | `Err` |
| `OkMonadic` | `Err` | `Ok` |

For `ErrMonadic`, `ContainsValue<Result<T, E>>::Value` is `T` — the `Ok` payload threads forward — and
`lift_value` is `Ok`. For `OkMonadic` the roles swap: `Value` is `E` and `lift_value` is `Err`. So `ErrMonadic` is
the ordinary `?` behaviour, and `OkMonadic` is the inverted one that runs until something succeeds — useful for a
fallback chain.

**The transformer forms delegate one layer down.** `OkMonadicTrans<M>` and `ErrMonadicTrans<M>` implement the same
traits by peeling their own `Result` layer and handing the rest to `M`, requiring
`M: ContainsValue<V, Value = Result<…>>`. Their `MonadicTrans` impls compose, so a stack like
`OkMonadicTrans<ErrMonadic>` resolves layer by layer — which is how monads reach arbitrary depth over nested
result types.

## When to reach for it, and when not

**Reach for the [monad providers](../providers/monad_providers.md), not these traits.** Building a pipeline means
wiring `PipeMonadic` with a monad marker and a handler list; the traits are what that provider bounds on
internally.

The one real reason to name them is **defining a new monad**. If you need short-circuiting over a type that is not
`Result` — an `Option`, a custom two-branch enum — implementing the four traits for a new marker is the supported
extension point, and the mirror-image `OkMonadic`/`ErrMonadic` pair is the model to copy.

And the alternatives to prefer when you do *not* need short-circuiting:

- **[Handler combinators](../providers/handler_combinators.md)** — `ComposeHandlers` and `PipeHandlers` chain
  handlers without a branch. If every step in your pipeline runs unconditionally, these are simpler and you should
  not involve a monad at all.
- **Ordinary `?` in one provider body** — if the whole chain lives in a single implementation, Rust's own operator
  is clearer than any composition. The monadic providers earn their keep when the *steps are separately wired*, so
  each application can swap or reorder them.
- **[`TryComputer`](../components/try_computer.md) or [`Handler`](../components/handler.md)** — for a single
  fallible step, the fallible members of the handler family already carry the error, and no pipeline is needed.

## Under the hood

:::note

### Advanced

This section shows how a pipeline is folded from these traits. You do not need it to use a monad, but it is what
makes the division into four traits look inevitable rather than arbitrary.

:::

The traits divide the way they do because building a pipeline needs three separable decisions, taken at different
times.

**Folding the list** is `MonadicTrans` and `MonadicBind`. `PipeMonadic` walks the handler list and, for each step,
asks the monad to turn the continuation built so far into a bind step. `MonadicTrans` is what applies one monad as
a transformer over another while doing so, which is how a stacked monad is resolved before any binding happens.

**Running one step** is `ContainsValue` and `LiftValue`. The `BindOk` and `BindErr` providers use them in their
[`Computer`](../components/computer.md) and `AsyncComputer` impls: `ContainsValue` says what the underlying value
beneath the wrapper is, so the step knows what to hand the continuation, and `LiftValue` puts a result back into
the output type — `lift_value` for the branch that short-circuits, `lift_output` for the branch that forwarded to
the continuation.

**Stacking** is `MonadicTrans` again, applied to the monads rather than to the handlers. Because the transformer
forms implement `ContainsValue` and `LiftValue` by delegating to the base monad, a two-layer stack unwraps two
`Result` layers in order, and an *n*-layer stack unwraps *n* — with no code specific to any depth.

Pipelines built this way implement the [`Computer`](../components/computer.md) family, so they slot into the same
wiring as any other handler. The whole fold happens during trait resolution; a monadic pipeline is not a runtime
structure.

## Gotchas

**None of the four is in the prelude.** Import from `cgp::extra::monad::traits`.

**`OkMonadic` short-circuits on `Ok`, not on `Err`.** The naming reads as "the monad *for* `Ok`" and means "the
monad whose continue branch is `Err`". `ErrMonadic` is the one that behaves like `?`. Getting these the wrong way
round produces a pipeline that runs exactly when you expected it to stop.

**These are not components and cannot be wired.** There is no `…Component` marker, so they never appear in a
[`delegate_components!`](../macros/delegate_components.md) block. What gets wired is the provider that consumes
them.

**A stack must be written in layer order.** `OkMonadicTrans<ErrMonadic>` and `ErrMonadicTrans<OkMonadic>` unwrap
their `Result` layers in opposite orders and are not interchangeable.

**`LiftValue` has two methods for a reason.** Implementing `lift_output` as `lift_value` — or forgetting the
distinction — collapses the two branches and silently changes what a pipeline does on a short-circuit.

**A new monad needs all four traits, plus the transformer form to stack.** Implementing three of them yields a
marker that works in some positions and fails to resolve in others, with an error naming the missing trait rather
than the gap.

## Related constructs

- [Monad providers](../providers/monad_providers.md) — `PipeMonadic`, `BindOk`, `BindErr`, and the monad markers;
  what you actually wire.
- [Handler combinators](../providers/handler_combinators.md) — `ComposeHandlers` and `PipeHandlers`, composition
  without a short-circuit branch.
- [`Computer`](../components/computer.md) — the component family a monadic pipeline implements.
- [`TryComputer`](../components/try_computer.md) and [`Handler`](../components/handler.md) — the fallible members
  a single step usually is.
- [`Product!`](../macros/product.md) — the type-level list a pipeline's steps are given in.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers) — why a pipeline short-circuits and how the monads compose.
- [Handlers](/docs/concepts/handlers) — the computation family these pipelines are built from.

## Source

- The traits: [`cgp-monad/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/traits)
  — `monadic_trans.rs`, `bind.rs`, `lift.rs`, `value.rs`
- The per-marker impls: [`cgp-monad/src/monadic/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-monad/src/monadic)
  — `ident.rs`, `ok.rs`, `err.rs`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
