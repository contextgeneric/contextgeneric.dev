---
sidebar_label: 'Monadic handlers'
sidebar_position: 12
description: 'How monadic handlers put a pipeline continuation rule in a type, the err, ok, and identity monads, and how a monad transformer stacks two result layers.'
---

# Monadic handlers

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations, and
monadic handlers make the rule that joins them a choice too. Monadic handlers separate a pipeline's
computations from the rule that decides whether another step should run. The same composition
machinery can stop at the first error, stop at the first success, or pass every output onward. A
monad supplies that continuation rule, while the providers supply the work performed at each step.

Ordinary [handler composition](./handlers.md) already connects outputs to inputs. Monadic
composition extends that connection to cases where the next step needs a value inside a result,
or where one branch should end the pipeline.

## When direct composition does not fit

When computers compose directly, one step passes its whole output to the next. That works only if
the next computer accepts that type. A fallible increment makes the difficulty visible; this
fragment assumes imports from `cgp::prelude::*`:

```rust
#[cgp_computer]
pub fn increment(value: u8) -> Result<u8, &'static str> {
    value.checked_add(1).ok_or("overflow")
}
```

The generated `Increment` computer takes a `u8` but returns `Result<u8, &'static str>`. If a second
copy receives that whole result, its input type does not fit. To connect the two, the pipeline needs
to unpack an `Ok` and continue with its `u8`, or return an `Err` before running the next increment.

Composition through `TryComputer` or `Handler` already propagates the interface's outer error with
`?`, so this mismatch is specific to treating the result as a `Computer`'s output value. Monadic
composition lets the pipeline choose that continuation behavior separately. It can reproduce error
propagation here or choose a different rule, such as stopping when an attempt succeeds.

## Choosing the continuation rule

The continuation rule belongs to a **monad**, which describes how computations compose through a
surrounding type. For CGP's result-based monads, this includes which branch supplies the next input,
which branch stops the pipeline, and how a plain value enters the result type. The identity monad
uses the same idea for ordinary composition, with no result wrapper.

`PipeMonadic` combines that choice with a type-level list of providers. This type fragment uses
`PipeMonadic` from `cgp::extra::monad::providers` and `ErrMonadic` from
`cgp::extra::monad::monadic::err`:

```rust
PipeMonadic<ErrMonadic, Product![Increment, Increment, Increment]>
```

With `ErrMonadic`, each `Ok` supplies the next `Increment` with its contained value, so starting
from `1` produces `Ok(4)`. Starting from `253` reaches `254`, then `255`, before the last increment
returns `Err("overflow")`. An input of `255` fails immediately and skips the remaining steps.
The provider types therefore assemble the same control flow as successive `?` expressions.

## The built-in monads

Choose the marker according to which result should end the pipeline:

| Marker | Input passed to the next step | Result that ends the pipeline |
| --- | --- | --- |
| `ErrMonadic` | The value inside `Ok` | `Err` |
| `OkMonadic` | The value inside `Err` | `Ok` |
| `IdentMonadic` | The complete output | Every step continues |

For a sequence of attempts, `OkMonadic` reverses the stopping rule: the first success ends the
search. A failed attempt returns the next step's input inside `Err`. Variant dispatch uses this to
pass an unmatched remainder from one attempt to another until a handler accepts it.

If every step should continue, `IdentMonadic` forwards the complete output without unpacking it.
This keeps the same composition machinery, but requires the next provider to accept that exact
output type. It therefore leaves the `Increment` input mismatch unresolved.

When a computation has nested results, monad transformers let both layers participate in the
continuation decision. For `Result<Result<T, E>, F>`, `OkMonadicTrans<ErrMonadic>` stops on an outer
`Err(F)` or an inner `Ok(T)`. Only `Ok(Err(E))` continues, supplying `E` to the next step. The order
of the nested rules matters because it determines how each result layer is interpreted.

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

Once wired, the pipeline is available through the ordinary `CanCompute` interface. A caller
supplies the `()` code tag and a `u8`, then receives `Result<u8, &'static str>` without needing to
name a monad. The context's table determines how those increments are composed.

Rust selects the providers from their types at compile time, so the marker and list need no
runtime instances or lookup table. The increments and branches still execute when the pipeline
runs. The same `PipeMonadic` machinery supports async and fallible handler forms, retaining their
execution and error behavior.

## What it costs

A fixed sequence of fallible operations is usually clearer as a function using `?`. Monadic
composition is useful when contexts select different sequences or when reusable steps need a
continuation rule such as stopping at the first success.

Moving the continuation rule into types can make an incompatible stage harder to identify.
The error may involve associated outputs and monad traits without pointing directly to the stage
that does not fit. Checking a short pipeline before extending it narrows the possible source of a
mismatch.

With nested monads, understanding a call also means following the decision at each result layer.
A transformer is useful when those layers represent distinct choices, but its provider type may
not explain them to a reader. Documenting exactly which cases continue makes that control flow
easier to follow.

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
