---
title: 'HasExtractorMut — edit a variant in place'
sidebar_label: 'HasExtractorMut'
sidebar_position: 4
description: 'Obtain an extractor that borrows an enum mutably, so a named variant''s payload can be changed in place without consuming or rebuilding the value.'
---

# `HasExtractorMut`

Obtaining an extractor that can change a payload in place.

## Overview

[`HasExtractorRef`](./has_extractor_ref.md) borrows an enum so its variants can be read without consuming
it. `HasExtractorMut` borrows it mutably, so a payload can be **changed in place**.

Payloads come out as `&mut T`. The original survives, mutably borrowed for the duration, and the
narrowing works exactly as it does for the other two accessors: [`ExtractField`](./extract_field.md)
does not care how the payloads are held.

It is the third of the group, and the one to reach for least often:

| | payloads come out as | the original |
|---|---|---|
| [`HasExtractor`](./has_extractor.md) | owned | consumed |
| [`HasExtractorRef`](./has_extractor_ref.md) | `&T` | survives |
| `HasExtractorMut` | `&mut T` | survives, mutably borrowed |

## Definition

`HasExtractorMut` produces a mutably-borrowing extractor:

```rust
pub trait HasExtractorMut {
    type ExtractorMut<'a>
    where
        Self: 'a;

    fn extractor_mut(&mut self) -> Self::ExtractorMut<'_>;
}
```

`Self` is the enum. `ExtractorMut<'a>` is a **generic associated type**: the borrowed extractor over
the same partial companion enum, with every payload held as a mutable reference for `'a`. The
`where Self: 'a` clause keeps it from outliving the value. `extractor_mut` takes a mutable borrow
and returns the extractor at the anonymous lifetime, so the call reads without a written lifetime
and the borrow ends where the extractor is dropped.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

The impls come from [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md), alongside the owning
and shared-borrow accessors.

## Examples

Changing whichever payload is present, without rebuilding the enum:

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

pub fn scale(shape: &mut Shape, factor: f64) {
    match shape.extractor_mut().extract_field(PhantomData::<Symbol!("Circle")>) {
        Ok(circle) => circle.radius *= factor,
        Err(remainder) => {
            let rect = remainder
                .extract_field(PhantomData::<Symbol!("Rectangle")>)
                .finalize_extract_result();
            rect.width *= factor;
            rect.height *= factor;
        }
    }
}

pub fn demo() {
    let mut shape = Shape::Circle(Circle { radius: 1.0 });
    scale(&mut shape, 5.0);

    // `shape` now holds the updated `Circle`.
    assert_eq!(shape, Shape::Circle(Circle { radius: 5.0 }));
}
```

Each payload arrives as `&mut`, so the writes land in the original value, and the chain still ends
in a checked `finalize_extract_result`. Written with [`HasExtractor`](./has_extractor.md) instead,
the same code would consume `shape` and have to rebuild it.

## When to use it

**Reach for it only when a payload must be mutated in place**, which is the narrowest of the three cases.

- **[`HasExtractorRef`](./has_extractor_ref.md)** for a read. Requiring mutable access where a shared
  borrow would do prevents a caller from holding any other reference to the value.
- **[`HasExtractor`](./has_extractor.md)** when the payload must be moved out.
- **`HasExtractorMut`** when the value stays where it is and one of its payloads changes.
- **A `match` on `&mut`** when the enum is concrete. `if let Shape::Circle(c) = &mut shape` does this in
  a line, and the family is for code that cannot name the enum.
- **Consider returning a new value instead.** Much CGP code threads state through handler outputs rather
  than mutating in place, which composes better with the [handler family](../../components/handler/handler.md).

## Under the hood

It uses the **same borrowed companion** as
[`HasExtractorRef`](./has_extractor_ref.md#under-the-hood), with the
[`MapTypeRef`](../type-level/map_type_ref.md) marker fixed to `IsMut` rather than `IsRef`.
`cargo cgp expand` on the example's `Shape` shows the impl:

```rust
impl HasExtractorMut for Shape {
    type ExtractorMut<'__a__> = __PartialRefShape<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(&'__a__ mut self) -> Self::ExtractorMut<'__a__> {
        match self {
            Self::Circle(value) => __PartialRefShape::Circle(value),
            Self::Rectangle(value) => __PartialRefShape::Rectangle(value),
        }
    }
}
```

Since `IsMut::Map<'a, T>` is `&'a mut T`, every payload slot becomes a mutable reference while the
per-variant [`MapType`](../type-level/map_type.md) markers continue to track possibility. That one
marker is the whole difference between the two borrowing accessors, and the same
[`ExtractField`](./extract_field.md) and [`FinalizeExtract`](./finalize_extract.md) impls serve
both.

The trait lacks a rebuild counterpart, and needs none, because the value was never taken apart, only
borrowed.

## Common Mistakes

**It takes `&mut self`, so nothing else may borrow the value** for as long as the extractor or any
payload taken from it lives. That is ordinary borrow checking.

**Payloads are `&mut T`, so a chain cannot move one out.** Taking ownership needs
[`HasExtractor`](./has_extractor.md).

**It cannot rebuild the enum.** `from_extractor` belongs to the owning accessor alone.

**The `where Self: 'a` clause propagates**, so a signature holding an `ExtractorMut<'a>` rarely elides
its lifetime cleanly.

**The owned and borrowed extractors are different enums.** The two borrowing accessors share
`__PartialRefShape` and differ in its [`MapTypeRef`](../type-level/map_type_ref.md) marker, while
the owned one is `__PartialShape`, so code generic over "an extractor" is generic over the extractor
type itself.

**A remainder still carries none of the enum's attributes**, so a `Result` holding one is neither
`Debug` nor `PartialEq`.

## Related constructs

- [`HasExtractorRef`](./has_extractor_ref.md): the shared-borrow accessor, and the one to prefer.
- [`HasExtractor`](./has_extractor.md): the owning accessor, and where the group is compared.
- [`ExtractField`](./extract_field.md): the narrowing, identical through a mutable borrow.
- [`FinalizeExtract`](./finalize_extract.md) and
  [`FinalizeExtractResult`](./finalize_extract_result.md): how a chain ends.
- [`MapTypeRef`](../type-level/map_type_ref.md): the `IsMut` marker this fixes.
- [`MapType`](../type-level/map_type.md): the per-variant markers it composes with.
- [`HasFieldMut`](../field-access/has_field_mut.md): the record side's mutable access.
- [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md): generates the borrowed companion.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants): matching a variant without consuming the
  value.

## Source

- [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs):
  `HasExtractorMut` and the rest of the family
- [`impls/map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/map_type_ref.rs):
  the `IsMut` marker

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
