---
title: 'CanDowncast — narrow an enum, or get the rest'
sidebar_label: 'CanDowncast'
sidebar_position: 2
description: 'Try to convert a wide enum into a narrower one, getting the narrowed value or a remainder with the attempted variants ruled out for the next attempt.'
---

# `CanDowncast`

Narrowing a wide enum into a smaller one, or handing back what is left.

## Overview

Converting a wide enum into a narrower one is the direction that can fail: the value's current variant
may be one the target does not have. `CanDowncast` attempts that conversion, and it does more than
a fallible `From` because of what it returns on failure.

**A failed downcast hands back a *remainder* rather than an error.** The remainder is the same value with
the attempted variants ruled out *in its type*, so it can be tried against another target, and each
attempt narrows what remains. That lets a value travel through a chain of candidate targets
while the compiler tracks, at every step, which variants are still possible.

The narrowing counterpart of [`CanUpcast`](./can_upcast.md), which always succeeds because a wider target
has a home for everything.

## Definition

`CanDowncast` is parameterized by the target enum, with a fallible method and a `Remainder` type:

```rust
pub trait CanDowncast<Target> {
    type Remainder;

    fn downcast(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder>;
}
```

`Self` is the wide enum and `Target` the narrower one. The method takes `self` by value and returns
`Result<Target, Remainder>`: `Ok` carries the narrowed value, and `Err` carries the source's extractor
with the target's variants removed. `Remainder` is that leftover extractor type, and it is the value you
pass to [`CanDowncastFields`](./can_downcast_fields.md) to continue a chain against another candidate. The
`PhantomData<Target>` argument names the target for inference.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::impls`, with
`core::marker::PhantomData` for naming the target:

```rust
use cgp::core::field::impls::CanDowncast;
use core::marker::PhantomData;
```

**The two sides swap roles from an upcast.** The source is only taken apart, so it needs
[`#[derive(ExtractField)]`](../../derives/derive_extract_field.md); the target's variant list is the
one walked and each match is rebuilt into it, so it needs
[`#[derive(HasFields)]`](../../derives/derive_has_fields.md) and
[`#[derive(FromVariant)]`](../../derives/derive_from_variant.md).


## Examples

A downcast succeeds for a variant the target shares and fails for one it lacks:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::CanDowncast;

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

pub fn demo() {
    assert_eq!(
        FooBarBaz::Bar("hi".to_owned())
            .downcast(PhantomData::<FooBar>)
            .ok(),
        Some(FooBar::Bar("hi".to_owned())),
    );

    assert_eq!(FooBarBaz::Baz(true).downcast(PhantomData::<FooBar>).ok(), None);
}
```

`.ok()` suits a single attempt and is wrong when another target should be tried, because it throws
away the remainder the next attempt needs.

## When to use it

**Reach for it when a value must be tried against a target the source may or may not fit**, and reach
for it *with* [`CanDowncastFields`](./can_downcast_fields.md) when there is more than one candidate: the
first starts the chain and the second continues it on each remainder.

- **Prefer a `match`** for a one-off narrowing of a concrete enum. It is shorter and already exhaustive.
  A downcast pays off when the target is a type parameter, or when the chain of candidates is long.
- **Prefer a plain `TryFrom` impl** when both types are yours and the conversion is one you would write
  once. Nothing about a hand-written impl needs either type to derive anything.
- **Reach for [`CanUpcast`](./can_upcast.md)** for the widening direction, which cannot fail.
- **Reach for the [extractor family](../variant/extract_field.md)** when you want to handle *every* variant rather
  than convert to another enum. A downcast chain and an extraction chain narrow the same way; the
  difference is whether each step produces another enum or a payload.

## Under the hood

**The downcasts recurse over the *target's* variants**, which is the mirror image of
[`CanUpcast`](./can_upcast.md#under-the-hood) walking the source's. The impl hands the source's
extractor to the target's variant list:

```rust
impl<Context, Source, Target, Remainder> CanDowncast<Target> for Context
where
    Context: HasExtractor<Extractor = Source>,
    Target: HasFields,
    Target::Fields: FieldsExtractor<Source, Target, Remainder = Remainder>,
{
    type Remainder = Remainder;

    fn downcast(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder> {
        Target::Fields::extract_from(self.to_extractor())
    }
}
```

`FieldsExtractor` is the same walk the upcast uses: for each `Field<Tag, Value>` in
`Target::Fields`, it tries pulling that variant out of the source extractor with
[`ExtractField`](../variant/extract_field.md). On success it rebuilds the target with
[`FromVariant`](../variant/from_variant.md) and returns `Ok`; on failure it threads the shrunken
remainder into the next attempt. If no target variant matches, the terminal `Void` impl returns the
whole remainder as `Err`. The walk needs `ExtractField` for every target variant, so each one must
exist in the source; the source's other variants are what can reach the `Err`.

`CanDowncast` differs from [`CanDowncastFields`](./can_downcast_fields.md) only in where the walk
starts: this one calls [`to_extractor`](../variant/has_extractor.md) on the enum first, while that
one operates on an extractor it is handed. That is the entire reason two traits exist rather than
one, and it makes chaining possible: the `Remainder` a `downcast` returns is precisely a
`downcast_fields` input.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::impls`.

**A downcast returns a remainder, not an `Option`.** `.ok()` discards it, which suits a single
attempt and is wrong if you meant to try another target.

**A remainder carries none of the enum's attributes**, so a `Result<Target, Remainder>` is neither
`Debug` nor `PartialEq`, and comparing one whole does not compile, as
[`ExtractField`](../variant/extract_field.md#common-mistakes) shows. Reach for `.ok()`, `.is_ok()`,
or a `match`.

**Every target variant must exist in the source, by name and payload type.** A source variant the
target lacks is the runtime `Err` case, but a target variant the source lacks is a compile error.
With a target `FooQux { Foo(u64), Qux(char) }` and a source `FooBar { Foo(u64), Bar(String) }`:

```rust
let _ = FooBar::Foo(1).downcast(PhantomData::<FooQux>);
```

the walk cannot extract `Qux` from what remains of the source:

```text
error[E0277]: the trait bound `__PartialFooBar<IsVoid, IsPresent>: ExtractField<Symbol<3, cgp::prelude::Chars<'Q', cgp::prelude::Chars<'u', cgp::prelude::Chars<'x', Nil>>>>>` is not satisfied
...
   = note: required for `FooBar` to implement `CanDowncast<FooQux>`
```

**The remainder's type is not the source enum.** It is a partial companion with markers, so it cannot be
stored where the original was.

## Related constructs

- [`CanDowncastFields`](./can_downcast_fields.md): the continuation, called on each remainder.
- [`CanUpcast`](./can_upcast.md): the widening direction, which cannot fail.
- [`CanBuildFrom`](./can_build_from.md): the record counterpart of casting.
- [`ExtractField`](../variant/extract_field.md): the primitive each attempt uses, and the family that narrows the
  same way to a payload.
- [`HasExtractor`](../variant/has_extractor.md): where the walk starts.
- [`FromVariant`](../variant/from_variant.md): how a matched variant is rebuilt into the target.
- [`#[derive(CgpVariant)]`](../../derives/derive_cgp_variant.md): what makes an enum eligible.
- [Type-level lists](../../types/index.md): the `Either`/`Void` chain underneath.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants): upcasting and downcasting between enums.

## Source

- [`cast.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/cast.rs):
  `CanDowncast` and the `FieldsExtractor` recursion

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
