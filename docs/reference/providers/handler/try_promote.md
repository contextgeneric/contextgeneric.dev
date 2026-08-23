---
sidebar_label: 'TryPromote'
sidebar_position: 7
---

# `TryPromote`

Bridge in both directions between a `Result`-valued output and a fallible handler trait.

:::info

### Generated machinery

**You are not expected to name `TryPromote` directly.** The promotion bundle
[`PromoteTryComputer`](promote_try_computer.md) wires it, and [`PipeMonadic`](../monad/pipe_monadic.md)
uses it internally to bridge fallible and infallible handlers. You reach for it by hand only when
wiring the handler family one slot at a time. This page explains what that wiring does.

:::

## Overview

`TryPromote<Provider>` unifies the two ways of expressing fallibility: a handler that *returns* a
`Result`, and a genuinely fallible handler trait. It converts between them in both directions on a
**context**, the type a capability runs against. Every impl requires the context to have an error type.
Like every CGP provider, it carries no runtime value; the inner provider rides in `PhantomData`.

## Usage

Import it from `cgp::extra::handler`. It takes one type parameter, the inner provider:

```rust
use cgp::extra::handler::TryPromote;

// Turn a Computer that returns Result into a genuine TryComputer.
delegate_components! {
    App {
        TryComputerComponent: TryPromote<CheckedAdd>,
    }
}
```

## When to reach for it, and when not

**You rarely reach for `TryPromote` by hand.** The [`PromoteTryComputer`](promote_try_computer.md) and
[`PromoteHandler`](promote_handler.md) bundles wire it, and [`PipeMonadic`](../monad/pipe_monadic.md)
uses it internally. Reach for it directly only to convert between a `Result`-valued output and a
fallible trait when wiring a slot by hand. For the other lifts use [`Promote`](promote.md),
[`PromoteAsync`](promote_async.md), or [`PromoteRef`](promote_ref.md).

## Under the hood

`TryPromote<Provider>` carries the inner provider in `PhantomData`:

```rust
pub struct TryPromote<Provider>(pub PhantomData<Provider>);
```

- As a `TryComputer`, it requires the inner `Provider: Computer` whose `Output` is itself a
  `Result<Output, Context::Error>`, and unwraps that into the `TryComputer` result. This turns a
  computer that *returns* a `Result` into a genuine fallible computer.
- As a `Computer`, it goes the other way: given `Provider: TryComputer`, its output type is
  `Result<Output, Context::Error>`, surfacing the fallible result as an ordinary value.

The analogous pair lifts between `Handler` (from an `AsyncComputer` returning a `Result`) and
`AsyncComputer` (from a `Handler`). All four impls require the context to have an error type.

## Related constructs

- [`Promote`](promote.md), [`PromoteAsync`](promote_async.md), [`PromoteRef`](promote_ref.md) — the
  other single-step lifts.
- [`PromoteTryComputer`](promote_try_computer.md) — the bundle that wires `TryPromote`.
- [`PipeMonadic`](../monad/pipe_monadic.md) — uses `TryPromote` to demote fallible handlers when
  composing a fallible monadic pipeline.
- [`TryComputer`](../../components/try_computer.md), [`Handler`](../../components/handler.md) — the
  fallible family members it bridges.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family and its fallibility axis.

## Source

- [`providers/try_promote.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/try_promote.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
