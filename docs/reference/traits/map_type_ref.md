---
sidebar_label: 'MapTypeRef'
---

# `MapTypeRef`

The marker trait deciding how a borrowed partial type holds its payloads.

:::info

### Generated machinery

**You are not expected to implement `MapTypeRef`.** Its three markers are supplied
by CGP, and [`#[derive(ExtractField)]`](../derives/derive_extract_field.md) generates the borrowed
companion that carries one. What you do with this trait is recognize its markers in an error; this page
explains what each one means and how it composes with the per-field markers. The one case for bounding on the trait is generic code of your own over a borrowed partial type's outer marker.

:::

## Overview

[`MapType`](./map_type.md) decides whether a field of a partial type holds its value, nothing, or an
uninhabited type. A **borrowed** partial type needs a second decision on top: whether each payload is
held by shared reference, by mutable reference, or by value. That decision brings a lifetime with it,
which is why it needs its own trait:

```rust
pub trait MapTypeRef {
    type Map<'a, T: 'a>: 'a;
}
```

Three markers implement it:

```rust
impl MapTypeRef for IsRef   { type Map<'a, T: 'a> = &'a T; }
impl MapTypeRef for IsMut   { type Map<'a, T: 'a> = &'a mut T; }
impl MapTypeRef for IsOwned { type Map<'a, T: 'a> = T; }
```

**The two families compose rather than compete.** A borrowed extractor carries one outer `MapTypeRef`
marker shared across all fields and one [`MapType`](./map_type.md) marker per field, so a field's storage
is `MapType::Map<MapTypeRef::Map<'a, T>>`. [`HasExtractorRef`](./has_extractor_ref.md) fixes the outer
marker to `IsRef` and [`HasExtractorMut`](./has_extractor_mut.md) to `IsMut` — which is what lets a value
be matched without being moved, with the narrowing working identically.

## Usage

**`IsRef` and `IsMut` are in the prelude; `IsOwned` is not** — import it from `cgp::core::field::impls`.
The trait itself comes with the prelude.

The associated type is generic over both a lifetime and a type, with `T: 'a` required and the result
bounded `'a`. That is more machinery than [`MapType`](./map_type.md)'s single-parameter `Map<T>`, and it
is what makes the borrowed shape expressible at whatever lifetime a caller has.

:::warning

**`IsOwned` has no consumer in CGP.** No derive emits it and no provider resolves against it. It is
available for a borrowed view that holds its payloads by value, and at present that view exists only if
you write the machinery yourself.

:::

## Examples

You meet these markers inside the borrowed companion types the derives generate. Reading a variant
through a shared borrow leaves the value intact:

```rust
use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

let radius = shape
    .extractor_ref()                                        // outer marker is IsRef
    .extract_field(PhantomData::<Symbol!("Circle")>)
    .map(|circle| circle.radius)
    .ok();
```

and the mutable form changes a payload in place:

```rust
if let Ok(circle) = shape
    .extractor_mut()                                        // outer marker is IsMut
    .extract_field(PhantomData::<Symbol!("Circle")>)
{
    circle.radius = 5.0;
}
```

Neither call names a marker. What the markers do is decide the type the payload comes out as — `&Circle`
in the first case, `&mut Circle` in the second — while the per-field
[`MapType`](./map_type.md) markers track which variants are still possible.

## When to use it

**You will recognize these markers far more often than you name them**, and recognizing them is most of
what this page is for: an error mentioning `IsRef` is telling you the value is being read through
[`extractor_ref`](./has_extractor_ref.md) rather than consumed.

- **Use [`HasExtractorRef`](./has_extractor_ref.md) or [`HasExtractorMut`](./has_extractor_mut.md)**
  rather than naming a marker. They fix the outer marker for you, and they are the constructs this trait
  serves.
- **Bound on `MapTypeRef`** only when writing generic machinery over a borrowed partial type's outer
  marker.
- **Do not select `IsOwned` expecting it to work.** Nothing in CGP consumes it.
- **Use [`MapType`](./map_type.md)** for the per-field state, which is the decision this one sits beside
  rather than replaces.
- **Use [`ToFieldsRef`](./to_fields_ref.md)** when what you want is the borrowed *shape* of a struct
  rather than a borrowed extractor. That trait solves the same problem for the record side, without
  markers.

## Under the hood

A borrowed companion enum carries the outer marker as an extra parameter alongside the per-variant ones,
and each payload slot projects through both:

```rust
// conceptually, for `enum Shape { Circle(Circle), Rectangle(Rectangle) }`
//
// pub enum __PartialRefShape<'a, R: MapTypeRef, F0: MapType, F1: MapType> {
//     Circle(F0::Map<R::Map<'a, Circle>>),
//     Rectangle(F1::Map<R::Map<'a, Rectangle>>),
// }
```

Read the nesting outward: `R::Map<'a, T>` decides *how* the payload is held, and `F::Map<…>` decides
*whether* it is there at all. Fixing `R = IsRef` gives a shared-borrow extractor; flipping an `F` from
`IsPresent` to `IsVoid` rules that variant out. The two axes are independent, which is why narrowing
behaves identically through a borrow and through an owned value.

The `T: 'a` bound and the `: 'a` bound on the associated type are what keep the projection well-formed:
a payload cannot be borrowed for longer than it lives, and the resulting storage type cannot outlive the
borrow either.

## Common Mistakes

**`IsOwned` is unused by CGP.** It is a legal marker with no consumer, so selecting it means writing the
machinery that uses it yourself.

**`IsOwned` is not in the prelude.** Import it from `cgp::core::field::impls`. `IsRef` and `IsMut` are.

**This is not [`MapType`](./map_type.md).** One decides presence per field, the other decides ownership
across the whole borrowed view, and a generated borrowed type carries both. An error naming the wrong one
usually means the owned and borrowed forms have been crossed.

**The associated type takes a lifetime as well as a type.** A bound over it is more verbose than a
`MapType` bound, and eliding the lifetime rarely works.

**A mutable extractor cannot coexist with another borrow of the same value**, which is ordinary borrow
checking rather than anything CGP adds — but it surfaces as an error about the companion type, which
reads as though the machinery is at fault.

## Related constructs

- [`MapType`](./map_type.md) — the per-field presence marker this composes with.
- [`HasExtractorRef`](./has_extractor_ref.md) and [`HasExtractorMut`](./has_extractor_mut.md) — the two
  accessors that fix the outer marker.
- [`ExtractField`](./extract_field.md) — the narrowing that works identically through a borrow.
- [`ToFieldsRef`](./to_fields_ref.md) — the record side's answer to the same problem.
- [`HasFieldsRef`](./has_fields_ref.md) — the borrowed shape it produces.
- [`#[derive(ExtractField)]`](../derives/derive_extract_field.md) — generates the borrowed companion.
- [Type-level spines](../types/type_level_spines.md) — the `Either`/`Void` chain underneath.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — matching a variant without consuming the
  value.

## Source

- [`map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_type_ref.rs)
  — the trait
- [`impls/map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/map_type_ref.rs)
  — `IsRef`, `IsMut`, `IsOwned`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
