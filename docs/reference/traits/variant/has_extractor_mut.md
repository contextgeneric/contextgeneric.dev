---
sidebar_label: 'HasExtractorMut'
sidebar_position: 4
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

`Self` is the enum. `ExtractorMut<'a>` is a **generic associated type**: the borrowed extractor over the
same partial companion enum, with every payload held as a mutable reference for `'a`. The `where Self:
'a` clause keeps it from outliving the value. `extractor_mut` takes a mutable borrow and returns the
extractor at the anonymous lifetime, so the call reads with no lifetime written and the borrow ends where
the extractor is dropped.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

The impls come from [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md), alongside the owning
and shared-borrow accessors.

## Examples

Changing a payload without rebuilding the enum:

```rust
use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

if let Ok(circle) = shape
    .extractor_mut()
    .extract_field(PhantomData::<Symbol!("Circle")>)
{
    circle.radius = 5.0;
}

// `shape` now holds the updated Circle
```

The payload arrives as `&mut Circle`, so the write lands in the original value. Written with
[`HasExtractor`](./has_extractor.md) instead, the same code would consume `shape` and have to rebuild it.

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

It uses the **same borrowed companion** as [`HasExtractorRef`](./has_extractor_ref.md#under-the-hood),
with the outer [`MapTypeRef`](../type-level/map_type_ref.md) marker fixed to `IsMut` rather than `IsRef`:

```rust
// impl HasExtractorMut for Shape {
//     type ExtractorMut<'a> = __PartialRefShape<'a, IsMut, IsPresent, IsPresent>;
// }
```

Since `IsMut::Map<'a, T>` is `&'a mut T`, every payload slot becomes a mutable reference while the
per-variant [`MapType`](../type-level/map_type.md) markers continue to track possibility. That is the whole
difference between the two borrowing accessors: one marker.

There is no rebuild counterpart, and none is needed, because the value was never taken apart, only
borrowed.

## Common Mistakes

**It takes `&mut self`, so nothing else may borrow the value** for as long as the extractor lives. That is
ordinary borrow checking, but it surfaces as an error about the companion type, which reads as though the
machinery is at fault.

**Payloads are `&mut T`, so a chain cannot move one out.** Taking ownership needs
[`HasExtractor`](./has_extractor.md).

**There is no rebuild.** `from_extractor` belongs to the owning accessor alone.

**The `where Self: 'a` clause propagates**, so a signature holding an `ExtractorMut<'a>` rarely elides
its lifetime cleanly.

**The three extractors are three different types.** Code generic over "an extractor" must pick one or be
generic over the [`MapTypeRef`](../type-level/map_type_ref.md) marker too.

**A remainder still carries none of the enum's attributes.**

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

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
