---
sidebar_label: 'HasExtractor'
---

# `HasExtractor`

Turning an enum into an extractor, and back.

## Overview

[`ExtractField`](./extract_field.md) narrows an *extractor*, not an enum. `HasExtractor` is where one
comes from:

```rust
pub trait HasExtractor {
    type Extractor;

    fn to_extractor(self) -> Self::Extractor;
    fn from_extractor(extractor: Self::Extractor) -> Self;
}
```

`to_extractor` **consumes** the value and yields an extractor whose payloads are owned, which is what a
chain needs when it will take a payload away. `from_extractor` reverses it, rebuilding the enum from an
extractor that was not narrowed — useful when a routine inspects a value and hands it back unchanged.

It is the owning member of a group of three, and choosing between them is most of using the extractor
family:

| | payloads come out as | the original |
|---|---|---|
| `HasExtractor` | owned | consumed |
| [`HasExtractorRef`](./has_extractor_ref.md) | `&T` | survives |
| [`HasExtractorMut`](./has_extractor_mut.md) | `&mut T` | survives, mutably borrowed |

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

`Extractor` names the partial companion enum at its all-possible configuration — every variant still
available. That is the type a chain starts from, and the type that appears in an error when a chain is
finalized too early.

The impls come from [`#[derive(ExtractField)]`](../derives/derive_extract_field.md), also available
through [`#[derive(CgpVariant)]`](../derives/derive_cgp_variant.md) and
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md).

## Examples

Starting a chain, which is what `to_extractor` is for:

```rust
use cgp::core::field::traits::FinalizeExtractResult;
use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

fn area(shape: Shape) -> f64 {
    match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
        Ok(circle) => core::f64::consts::PI * circle.radius * circle.radius,
        Err(remainder) => {
            let rect = remainder
                .extract_field(PhantomData::<Symbol!("Rectangle")>)
                .finalize_extract_result();
            rect.width * rect.height
        }
    }
}
```

`from_extractor` is the less common half, and it round-trips an un-narrowed extractor:

```rust
let extractor = shape.to_extractor();
// ... inspect without extracting ...
let shape = Shape::from_extractor(extractor);
```

Note that this only works on an extractor still at the all-possible configuration. Once a variant has
been ruled out the type has changed, and there is no way back.

## When to reach for it, and when not

**Reach for `to_extractor` only when the chain will take payloads by value**, and prefer a borrowing
accessor otherwise — the weakest that works is the right one.

- **[`HasExtractorRef`](./has_extractor_ref.md)** for a read-only operation over a value you do not own.
  This is the common case, and requiring ownership where a borrow would do forces callers to clone or
  give up their value.
- **[`HasExtractorMut`](./has_extractor_mut.md)** to change a payload in place.
- **`HasExtractor`** when the payload must be moved out — returned, stored, or handed on by value.
- **A `match`** when the enum is concrete. The family is for code that cannot name it.
- **The [dispatch combinators](../providers/dispatch_combinators.md)** rather than a hand-written chain,
  since they derive it from the enum's variant list.

## Under the hood

:::note

### Advanced

This section shows what `Extractor` is.

:::

The derive generates a companion enum with one [`MapType`](./map_type.md) parameter per variant, and this
trait fixes the starting configuration to all-possible:

```rust
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Rectangle(<__F1__ as MapType>::Map<Rectangle>),
}

impl HasExtractor for Shape {
    type Extractor = __PartialShape<IsPresent, IsPresent>;   // every variant still possible
    // ...
}
```

Because `IsPresent::Map<T>` is `T`, the all-possible companion holds exactly the enum's own payloads, so
`to_extractor` is a variant-for-variant move rather than a wrapping, and `from_extractor` is the same
move back.

The companion also implements [`PartialData`](./partial_data.md), with `Target` naming the original enum
— which is how `from_extractor` knows what to rebuild, and the one trait the builder and extractor
families share directly.

Narrowing from here is [`ExtractField`](./extract_field.md), and the chain ends at
[`FinalizeExtract`](./finalize_extract.md).

## Gotchas

**It consumes the value.** Reach for [`HasExtractorRef`](./has_extractor_ref.md) when the original must
survive. This is the commonest over-requirement in the family.

**`from_extractor` only accepts an un-narrowed extractor.** Once a variant is ruled out the type is
different and there is no rebuild — which is correct, since the value may no longer be representable.

**The extractor carries none of the enum's attributes.** The companion is generated without your derives,
so it is neither `Debug` nor `PartialEq` however the enum is derived.

**`Extractor` is not the enum.** A signature that returns `Self::Extractor` is returning a companion
type, and naming it in a public API exposes a generated name.

**Every variant must carry exactly one unnamed payload** for the derive to apply at all — the family's
one real restriction, covered on the
[derive's page](../derives/derive_extract_field.md).

## Related constructs

- [`ExtractField`](./extract_field.md) — what narrows the extractor this produces.
- [`HasExtractorRef`](./has_extractor_ref.md) and [`HasExtractorMut`](./has_extractor_mut.md) — the two
  borrowing accessors.
- [`FinalizeExtract`](./finalize_extract.md) and
  [`FinalizeExtractResult`](./finalize_extract_result.md) — how a chain ends.
- [`FromVariant`](./from_variant.md) — constructing an enum from one named variant.
- [`PartialData`](./partial_data.md) — what names the enum a companion belongs to.
- [`MapType`](./map_type.md) — the `IsPresent`/`IsVoid` markers the companion is parameterized by.
- [`CanUpcast`](./can_upcast.md) and [`CanDowncast`](./can_downcast.md) — casts built on this recursion.
- [`#[derive(ExtractField)]`](../derives/derive_extract_field.md) — generates the companion and this
  impl.
- [`HasBuilder`](./has_builder.md) — the struct analogue.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — partial variants and the extensible visitor
  pattern.

## Source

- [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)
  — `HasExtractor` and the rest of the family
- [`partial_data.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/partial_data.rs)
  — `PartialData`, which the companions also implement

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
