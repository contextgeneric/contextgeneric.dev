---
sidebar_label: 'Monadic handlers'
sidebar_position: 12
---

# Monadic handlers

Monadic handlers let a pipeline choose whether to continue based on each step's result. A pipeline
can stop at the first error, stop at the first success, or pass every output onward. This page
builds on [handlers](./handlers.md) to explain those choices, how CGP composes them, and when an
ordinary function is simpler.

## When direct composition does not fit

Direct composition through `Computer` passes each step's entire output to the next step.
That does not fit a chain whose steps return `Result<T, E>` but accept only `T`. Consider this
computer, defined with imports from `cgp::prelude::*`:

```rust
#[cgp_computer]
pub fn increment(value: u8) -> Result<u8, &'static str> {
    value.checked_add(1).ok_or("overflow")
}
```

`Increment` accepts a `u8` and produces a `Result<u8, &'static str>`. Directly composing two copies
through `Computer` fails because the second expects a `u8`, not the complete `Result`. The chain
needs a rule that passes the value inside `Ok` onward and returns an `Err` immediately.

The handler interface matters here. Ordinary composition through `TryComputer` or `Handler`
already propagates its outer error with `?`. Monadic composition makes the continuation rule a
separate choice, including when `Result` is the output of a `Computer` or when success should end
the chain.

## Choosing the continuation rule

A **monad** describes how to compose computations through a surrounding type. In CGP's result-based
monads, it determines which branch supplies the next input, which branch ends the pipeline, and how
a value is placed into the result type. The identity monad supplies ordinary composition without a
result wrapper.

`PipeMonadic` combines that choice with a type-level list of providers. This type fragment uses
`PipeMonadic` from `cgp::extra::monad::providers` and `ErrMonadic` from
`cgp::extra::monad::monadic::err`:

```rust
PipeMonadic<ErrMonadic, Product![Increment, Increment, Increment]>
```

`ErrMonadic` passes each `Ok` value to the next `Increment` and stops on `Err`. Starting from `1`
produces `Ok(4)`. Starting from `253` reaches `254`, then `255`, then returns `Err("overflow")`.
Starting from `255` fails in the first step and skips the other two. This is the control flow of
successive `?` expressions, assembled from provider types.

## The built-in monads

Choose the marker according to which result should end the pipeline:

| Marker | Input passed to the next step | Result that ends the pipeline |
| --- | --- | --- |
| `ErrMonadic` | The value inside `Ok` | `Err` |
| `OkMonadic` | The value inside `Err` | `Ok` |
| `IdentMonadic` | The complete output | Every step continues |

`OkMonadic` supports a sequence of attempts where the first success wins. Each failed attempt must
return, in `Err`, the input the next attempt needs. For example, variant dispatch can pass the
unmatched remainder onward until one handler accepts it.

`IdentMonadic` lets a pipeline use the same composition interface without short-circuiting.
It forwards the whole output, so the next provider must accept that exact type. Using it with
`Increment` does not resolve the `Result`-versus-`u8` mismatch.

Monad transformers combine continuation rules for nested results. For
`Result<Result<T, E>, F>`, `OkMonadicTrans<ErrMonadic>` stops on an outer `Err(F)` or an inner
`Ok(T)` and continues with the `E` inside `Ok(Err(E))`. Both result layers affect whether another
step runs; their nesting order determines how the output is interpreted.

## A pipeline is an ordinary provider

The composed pipeline fits into the same wiring as any other handler provider. This fragment
assumes an `App` context and imports `ComputerComponent` from `cgp::extra::handler`:

```rust
delegate_components! {
    App {
        ComputerComponent: PipeMonadic<ErrMonadic, Product![Increment, Increment]>,
    }
}
```

Callers use the ordinary `CanCompute` interface. For this example, the code tag is `()` and the
input is `u8`; the output is `Result<u8, &'static str>`. The context's wiring selects the pipeline,
and its callers do not need a separate monadic interface.

Provider selection happens at compile time, while the increments and result branches execute when
the pipeline runs. The marker and provider list require neither runtime instances nor a runtime
lookup table. `PipeMonadic` also supports the async and fallible handler forms; those forms retain
their own execution and error behavior.

## What it costs

A fixed sequence of fallible operations is usually clearer as a function using `?`. Monadic
composition is useful when contexts select different sequences or when reusable steps need a
continuation rule such as stopping at the first success.

Pipeline errors can be difficult to locate. An incompatible stage can produce a trait-resolution
error involving associated output types and monad traits rather than identifying the stage directly.
Checking a short pipeline before adding more stages narrows the source of a mismatch.

Nested monads require readers to track each result layer separately. Use a transformer when those
layers express distinct decisions, and document which cases continue. The provider type alone may
not make that behavior obvious to a reader unfamiliar with the chosen transformer.

## Where to go next

These pages cover the computations, applications, and traits behind the pipeline:

- [Handlers](./handlers.md): The computation interfaces and ordinary composition.
- [Dispatching](./dispatching.md): Variant matching as a sequence of attempts.
- [Monad providers](/docs/reference/providers/monad): `PipeMonadic`, marker and transformer types,
  and the per-step `BindOk` and `BindErr` providers.
- [`MonadicBind`](/docs/reference/traits/monad/monadic_bind),
  [`ContainsValue`](/docs/reference/traits/monad/contains_value),
  [`LiftValue`](/docs/reference/traits/monad/lift_value), and
  [`MonadicTrans`](/docs/reference/traits/monad/monadic_trans): The composition and lifting interfaces.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
