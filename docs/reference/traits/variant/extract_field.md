---
title: 'ExtractField — take one variant out'
sidebar_label: 'ExtractField'
sidebar_position: 1
description: 'Try one named variant of an enum''s extractor, getting its payload or a remainder with that variant ruled out, so a chain ends in a proven-exhaustive match.'
---

# `ExtractField`

Pulling one variant out of an extractor, or narrowing what remains.

## Overview

A `match` on a concrete enum is checked for exhaustiveness. Code that is generic over the enum cannot
write one, so it falls back on a wildcard arm and an `unreachable!()`, losing exactly the guarantee you
wanted.

`ExtractField` is the operation that recovers it. Each attempt either yields the payload or hands back a
*remainder* whose type has that variant ruled out, and **the impl exists only while the requested variant
is still possible**, so attempting the same variant twice is a compile error rather than a guaranteed
miss.

Keep going and the remainder narrows. Once every variant has been ruled out its type is
**uninhabited** (a value of it cannot exist), and [`FinalizeExtract`](./finalize_extract.md) closes
the chain with without a wildcard or a panic path. Add a variant to the enum and the final remainder
becomes inhabited again, so the code stops compiling until it is handled.

The family is the mirror of the [builder](../builder/has_builder.md): a builder tracks which fields are *present*,
an extractor tracks which variants are still *possible*.

## Definition

`ExtractField<Tag>` takes one variant, named by a type-level tag, out of an extractor:

```rust
pub trait ExtractField<Tag> {
    type Value;
    type Remainder;

    fn extract_field(self, _tag: PhantomData<Tag>) -> Result<Self::Value, Self::Remainder>;
}
```

`Self` is an extractor rather than the enum. `Tag` is the variant's name as a
[`Symbol!`](../../macros/symbol.md), `Value` is that variant's payload type, and `Remainder` is the same
extractor with this one variant ruled out. `extract_field` consumes the extractor and takes a
`PhantomData<Tag>` to name the variant, returning `Ok(value)` when the runtime value is that variant and
`Err(remainder)` when it is not.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

`Self` is an extractor rather than the enum, so a chain begins by obtaining one:
[`to_extractor`](./has_extractor.md) to consume the value, [`extractor_ref`](./has_extractor_ref.md) to
borrow it, or [`extractor_mut`](./has_extractor_mut.md) to borrow it mutably. Which one you pick decides
whether the payloads come out owned, shared, or mutable; the narrowing is identical in all three.

The `PhantomData<Tag>` argument names the variant, keyed by [`Symbol!`](../../macros/symbol.md) of its name.

The impls come from [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md), also available
through [`#[derive(CgpVariant)]`](../../derives/derive_cgp_variant.md) and
[`#[derive(CgpData)]`](../../derives/derive_cgp_data.md).

## Examples

The chain reads as a sequence of attempts, each handling one variant, closed by
[`finalize_extract_result`](./finalize_extract_result.md), written here in both orders:

```rust
use cgp::prelude::*;
use cgp::core::field::traits::FinalizeExtractResult;

#[derive(Debug, PartialEq)]
pub struct Circle {
    pub radius: f64,
}

#[derive(Debug, PartialEq)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

pub fn area(shape: Shape) -> f64 {
    match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
        Ok(circle) => core::f64::consts::PI * circle.radius * circle.radius,
        Err(remainder) => {
            // `remainder` can no longer be a `Circle`.
            let rect = remainder
                .extract_field(PhantomData::<Symbol!("Rectangle")>)
                .finalize_extract_result();
            rect.width * rect.height
        }
    }
}

// The same chain with the extractions the other way round.
pub fn perimeter(shape: Shape) -> f64 {
    match shape.to_extractor().extract_field(PhantomData::<Symbol!("Rectangle")>) {
        Ok(rect) => 2.0 * (rect.width + rect.height),
        Err(remainder) => {
            let circle = remainder
                .extract_field(PhantomData::<Symbol!("Circle")>)
                .finalize_extract_result();
            2.0 * core::f64::consts::PI * circle.radius
        }
    }
}

pub fn demo() {
    assert_eq!(area(Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 })), 12.0);
    assert_eq!(perimeter(Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 })), 14.0);
    assert!(area(Shape::Circle(Circle { radius: 1.0 })) > 3.14);
}
```

After the second extraction both variants are ruled out, so the remainder's type is uninhabited and
the finalize is accepted without a wildcard arm. The type enforces this: finalize after only the
first extraction and the code does not compile. The two functions try the variants in opposite
orders, which works because each step changes only its own variant's marker.

**In practice you rarely write these chains.** The [dispatch
combinators](../../providers/dispatch/index.md) build them from a set of per-variant
implementations, which is the extensible visitor pattern: a chain like the one above, generated,
with one implementation per variant chosen by wiring.

## When to use it

**Reach for the family when independent code handles one variant each, or when the matching code cannot
name the enum.** Outside those two cases a `match` wins on every count: shorter, clearer, already
exhaustive, and it generates nothing. This is not an improvement on `match`; you reach for it only where
`match` is unavailable.

- **Bound on [`HasExtractor`](./has_extractor.md) + `ExtractField`** to write a routine that
  deconstructs an enum it does not name.
- **Pick the accessor by ownership**, preferring the weakest that works:
  [`extractor_ref`](./has_extractor_ref.md) for a read-only operation,
  [`extractor_mut`](./has_extractor_mut.md) to mutate a payload,
  [`to_extractor`](./has_extractor.md) only when you genuinely want to consume.
- **Do not use it to test which variant a value holds.** `matches!` or an `if let` answers that in a
  line. The family's value is in the *chain* and what the chain proves.
- **Reach for the [dispatch combinators](../../providers/dispatch/index.md) rather than writing the
  chain**, since they derive it from the enum's own variant list instead of repeating it at each site,
  which keeps "add a variant" from breaking every call site by hand.

The construction counterpart is [`FromVariant`](./from_variant.md), and the two are commonly wanted
together because a generic pipeline usually takes a value apart and puts one back. The struct analogue of
the whole family is [`HasBuilder`](../builder/has_builder.md).

## Under the hood

The derive generates a companion enum with one [`MapType`](../type-level/map_type.md) parameter per
variant, each payload wrapped in that parameter's projection. `cargo cgp expand` on the example's
`Shape` shows it:

```rust
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Rectangle(<__F1__ as MapType>::Map<Rectangle>),
}
```

[`to_extractor`](./has_extractor.md) starts at the all-`IsPresent` configuration, where every
variant is still possible. The derive then writes one `ExtractField` impl per variant, and the
`Circle` one shows the whole mechanism:

```rust
impl<__F1__: MapType> ExtractField<Symbol!("Circle")>
for __PartialShape<IsPresent, __F1__> {
    type Value = Circle;
    type Remainder = __PartialShape<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Circle")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialShape::Circle(value) => Ok(value),
            __PartialShape::Rectangle(value) => Err(__PartialShape::Rectangle(value)),
        }
    }
}
```

**The impl exists only while `Circle`'s marker is `IsPresent`**, and on a miss it returns the
remainder with that one marker flipped to `IsVoid` while `__F1__` passes through, which is why
extractions may happen in any order. That per-variant scoping is the mirror of
[`BuildField`](../builder/build_field.md)'s requirement that a field be `IsNothing` before it can be
set, and it makes a repeated attempt a compile error rather than a guaranteed `Err`.

The exhaustiveness argument itself belongs to [`FinalizeExtract`](./finalize_extract.md), and turns
on `IsVoid` mapping a payload to the uninhabited `Void`.

The borrowed accessors use a second companion, `__PartialRefShape`, with a lifetime and an extra
[`MapTypeRef`](../type-level/map_type_ref.md) parameter that
[`HasExtractorRef`](./has_extractor_ref.md) fixes to `IsRef` and
[`HasExtractorMut`](./has_extractor_mut.md) to `IsMut`. The derive emits the same per-variant
`ExtractField` impls on it, so a value can be matched without being moved and the narrowing works
identically.

## Common Mistakes

**`Self` is an extractor, not the enum.** A chain starts with [`to_extractor`](./has_extractor.md) or one
of its borrowing siblings.

**Attempting the same variant twice does not compile, and the error is not about the variant.**
Extracting `Circle` again from the remainder of a failed `Circle` attempt:

```rust
if let Err(remainder) = shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
    let _ = remainder.extract_field(PhantomData::<Symbol!("Circle")>);
}
```

leaves only the `Rectangle` impl applicable, so rustc settles on it and reports the tag argument as
the wrong type:

```text
error[E0308]: mismatched types
...
   |                           ------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `9`, found `6`
```

The two numbers are the lengths of `Rectangle` and `Circle` in their `Symbol` types, and the `note`
lines below spell both out.

**Absence is `IsVoid` here and `IsNothing` in a builder, and they are not interchangeable.** An error
naming the wrong one usually means record and variant machinery have been crossed.

**A remainder carries none of the enum's attributes.** The partial enums are generated without your
derives, so a `Result<Payload, Remainder>` is neither `Debug` nor `PartialEq` however the enum is
derived. On a `Shape` deriving both:

```rust
assert_eq!(
    shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>),
    Ok(Circle { radius: 1.0 })
);
```

fails with

```text
error[E0369]: binary operation `==` cannot be applied to type `Result<Circle, __PartialShape<IsVoid, IsPresent>>`
...
error[E0277]: `__PartialShape<IsVoid, IsPresent>` doesn't implement `Debug`
```

Reach for `.ok()`, `.is_ok()`, or a `match`.

**Order is free but the set is not.** Each step changes only its own variant's marker, so extractions may
be written in any order, but every variant must be tried before the remainder can be finalized.

**Adding a variant breaks every hand-written chain, by design.** That is the guarantee, and it is the
reason to prefer the [dispatch combinators](../../providers/dispatch/index.md).

**Five enum variant names are reserved**, because the generated impls name their associated types through
`Self::…`. The [derive's page](../../derives/derive_extract_field.md) lists them.

## Related constructs

- [`HasExtractor`](./has_extractor.md), [`HasExtractorRef`](./has_extractor_ref.md), and
  [`HasExtractorMut`](./has_extractor_mut.md): the three ways to obtain an extractor.
- [`FinalizeExtract`](./finalize_extract.md) and
  [`FinalizeExtractResult`](./finalize_extract_result.md): how a chain ends.
- [`FromVariant`](./from_variant.md): the construction counterpart.
- [`HasBuilder`](../builder/has_builder.md): the struct analogue of the whole family.
- [`MapType`](../type-level/map_type.md): the `IsPresent`/`IsVoid` markers, and where the contrast with `IsNothing`
  is explained.
- [`CanDowncast`](../casting/can_downcast.md): narrowing to another enum rather than to a payload, built on this.
- [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md): generates the partial enums and every
  impl here.
- [Dispatch combinators](../../providers/dispatch/index.md): the providers that build the chain for
  you.
- [Type-level lists](../../types/index.md): `Either`/`Void`, where the uninhabited terminator
  comes from.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants): partial variants, the exhaustiveness
  argument, and the extensible visitor pattern.
- [Dispatching](/docs/concepts/dispatching): routing a variant to the implementation that handles it.

## Source

- [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs):
  `ExtractField` and the rest of the family

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
