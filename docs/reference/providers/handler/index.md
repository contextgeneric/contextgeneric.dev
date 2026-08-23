---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Handler combinators

The providers that build, sequence, and adapt handlers: composing them end to end, threading a list
through a pipeline, returning the input unchanged, and lifting one handler shape into another.

## Overview

The handler combinators exist because the handler family is not one trait but several related ones, and
code is rarely written against all of them at once. A provider author writes a plain synchronous
[`Computer`](../../components/handler/computer.md), a fallible [`TryComputer`](../../components/handler/try_computer.md),
or an async [`Handler`](../../components/handler/handler.md), depending on the computation. The combinators let
those single-shape providers be wired where a different shape is expected, and let several providers be
glued into a larger one, on a **context**, the type a capability runs against. Like every CGP provider,
each combinator is zero-sized: its type parameters are inner providers carried in `PhantomData`.

## The handler family

Every combinator is defined in terms of the handler component traits, so a short orientation helps. The
family shares one method signature: a context reference, a `PhantomData<Code>` tag selecting the
operation, and an input, producing an associated `Output`. The members differ on two axes.

- [`Computer`](../../components/handler/computer.md) is synchronous and infallible.
- [`TryComputer`](../../components/handler/try_computer.md) is synchronous and fallible, and requires the context
  to have an error type.
- `AsyncComputer` is asynchronous and infallible.
- [`Handler`](../../components/handler/handler.md) is asynchronous and fallible, the most general member.
- [`Producer`](../../components/handler/producer.md) takes no input.

Each of the first four has a `…Ref` companion whose method takes the input by reference. The promotion
combinators trade on the natural orderings among these: a `Computer` is also a valid `TryComputer` and a
valid `AsyncComputer`, a `TryComputer` is a valid `Handler`, and a value handler can serve a reference
handler by dereferencing.

## The combinators, by role

**Composition** feeds one handler's output into the next:

- [`ComposeHandlers`](compose_handlers.md) runs two handlers back to back.
- [`PipeHandlers`](pipe_handlers.md) generalizes that to a type-level list of handlers.

**Identity** is the neutral element of composition:

- [`ReturnInput`](return_input.md) passes its input straight through.

**Promotion** lifts one handler shape into another, so one written implementation satisfies several
traits:

- [`Promote`](promote.md) lifts along the infallible-to-fallible and sync-to-async axes.
- [`PromoteAsync`](promote_async.md) lifts a synchronous provider into an asynchronous one.
- [`PromoteRef`](promote_ref.md) bridges value handlers and reference handlers by dereferencing.
- [`TryPromote`](try_promote.md) bridges a `Result`-valued output and a fallible trait.

**Promotion bundles** wire a whole cluster of handler components to the right single-step promotion at
once, so a provider author implements one trait and gets the rest of the family. These are what
[`#[cgp_computer]`](../../macros/cgp_computer.md) and [`#[cgp_producer]`](../../macros/cgp_producer.md)
wire automatically:

- [`PromoteComputer`](promote_computer.md), [`PromoteTryComputer`](promote_try_computer.md),
  [`PromoteProducer`](promote_producer.md), [`PromoteAsyncComputer`](promote_async_computer.md), and
  [`PromoteHandler`](promote_handler.md), one per base trait.

**Input dispatch** chooses a handler by the type of the value:

- [`UseInputDelegate`](use_input_delegate.md) keys a lookup table on the handler's `Input` type.

## Related constructs

- [`Computer`](../../components/handler/computer.md), [`TryComputer`](../../components/handler/try_computer.md),
  [`Handler`](../../components/handler/handler.md), [`Producer`](../../components/handler/producer.md) — the components
  these providers implement.
- [`#[cgp_computer]`](../../macros/cgp_computer.md) and [`#[cgp_producer]`](../../macros/cgp_producer.md)
  — generate a single-trait provider and wire the rest of the family through the promotion bundles.
- [`Product!`](../../macros/product.md) — the type-level list `PipeHandlers` composes.
- [`UseDelegate`](../use_delegate.md) — the `Code`-keyed sibling of `UseInputDelegate`.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.

## Source

- The combinators are in `cgp-handler` under
  [`providers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-handler/src/providers),
  and `UseInputDelegate` in
  [`types.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/types.rs).

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
