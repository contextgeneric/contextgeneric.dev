---
title: 'CanDowncastFields — continue a narrowing'
sidebar_label: 'CanDowncastFields'
sidebar_position: 3
description: 'Continue narrowing on the remainder a failed downcast hands back, so a value can be tried against a second target and a third. Never used on its own.'
---

# `CanDowncastFields`

Continuing a narrowing chain on a remainder.

## Overview

[`CanDowncast`](./can_downcast.md) narrows an enum and, when it fails, hands back a *remainder*: the
same value with the attempted variants ruled out in its type. You then pass that remainder to
`CanDowncastFields`, so a value can be tried against a second target, a third, and so on.

The two traits do the same narrowing and differ in one thing: **where the walk starts**. `CanDowncast`
is called on an enum and converts it to an extractor first; `CanDowncastFields` is called on an extractor
it is already given. Since a remainder *is* an extractor, that is exactly the shape a chain needs.

**So the pair is the construct, and this half is almost always the continuation.** A chain reads as
one `downcast` followed by one `downcast_fields` per additional candidate. It applies to any
extractor that can yield each target variant, so it also works on a fresh `to_extractor()`, where it
does exactly what `downcast` does.

## Definition

`CanDowncastFields` has the same signature as [`CanDowncast`](./can_downcast.md):

```rust
pub trait CanDowncastFields<Target> {
    type Remainder;

    fn downcast_fields(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder>;
}
```

`Target` is the enum being narrowed to. `Self` here is a *remainder* rather than an enum, and the method
takes it by value, returning `Result<Target, Remainder>` as `downcast` does. The returned
`Remainder` is narrower again, so each call in a chain has a different, progressively smaller `Self` type.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::impls`, alongside `CanDowncast`, which
you will always be using with it:

```rust
use cgp::core::field::impls::{CanDowncast, CanDowncastFields};
use core::marker::PhantomData;
```

## Examples

Trying a value against three candidate targets in turn:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::{CanDowncast, CanDowncastFields};
use cgp::core::field::traits::FinalizeExtractResult;

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

#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum JustBaz {
    Baz(bool),
}

pub fn classify(value: FooBarBaz) -> &'static str {
    match value.downcast(PhantomData::<JustFoo>) {
        Ok(_) => "foo",
        // `remainder` can no longer be a `Foo`.
        Err(remainder) => match remainder.downcast_fields(PhantomData::<JustBar>) {
            Ok(_) => "bar",
            Err(remainder) => {
                // Every variant is now covered, so the last attempt cannot fail.
                let JustBaz::Baz(_) = remainder
                    .downcast_fields(PhantomData::<JustBaz>)
                    .finalize_extract_result();
                "baz"
            }
        },
    }
}

pub fn demo() {
    assert_eq!(classify(FooBarBaz::Foo(1)), "foo");
    assert_eq!(classify(FooBarBaz::Bar("hi".to_owned())), "bar");
    assert_eq!(classify(FooBarBaz::Baz(true)), "baz");
}
```

The first call starts the chain and each later one continues it on the remainder. The three
candidates cover every variant of `FooBarBaz`, so the last remainder is uninhabited and the final
step closes with [`finalize_extract_result`](../variant/finalize_extract_result.md) instead of a
`match`, exactly as an [extraction chain](../variant/extract_field.md) does.

## When to use it

**Reach for it as the continuation of a [`CanDowncast`](./can_downcast.md) chain.** Its `Self` is an
extractor, and in practice that is the remainder a prior downcast returned.

- **Use one `downcast` and then one `downcast_fields` per further candidate.** That is the whole
  pattern.
- **Prefer the [extractor family](../variant/extract_field.md)** when what you want is the *payload* of whichever
  variant is present rather than another enum. It narrows identically and ends with a checked,
  wildcard-free discharge through [`FinalizeExtract`](../variant/finalize_extract.md).
- **Prefer a `match`** when the enum is concrete and the candidates are few. A cast chain pays off
  when the targets are type parameters or the chain is long.

## Under the hood

Its one impl applies to any `Source` the target's variant walk can extract from, and hands it
straight to that walk:

```rust
impl<Source, Target, Remainder> CanDowncastFields<Target> for Source
where
    Target: HasFields,
    Target::Fields: FieldsExtractor<Source, Target, Remainder = Remainder>,
{
    type Remainder = Remainder;

    fn downcast_fields(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder> {
        Target::Fields::extract_from(self)
    }
}
```

`FieldsExtractor` is the same recursion [`CanDowncast`](./can_downcast.md#under-the-hood) uses: it
tries each target variant against the extractor, threading the shrunken remainder into the next
attempt, with the terminal `Void` impl returning the whole remainder as `Err`. The only difference
is the entry point: `CanDowncast` calls [`to_extractor`](../variant/has_extractor.md) on an enum to
obtain the extractor, and this trait takes one it is handed.

Because the remainder type narrows at every step, a chain's types are all distinct, and the compiler
knows at each point exactly which variants remain, the same narrowing a hand-written
[extraction chain](../variant/extract_field.md) performs. So once the candidates have covered every
variant, the last remainder is uninhabited and the final `Result` closes with
[`finalize_extract_result`](../variant/finalize_extract_result.md), as the example's last step does.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::impls`.

**It cannot be called on an enum, and `downcast` cannot be called on a remainder.** `Self` here is
an extractor, and `downcast` needs an enum to call `to_extractor` on. Writing the second step of the
example as `remainder.downcast(PhantomData::<JustBar>)` fails with:

```text
error[E0599]: the method `downcast` exists for enum `__PartialFooBarBaz<IsVoid, IsPresent, IsPresent>`, but its trait bounds were not satisfied
...
note: the trait `HasExtractor` must be implemented
```

Start a chain with [`CanDowncast`](./can_downcast.md) and continue it with this trait.

**`.ok()` on an intermediate step ends the chain.** Discarding the remainder discards the input the next
attempt needs.

**A remainder carries none of the enum's attributes**, so a `Result` holding one is neither `Debug` nor
`PartialEq`.

**Each step has a different `Self` type**, so a chain cannot be written as a loop or collected into a
`Vec` of attempts: it is unrolled by construction.

## Related constructs

- [`CanDowncast`](./can_downcast.md): the first step, and where the pair is explained in full.
- [`CanUpcast`](./can_upcast.md): the widening direction.
- [`ExtractField`](../variant/extract_field.md): narrowing to a payload rather than to another enum.
- [`FinalizeExtract`](../variant/finalize_extract.md): how an exhausted remainder is discharged.
- [`HasExtractor`](../variant/has_extractor.md): what turns an enum into the extractor this operates on.
- [`#[derive(CgpVariant)]`](../../derives/derive_cgp_variant.md): what makes an enum eligible.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants): partial variants and the narrowing chain.

## Source

- [`cast.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/cast.rs):
  `CanDowncastFields` and the `FieldsExtractor` recursion

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
