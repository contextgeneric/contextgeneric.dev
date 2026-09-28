---
title: 'HasExtractorRef — extract through a borrow'
sidebar_label: 'HasExtractorRef'
sidebar_position: 3
description: 'Obtain an extractor that borrows an enum, so its variants can be matched by name with payloads as shared references and the value left intact.'
---

# `HasExtractorRef`

Obtaining an extractor that borrows the value rather than consuming it.

## Overview

[`HasExtractor`](./has_extractor.md) consumes an enum to produce an extractor with owned payloads. Code
that only *reads* a variant (checking it, measuring it, rendering it) should not have to give up the
value. `HasExtractorRef` is the borrowing accessor.

Payloads come out as shared references and the original survives. **The narrowing works identically**:
[`ExtractField`](./extract_field.md) rules variants out of a borrowed extractor exactly as it does an
owned one, and the chain ends the same way.

It is the weakest of the three accessors, so prefer it wherever it suffices.

## Definition

`HasExtractorRef` produces a borrowing extractor:

```rust
pub trait HasExtractorRef {
    type ExtractorRef<'a>
    where
        Self: 'a;

    fn extractor_ref(&self) -> Self::ExtractorRef<'_>;
}
```

`Self` is the enum. `ExtractorRef<'a>` is a **generic associated type**: the borrowed extractor over the
same partial companion enum, with every payload held for the lifetime `'a`. The `where Self: 'a` clause
keeps the borrowed extractor from outliving the value it came from. `extractor_ref` takes a shared borrow
of the value and returns the extractor at the anonymous lifetime, which is why the call reads with no
lifetime written.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

The impls come from [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md), alongside the owning
and mutable accessors.

## Examples

Reading through a borrow, as a single attempt and as a full chain, leaves the value intact:

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

#[derive(Debug, PartialEq, ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

pub fn radius(shape: &Shape) -> Option<f64> {
    shape
        .extractor_ref()
        .extract_field(PhantomData::<Symbol!("Circle")>)
        .map(|circle| circle.radius)
        .ok()
}

pub fn area(shape: &Shape) -> f64 {
    match shape.extractor_ref().extract_field(PhantomData::<Symbol!("Circle")>) {
        Ok(circle) => core::f64::consts::PI * circle.radius * circle.radius,
        Err(remainder) => {
            let rect = remainder
                .extract_field(PhantomData::<Symbol!("Rectangle")>)
                .finalize_extract_result();
            rect.width * rect.height
        }
    }
}

pub fn demo() {
    let shape = Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 });
    assert_eq!(radius(&shape), None);
    assert_eq!(area(&shape), 12.0);

    // `shape` is still usable here.
    assert_eq!(shape, Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 }));
}
```

The payloads arrive as `&Circle` and `&Rectangle`, so fields read through the borrow and nothing is
moved. The full chain narrows and finalizes exactly as an owned one does, with each remainder also
borrowing.

## When to use it

**Reach for it whenever the value must survive**, which is most read-only code over an enum.

- **`HasExtractorRef`** for a read-only operation. Prefer it over consuming, since requiring
  ownership narrows what a caller can pass without any benefit.
- **[`HasExtractorMut`](./has_extractor_mut.md)** to change a payload in place.
- **[`HasExtractor`](./has_extractor.md)** only when the payload must be moved out.
- **A `match`** when the enum is concrete. This family is for code that cannot name it.
- **[`ToFieldsRef`](../shape/to_fields_ref.md)** when what you want is the value's borrowed *shape* rather than
  a narrowing chain: the same borrow-rather-than-consume idea, applied to the whole-shape view.

## Under the hood

The borrowing accessors use a **second companion enum**, `__PartialRefShape`, beside the owned
`__PartialShape`. `cargo cgp expand` on the example's `Shape` shows it:

```rust
pub enum __PartialRefShape<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Circle>>),
    Rectangle(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Rectangle>>),
}
```

and this trait's impl, which fixes the [`MapTypeRef`](../type-level/map_type_ref.md) marker to
`IsRef` and every variant to still possible:

```rust
impl HasExtractorRef for Shape {
    type ExtractorRef<'__a__> = __PartialRefShape<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(&'__a__ self) -> Self::ExtractorRef<'__a__> {
        match self {
            Self::Circle(value) => __PartialRefShape::Circle(value),
            Self::Rectangle(value) => __PartialRefShape::Rectangle(value),
        }
    }
}
```

Read the payload type outward: the [`MapTypeRef`](../type-level/map_type_ref.md) marker decides
*how* a payload is held (`IsRef::Map<'a, T>` is `&'a T`), and the per-variant
[`MapType`](../type-level/map_type.md) marker decides *whether* it is still possible. The two axes
are independent, which is why narrowing behaves the same through a borrow as through an owned value.
The derive emits the per-variant [`ExtractField`](./extract_field.md) impls and the all-`IsVoid`
[`FinalizeExtract`](./finalize_extract.md) impl on this enum too, generic over the `MapTypeRef`
marker, so a borrowed chain finalizes exactly as an owned one does.

The trait lacks a `from_extractor` counterpart: a borrowed extractor cannot rebuild an owned enum,
and the original is still there anyway.

## Common Mistakes

**It cannot rebuild the enum.** [`HasExtractor`](./has_extractor.md)'s `from_extractor` lacks a
borrowing equivalent, which is rarely a problem since the value was never consumed.

**Payloads are `&T`, so a chain cannot move one out.** A routine that must take ownership of a payload
needs [`HasExtractor`](./has_extractor.md).

**The `where Self: 'a` clause propagates.** A signature that stores or returns `ExtractorRef<'a>` usually
has to spell the lifetime out rather than elide it.

**A remainder still carries none of the enum's attributes**, so a `Result` holding one is neither `Debug`
nor `PartialEq`.

**The borrowed and owned extractors are different enums**, `__PartialRefShape` and
`__PartialShape`. Code generic over "an extractor" is generic over the extractor type itself, with
[`ExtractField`](./extract_field.md) bounds on it, rather than over a marker.

## Related constructs

- [`HasExtractor`](./has_extractor.md): the owning accessor, and where the group is compared.
- [`HasExtractorMut`](./has_extractor_mut.md): the mutable accessor.
- [`ExtractField`](./extract_field.md): the narrowing, identical through a borrow.
- [`FinalizeExtract`](./finalize_extract.md) and
  [`FinalizeExtractResult`](./finalize_extract_result.md): how a chain ends.
- [`MapTypeRef`](../type-level/map_type_ref.md): the `IsRef` marker this fixes.
- [`MapType`](../type-level/map_type.md): the per-variant markers it composes with.
- [`ToFieldsRef`](../shape/to_fields_ref.md): the record side's borrow-rather-than-consume view.
- [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md): generates the borrowed companion.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants): matching a variant without consuming the
  value.

## Source

- [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs):
  `HasExtractorRef` and the rest of the family
- [`impls/map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/map_type_ref.rs):
  the `IsRef` marker

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
