---
sidebar_label: 'PromoteRef'
sidebar_position: 6
---

# `PromoteRef`

Bridge between value handlers and reference handlers by dereferencing.

:::info

### Generated machinery

**You are not expected to name `PromoteRef` directly.** The promotion bundles such as
[`PromoteComputer`](promote_computer.md) wire it into every `…Ref` slot, and the bundles are what
[`#[cgp_computer]`](../../macros/cgp_computer.md) and [`#[cgp_producer]`](../../macros/cgp_producer.md)
generate. You reach for it by hand only when wiring the handler family one slot at a time. This page
explains what that wiring emits.

:::

## Overview

`PromoteRef<Provider>` lets a provider written to take a value serve a slot that takes a reference, and
the reverse, without manual dereference code. It is the most thoroughly implemented promotion: it
covers all four handler families in both directions, on a **context**, the type a capability runs
against. Like every CGP provider, it carries no runtime value; the inner provider rides in
`PhantomData`.

## Usage

Import it from `cgp::extra::handler`. It takes one type parameter, the inner provider. To fill a
by-reference slot such as `ComputerRefComponent`, the inner provider is a by-value provider whose input
is a reference (`Computer<Context, Code, &Input>`), and `PromoteRef` calls it on the borrowed input:

```rust
use cgp::extra::handler::PromoteRef;

// `DoubleRef` is a `Computer<_, _, &u64>`; PromoteRef serves the by-reference slot from it.
delegate_components! {
    App {
        ComputerRefComponent: PromoteRef<DoubleRef>,
    }
}
```

## When to reach for it, and when not

**You rarely reach for `PromoteRef` by hand.** Every promotion bundle wires it into the `…Ref` slots, so
the by-reference family follows from a by-value provider automatically. Reach for it directly only when
wiring a `…Ref` slot by hand. For the other axes use [`Promote`](promote.md),
[`PromoteAsync`](promote_async.md), or [`TryPromote`](try_promote.md).

## Under the hood

`PromoteRef<Provider>` carries the inner provider in `PhantomData`:

```rust
pub struct PromoteRef<Provider>(pub PhantomData<Provider>);
```

For each of `Computer`/`ComputerRef`, `TryComputer`/`TryComputerRef`, `AsyncComputer`/`AsyncComputerRef`,
and `Handler`/`HandlerRef`, it provides two impls. One direction implements the by-value trait given an
inner by-reference provider plus `Input: Deref<Target = Target>`, calling the inner provider on
`input.deref()`. The other direction implements the by-reference trait given an inner by-value provider
that works `for<'a>` over `&'a Input`, calling the inner provider on the borrowed input. A provider
written to take `&T` can then serve a slot that hands it a smart pointer to `T`, and the reverse.

## Related constructs

- [`Promote`](promote.md), [`PromoteAsync`](promote_async.md), [`TryPromote`](try_promote.md) — the
  other single-step lifts, along the fallibility and asynchrony axes rather than the borrow axis.
- [`PromoteComputer`](promote_computer.md) and the other bundles — wire `PromoteRef` into every `…Ref`
  slot automatically.
- [`Computer`](../../components/handler/computer.md), [`Handler`](../../components/handler/handler.md) — the family
  members and their by-reference companions it bridges.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family, including the by-reference companion of each member.

## Source

- [`providers/promote_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_ref.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
