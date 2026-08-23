---
sidebar_label: 'HasExtractorRef'
---

# `HasExtractorRef`

Obtaining an extractor that borrows the value rather than consuming it.

## Overview

[`HasExtractor`](./has_extractor.md) consumes an enum to produce an extractor with owned payloads. Code
that only *reads* a variant — checking it, measuring it, rendering it — should not have to give up the
value. `HasExtractorRef` is the borrowing accessor:

```rust
pub trait HasExtractorRef {
    type ExtractorRef<'a>
    where
        Self: 'a;

    fn extractor_ref(&self) -> Self::ExtractorRef<'_>;
}
```

Payloads come out as shared references and the original survives. **The narrowing works identically** —
[`ExtractField`](./extract_field.md) rules variants out of a borrowed extractor exactly as it does an
owned one, and the chain ends the same way.

It is the weakest of the three accessors, so prefer it wherever it suffices.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

`ExtractorRef<'a>` is a **generic associated type** with a `where Self: 'a` clause, so the borrowed
extractor cannot outlive the value it came from. `extractor_ref(&self)` takes a shared borrow and returns
one at the anonymous lifetime, which is why the call reads with no lifetime written.

The impls come from [`#[derive(ExtractField)]`](../derives/derive_extract_field.md), alongside the owning
and mutable accessors.

## Examples

Reading through a borrow leaves the value intact:

```rust
use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

let radius = shape
    .extractor_ref()
    .extract_field(PhantomData::<Symbol!("Circle")>)
    .map(|circle| circle.radius)
    .ok();

// `shape` is still usable here
```

The payload arrives as `&Circle`, so `circle.radius` reads through the borrow and nothing is moved. A
full chain narrows the same way, with each remainder also borrowing.

## When to use it

**Reach for it whenever the value must survive**, which is most read-only code over an enum.

- **`HasExtractorRef`** for a read-only operation. Prefer it over consuming, since requiring ownership
  narrows what a caller can pass for no benefit.
- **[`HasExtractorMut`](./has_extractor_mut.md)** to change a payload in place.
- **[`HasExtractor`](./has_extractor.md)** only when the payload must be moved out.
- **A `match`** when the enum is concrete. This family is for code that cannot name it.
- **[`ToFieldsRef`](./to_fields_ref.md)** when what you want is the value's borrowed *shape* rather than
  a narrowing chain — the same borrow-rather-than-consume idea, applied to the whole-shape view.

## Under the hood

The borrowed accessor uses the **same partial enum** as the owning one, with an extra
[`MapTypeRef`](./map_type_ref.md) parameter fixed to `IsRef`:

```rust
// conceptually
//
// pub enum __PartialRefShape<'a, R: MapTypeRef, F0: MapType, F1: MapType> {
//     Circle(F0::Map<R::Map<'a, Circle>>),
//     Rectangle(F1::Map<R::Map<'a, Rectangle>>),
// }
//
// impl HasExtractorRef for Shape {
//     type ExtractorRef<'a> = __PartialRefShape<'a, IsRef, IsPresent, IsPresent>;
// }
```

Read the nesting outward: the [`MapTypeRef`](./map_type_ref.md) marker decides *how* a payload is held —
`IsRef::Map<'a, T>` is `&'a T` — and the per-variant [`MapType`](./map_type.md) markers decide *whether*
it is still possible. The two axes are independent, which is precisely why narrowing behaves the same
through a borrow as through an owned value.

There is no `from_extractor` counterpart here: a borrowed extractor cannot rebuild an owned enum, and the
original is still there anyway.

## Common Mistakes

**There is no rebuild.** [`HasExtractor`](./has_extractor.md)'s `from_extractor` has no borrowing
equivalent, which is rarely a problem since the value was never consumed.

**Payloads are `&T`, so a chain cannot move one out.** A routine that must take ownership of a payload
needs [`HasExtractor`](./has_extractor.md).

**The `where Self: 'a` clause propagates.** A signature that stores or returns `ExtractorRef<'a>` usually
has to spell the lifetime out rather than elide it.

**A remainder still carries none of the enum's attributes**, so a `Result` holding one is neither `Debug`
nor `PartialEq`.

**The borrowed and owned extractors are different types.** Code generic over "an extractor" has to pick
one, or be generic over the [`MapTypeRef`](./map_type_ref.md) marker as well.

## Related constructs

- [`HasExtractor`](./has_extractor.md) — the owning accessor, and where the group is compared.
- [`HasExtractorMut`](./has_extractor_mut.md) — the mutable accessor.
- [`ExtractField`](./extract_field.md) — the narrowing, identical through a borrow.
- [`FinalizeExtract`](./finalize_extract.md) and
  [`FinalizeExtractResult`](./finalize_extract_result.md) — how a chain ends.
- [`MapTypeRef`](./map_type_ref.md) — the `IsRef` marker this fixes.
- [`MapType`](./map_type.md) — the per-variant markers it composes with.
- [`ToFieldsRef`](./to_fields_ref.md) — the record side's borrow-rather-than-consume view.
- [`#[derive(ExtractField)]`](../derives/derive_extract_field.md) — generates the borrowed companion.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — matching a variant without consuming the
  value.

## Source

- [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)
  — `HasExtractorRef` and the rest of the family
- [`impls/map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/map_type_ref.rs)
  — the `IsRef` marker

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
