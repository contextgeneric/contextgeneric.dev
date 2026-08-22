---
sidebar_label: 'CanDowncastFields'
---

# `CanDowncastFields`

Continuing a narrowing chain on a remainder.

## Overview

[`CanDowncast`](./can_downcast.md) narrows an enum and, when it fails, hands back a *remainder* — the
same value with the attempted variants ruled out in its type. `CanDowncastFields` is what that remainder
is then passed to, so a value can be tried against a second target, a third, and so on.

The two traits do the same narrowing and differ in one thing: **where the walk starts**. `CanDowncast`
is called on an enum and converts it to an extractor first; `CanDowncastFields` is called on an extractor
it is already given. Since a remainder *is* an extractor, that is exactly the shape a chain needs.

**So the pair is the construct, and this half is never used alone.** A chain reads as one `downcast`
followed by one `downcast_fields` per additional candidate.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::impls`, alongside `CanDowncast`, which
you will always be using with it:

```rust
use cgp::core::field::impls::{CanDowncast, CanDowncastFields};
use core::marker::PhantomData;
```

```rust
pub trait CanDowncastFields<Target> {
    type Remainder;

    fn downcast_fields(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder>;
}
```

The signature is `CanDowncast`'s exactly. `Self` here is a remainder rather than an enum, and the
`Remainder` it returns is narrower again — so each call in a chain has a different, progressively
smaller `Self` type.

## Examples

Trying a value against two candidate targets in turn:

```rust
use cgp::core::field::impls::{CanDowncast, CanDowncastFields};
use cgp::prelude::*;
use core::marker::PhantomData;

#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum FooBarBaz {
    Foo(u64),
    Bar(String),
    Baz(bool),
}

#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum JustFoo {
    Foo(u64),
}

#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum JustBar {
    Bar(String),
}

let value = FooBarBaz::Bar("hi".to_owned());

match value.downcast(PhantomData::<JustFoo>) {
    Ok(foo) => { /* it was a Foo */ }
    Err(remainder) => {
        // `remainder` can no longer be a Foo
        let bar = remainder.downcast_fields(PhantomData::<JustBar>);
        assert!(bar.is_ok());
    }
}
```

The first call starts the chain and the second continues it. Written the other way round — a second
`downcast` on the remainder — it does not compile, because a remainder is not an enum.

## When to reach for it, and when not

**Reach for it only as the continuation of a [`CanDowncast`](./can_downcast.md) chain.** It has no other
use: its `Self` is a remainder, and a remainder only ever comes from a prior downcast.

- **Use one `downcast` and then one `downcast_fields` per further candidate.** That is the whole
  pattern.
- **Prefer the [extractor family](./extract_field.md)** when what you want is the *payload* of whichever
  variant is present rather than another enum. It narrows identically and ends with a checked,
  wildcard-free discharge through [`FinalizeExtract`](./finalize_extract.md).
- **Prefer a `match`** when the enum is concrete and the candidates are few. A cast chain earns its keep
  when the targets are type parameters or the chain is long.

## Under the hood

:::note

### Advanced

This section is one paragraph, because the recursion is
[`CanDowncast`](./can_downcast.md#under-the-hood)'s.

:::

Both traits walk the *target's* variant list, trying to pull each one out of the extractor they hold and
threading the shrunken remainder into the next attempt, with the terminal `Void` impl returning the whole
remainder as `Err`. The only difference is the entry point: `CanDowncast` calls
[`to_extractor`](./has_extractor.md) on an enum to obtain the extractor, and this trait takes one it is
handed.

Because the remainder type narrows at every step, a chain's types are all distinct, and the compiler
knows at each point exactly which variants remain — the same narrowing a hand-written
[extraction chain](./extract_field.md) performs.

## Gotchas

**It is not in the prelude.** Import from `cgp::core::field::impls`.

**It cannot be called on an enum.** `Self` is an extractor. Start with
[`CanDowncast`](./can_downcast.md).

**`.ok()` on an intermediate step ends the chain.** Discarding the remainder discards the input the next
attempt needs.

**A remainder carries none of the enum's attributes**, so a `Result` holding one is neither `Debug` nor
`PartialEq`.

**Each step has a different `Self` type**, so a chain cannot be written as a loop or collected into a
`Vec` of attempts — it is unrolled by construction.

## Related constructs

- [`CanDowncast`](./can_downcast.md) — the first step, and where the pair is explained in full.
- [`CanUpcast`](./can_upcast.md) — the widening direction.
- [`ExtractField`](./extract_field.md) — narrowing to a payload rather than to another enum.
- [`FinalizeExtract`](./finalize_extract.md) — how an exhausted remainder is discharged.
- [`HasExtractor`](./has_extractor.md) — what turns an enum into the extractor this operates on.
- [`#[derive(CgpVariant)]`](../derives/derive_cgp_variant.md) — what makes an enum eligible.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — partial variants and the narrowing chain.

## Source

- [`cast.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/cast.rs) —
  `CanDowncastFields` and the `FieldsExtractor` recursion

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
