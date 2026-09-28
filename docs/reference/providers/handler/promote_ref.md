---
title: 'PromoteRef — bridge owned and borrowed inputs'
description: 'The single-step lift between a handler family member and its …Ref companion: deref an owned input for a …Ref provider, or pass a borrow as the input.'
sidebar_label: 'PromoteRef'
sidebar_position: 6
---

# `PromoteRef`

Bridge between by-value handlers and by-reference handlers, in both directions.

## Overview

`PromoteRef<Provider>` lets a provider written for one input mode serve a slot that uses the other,
without manual dereference code, on a [**context**](/docs/reference/glossary#context), the type the
implementation runs against. It is the most thoroughly implemented promotion: it covers all four
handler families, `Computer`, `TryComputer`, `AsyncComputer`, and `Handler`, in both directions, and
the two directions ask different things of the inner provider. Every promotion bundle routes its
`…Ref` slots through it, and it is written by hand when a context wires one slot at a time. Like
every CGP provider, it carries no runtime value; the inner provider rides in `PhantomData`.

## Usage

Import it from `cgp::extra::handler`; it is not in the prelude. It takes one type parameter, the
inner provider, and the direction depends on the slot it fills:

- **A by-reference slot**, such as `ComputerRefComponent`, needs an inner by-value provider whose
  input is a reference, `Computer<Context, Code, &'a Input>` at every lifetime `'a`. `PromoteRef`
  passes the borrow straight through as that input. A provider written for an owned `u64` does not
  qualify.
- **A by-value slot**, such as `ComputerComponent`, needs an inner by-reference provider and an
  input that dereferences to its `Input`, such as a `Box<String>` for a `ComputerRef` over `String`.
  `PromoteRef` calls the inner provider on `input.deref()`.

```rust
use cgp::extra::handler::PromoteRef;

delegate_components! {
    App {
        ComputerRefComponent: PromoteRef<DoubleRef>,
        ComputerComponent: PromoteRef<StringLength>,
    }
}
```

## Examples

Both directions on one context, each from a hand-written provider:

```rust
use cgp::prelude::*;
use cgp::extra::handler::{CanCompute, CanComputeRef, ComputerRef, PromoteRef};

/// A by-value computer whose input is a reference.
#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, &u64> for DoubleRef {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: &u64) -> u64 {
        input * 2
    }
}

/// A by-reference computer over a `String`.
#[cgp_new_provider]
impl<Context, Code> ComputerRef<Context, Code, String> for StringLength {
    type Output = usize;

    fn compute_ref(_context: &Context, _code: PhantomData<Code>, input: &String) -> usize {
        input.len()
    }
}

pub struct App;

delegate_components! {
    App {
        ComputerRefComponent: PromoteRef<DoubleRef>,
        ComputerComponent: PromoteRef<StringLength>,
    }
}

check_components! {
    App {
        ComputerRefComponent: ((), u64),
        ComputerComponent: ((), Box<String>),
    }
}

pub fn demo() {
    let code = PhantomData::<()>;

    assert_eq!(App.compute_ref(code, &21), 42);
    assert_eq!(App.compute(code, Box::new("hello".to_owned())), 5);
}
```

`compute_ref` hands `DoubleRef` the borrowed `&21` as its input, and `compute` dereferences the
`Box<String>` for `StringLength`. `App` is an
[environmental context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `PromoteRef` when wiring a slot by hand across the owned and borrowed forms.** Every
promotion bundle already routes its `…Ref` slots here, so a macro-generated provider needs no
`PromoteRef` of its own; it serves the `…Ref` slots only when its function takes a reference
parameter, since an owned-input provider cannot accept the borrow. For the other axes use
[`Promote`](promote.md), [`PromoteAsync`](promote_async.md), or [`TryPromote`](try_promote.md).

## Under the hood

`PromoteRef<Provider>` carries the inner provider in `PhantomData`:

```rust
pub struct PromoteRef<Provider>(pub PhantomData<Provider>);
```

For each of `Computer`/`ComputerRef`, `TryComputer`/`TryComputerRef`,
`AsyncComputer`/`AsyncComputerRef`, and `Handler`/`HandlerRef`, it provides two impls. The by-value
impl requires an inner by-reference provider plus `Input: Deref<Target = Target>`, and calls the
inner provider on `input.deref()`. The by-reference impl requires an inner by-value provider that
works `for<'a>` over `&'a Input`, and calls it on the borrowed input:

```rust
#[cgp_provider]
impl<Context, Code, Input, Provider, Output> ComputerRef<Context, Code, Input>
    for PromoteRef<Provider>
where
    Provider: for<'a> Computer<Context, Code, &'a Input, Output = Output>,
{
    type Output = Output;

    fn compute_ref(context: &Context, tag: PhantomData<Code>, input: &Input) -> Self::Output {
        Provider::compute(context, tag, input)
    }
}
```

The fallible impls require the context to have an error type.

## Related constructs

- [`Promote`](promote.md), [`PromoteAsync`](promote_async.md), [`TryPromote`](try_promote.md) — the
  other single-step lifts, along the fallibility and asynchrony axes rather than the borrow axis.
- [`PromoteComputer`](promote_computer.md) and the other bundles — route every `…Ref` slot through
  `PromoteRef`.
- [`ComputerRef`](../../components/handler/computer_ref.md),
  [`HandlerRef`](../../components/handler/handler_ref.md) — the by-reference companions it bridges.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the family, including the by-reference companion of each
  member.

## Source

- [`providers/promote_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_ref.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
