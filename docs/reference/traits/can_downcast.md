---
sidebar_label: 'CanDowncast'
---

# `CanDowncast`

Narrowing a wide enum into a smaller one, or handing back what is left.

## What it's for

Converting a wide enum into a narrower one is the direction that can fail: the value's current variant
may be one the target does not have. `CanDowncast` attempts that conversion, and what makes it more than
a fallible `From` is what it returns on failure.

**A failed downcast hands back a *remainder* rather than an error.** The remainder is the same value with
the attempted variants ruled out *in its type*, so it can be tried against another target — and each
attempt narrows what remains. That is what lets a value be routed through a chain of candidate targets
with the compiler tracking, at every step, which variants are still possible.

The narrowing counterpart of [`CanUpcast`](./can_upcast.md), which always succeeds because a wider target
has a home for everything.

## Using it

**It is not in the prelude.** Import it from `cgp::core::field::impls`, with
`core::marker::PhantomData` for naming the target:

```rust
use cgp::core::field::impls::CanDowncast;
use core::marker::PhantomData;
```

```rust
pub trait CanDowncast<Target> {
    type Remainder;

    fn downcast(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder>;
}
```

`Ok` carries the narrowed value. `Err` carries the source's extractor with the target's variants removed
— which is exactly what [`CanDowncastFields`](./can_downcast_fields.md) is called on to continue the
chain. `Self` is the wide enum, consumed.

## Examples

A downcast succeeds for a variant the target shares and fails for one it lacks:

```rust
use cgp::core::field::impls::CanDowncast;
use cgp::prelude::*;
use core::marker::PhantomData;

#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum FooBar {
    Foo(u64),
    Bar(String),
}

#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum FooBarBaz {
    Foo(u64),
    Bar(String),
    Baz(bool),
}

assert_eq!(
    FooBarBaz::Bar("hi".to_owned()).downcast(PhantomData::<FooBar>).ok(),
    Some(FooBar::Bar("hi".to_owned())),
);

assert_eq!(FooBarBaz::Baz(true).downcast(PhantomData::<FooBar>).ok(), None);
```

`.ok()` is right for a single attempt and wrong when another target should be tried, because it throws
away the remainder that the next attempt needs.

## When to reach for it, and when not

**Reach for it when a value must be tried against a target the source may or may not fit**, and reach
for it *with* [`CanDowncastFields`](./can_downcast_fields.md) when there is more than one candidate: the
first starts the chain and the second continues it on each remainder.

- **Prefer a `match`** for a one-off narrowing of a concrete enum. It is shorter and already exhaustive.
  A downcast pays off when the target is a type parameter, or when the chain of candidates is long.
- **Prefer a plain `TryFrom` impl** when both types are yours and the conversion is one you would write
  once. Nothing about a hand-written impl needs either type to derive anything.
- **Reach for [`CanUpcast`](./can_upcast.md)** for the widening direction, which cannot fail.
- **Reach for the [extractor family](./extract_field.md)** when you want to handle *every* variant rather
  than convert to another enum. A downcast chain and an extraction chain narrow the same way; the
  difference is whether each step produces another enum or a payload.

## Under the hood

:::note

### Advanced

This section shows the recursion, and why the remainder is the shape it is.

:::

**The downcasts recurse over the *target's* variants**, which is the mirror image of
[`CanUpcast`](./can_upcast.md#under-the-hood) walking the source's. For each `Field<Tag, Value>` in
`Target::Fields`, the implementation tries pulling that variant out of the source extractor: on success
it rebuilds the target with [`FromVariant`](./from_variant.md) and returns `Ok`; on failure it threads the
shrunken remainder into the next attempt. If no target variant matches, the terminal `Void` impl returns
the whole remainder as `Err`.

`CanDowncast` differs from [`CanDowncastFields`](./can_downcast_fields.md) only in where the walk starts:
this one calls [`to_extractor`](./has_extractor.md) on the enum first, while that one operates on an
extractor it is handed. That is the entire reason two traits exist rather than one, and it is what makes
chaining possible — the `Remainder` a `downcast` returns is precisely a `downcast_fields` input.

## Gotchas

**It is not in the prelude.** Import from `cgp::core::field::impls`.

**A downcast returns a remainder, not an `Option`.** `.ok()` discards it, which is fine for a single
attempt and wrong if you meant to try another target.

**A remainder carries none of the enum's attributes**, so a `Result<Target, Remainder>` is neither
`Debug` nor `PartialEq` — comparing one whole does not compile. Reach for `.ok()`, `.is_ok()`, or a
`match`.

**Names and payload types must match exactly**, and a mismatch is a compile error rather than a runtime
miss: the variant simply drops out of the overlap.

**The remainder's type is not the source enum.** It is a partial companion with markers, so it cannot be
stored where the original was.

## Related constructs

- [`CanDowncastFields`](./can_downcast_fields.md) — the continuation, called on each remainder.
- [`CanUpcast`](./can_upcast.md) — the widening direction, which cannot fail.
- [`CanBuildFrom`](./can_build_from.md) — the record counterpart of casting.
- [`ExtractField`](./extract_field.md) — the primitive each attempt uses, and the family that narrows the
  same way to a payload.
- [`HasExtractor`](./has_extractor.md) — where the walk starts.
- [`FromVariant`](./from_variant.md) — how a matched variant is rebuilt into the target.
- [`#[derive(CgpVariant)]`](../derives/derive_cgp_variant.md) — what makes an enum eligible.
- [Type-level spines](../types/type_level_spines.md) — the `Either`/`Void` chain underneath.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — upcasting and downcasting between enums.

## Source

- [`cast.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/cast.rs) —
  `CanDowncast` and the `FieldsExtractor` recursion

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
