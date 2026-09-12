---
sidebar_label: '#[derive(CgpVariant)]'
sidebar_position: 5
---

# `#[derive(CgpVariant)]`

`#[derive(CgpVariant)]` generates a structural representation, constructors, and an extractor for an enum.

## Overview

Generic code needs traits to construct and handle variants across different enums. A type parameter
alone does not let it name a `Circle` variant or match exhaustively on the concrete enum.

`#[derive(CgpVariant)]` supplies those traits, making the enum **extensible data**. It generates a
representation of the whole enum, a constructor per variant, and an extractor that handles variants
individually while tracking which remain possible. Generic code can use these operations without
naming the concrete enum.

`CgpVariant` accepts only enums. It produces the same output as
[`CgpData`](./derive_cgp_data.md) on an enum, but rejects a struct at parse time.

## Usage

Apply `CgpVariant` to an enum without arguments or helper attributes:

```rust
use cgp::prelude::*;

#[derive(CgpVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

Each variant is keyed by [`Symbol!`](../macros/symbol.md) of its name. Generic parameters, lifetimes,
and a `where` clause are carried onto everything generated, including the companion enums.

### Every variant needs exactly one unnamed payload

Every variant must contain exactly one unnamed field. The constructor and extractor implementations
use that field's type as the payload type. Unit, multi-field, and struct-style variants fail with
`Expected variant to contain exactly one unnamed field`. Individual variants cannot opt out.

Wrap a richer payload in its own struct so the variant contains a single payload type:

```rust
// rejected: a struct-style variant has no single payload type
pub enum Shape {
    Circle { radius: f64 },
}

// accepted: the payload is one named type
pub struct Circle {
    pub radius: f64,
}

#[derive(CgpVariant)]
pub enum Shape {
    Circle(Circle),
}
```

The payload struct can also derive [`CgpRecord`](./derive_cgp_record.md). This lets generic code
process the payload's fields after extracting it from the enum.

[`HasFields`](./derive_has_fields.md) accepts every variant shape and provides a structural
representation with whole-value conversions. Use it alone for an enum with mixed variant shapes;
that enum cannot derive the per-variant constructor or extractor.

### What it generates

The derive combines these outputs, each also available separately:

| Group | The slice on its own |
|---|---|
| The representation — the enum as a sum of named entries, with conversions | [`#[derive(HasFields)]`](./derive_has_fields.md) |
| The constructors — one per variant | [`#[derive(FromVariant)]`](./derive_from_variant.md) |
| The extractor — two partial companion enums and the impls that narrow them | [`#[derive(ExtractField)]`](./derive_extract_field.md) |

## Examples

An extraction chain checks exhaustiveness through the type of its remainder. Each attempt returns
either the payload or a remainder whose type rules out that variant:

```rust
use cgp::core::field::traits::FinalizeExtractResult;
use cgp::prelude::*;

#[derive(CgpVariant)]
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
                .finalize_extract_result();   // no variants left, so this cannot fail
            rect.width * rect.height
        }
    }
}
```

The final extraction cannot fail because its remainder type is uninhabited: a value of that type
cannot exist. [`finalize_extract_result`](../traits/variant/finalize_extract_result.md) therefore
returns the payload without a wildcard arm or `unreachable!()`. Adding another variant makes the
function fail to compile until that variant is handled. The same operations support generic code
that cannot name the enum.

## When to use it

Use `CgpVariant` when generic code needs an enum's representation, constructors, and extractor, and
the derive name should specify that the input is an enum. Choose a narrower operation when the full
set is unnecessary:

- **`CgpVariant`**: generate the full variant output and reject non-enum inputs.
- **[`CgpData`](./derive_cgp_data.md)**: generate the same output while also accepting structs.
- **A plain `match`**: handle a closed enum with fixed operations and ordinary exhaustiveness checking.
- **[`FromVariant`](./derive_from_variant.md)**: generate only the constructors.
- **[`ExtractField`](./derive_extract_field.md)**: generate only the extractor.
- **[`HasFields`](./derive_has_fields.md)**: generate only the representation and conversions. This
  derive also accepts mixed variant shapes.

The [CgpData page](./derive_cgp_data.md#when-to-use-it) explains when generic structural operations
justify the additional generated code.

## Under the hood

The derive generates representation traits, constructors, and an extractor for the input enum:

```rust
#[derive(CgpVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

The representation describes the enum as a sum of named variants:

```rust
impl HasFields for Shape {
    type Fields = Sum![
        Field<Symbol!("Circle"), Circle>,
        Field<Symbol!("Rectangle"), Rectangle>,
    ];
}

// plus HasFieldsRef, FromFields, ToFields, ToFieldsRef
```

The constructor output contains an implementation per variant, as documented for
[`#[derive(FromVariant)]`](./derive_from_variant.md):

```rust
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;

    fn from_variant(_tag: PhantomData<Symbol!("Circle")>, value: Circle) -> Self {
        Self::Circle(value)
    }
}
```

The extractor output adds `__PartialShape` for owned extraction and `__PartialRefShape` for borrowed
extraction, as documented for [`#[derive(ExtractField)]`](./derive_extract_field.md). A
[`MapType`](../traits/type-level/map_type.md) marker wraps each payload: `IsPresent` keeps the value,
and `IsVoid` maps it to the uninhabited `Void`:

```rust
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Rectangle(<__F1__ as MapType>::Map<Rectangle>),
}

impl HasExtractor for Shape {
    type Extractor = __PartialShape<IsPresent, IsPresent>;   // every variant still possible
    // ...
}

impl FinalizeExtract for __PartialShape<IsVoid, IsVoid> {    // nothing left: uninhabited
    // ...
}
```

A ruled-out variant uses `IsVoid`, while an absent record field uses `IsNothing`. `IsNothing` maps a
type to the inhabited `()`; `IsVoid` maps it to the uninhabited `Void`. Once every variant is marked
`IsVoid`, the companion enum is uninhabited, so an empty match on it can return any type.

The companion names are reserved: `__Partial{Name}` and `__PartialRef{Name}`. They keep the original
type's visibility. Each generated impl is aimed at the token it came from, so a conflict with a
hand-written impl underlines that variant rather than the whole derive.

## Common Mistakes

**Every variant needs exactly one unnamed payload.** Individual variants cannot opt out.
[`HasFields`](./derive_has_fields.md) accepts all variant shapes when only the representation and
whole-value conversions are needed.

**Seven variant names are reserved.** Because the generated impls name their associated types through
`Self::…`, a variant called `Fields`, `FieldsRef`, `Value`, `Remainder`, `Extractor`, `ExtractorRef`, or
`ExtractorMut` makes that path ambiguous and the derive fails with `ambiguous associated item`, headlined
at the derive. Whether the message also names the variant depends on which part collided: the
representation and constructor impls are generated for your enum and point a note at the real variant,
while the extractor's are generated for the companion enums and point back at the derive. Renaming the
variant is the fix. Record field names are unaffected, since a field is not in the same namespace as an
associated type.

**The companion enums do not inherit your attributes.** Deriving `Debug` or `PartialEq` on the
original enum does not implement them for the extraction remainder. Use `.ok()`, `.is_ok()`, or a
`match` to inspect an extraction result instead of comparing the whole result with `assert_eq!`.

**Record and variant absence use different markers.** Use `IsVoid` for a ruled-out variant and
`IsNothing` for a missing record field. An error naming the wrong marker usually means record and
variant operations have been mixed.

**A variantless enum is accepted.** It produces empty companion enums without field-state parameters.

**`CgpVariant` rejects structs.** Use [`CgpRecord`](./derive_cgp_record.md) or
[`CgpData`](./derive_cgp_data.md) for a struct.

## Related constructs

These references cover the related derives, generated traits, and supporting types:

- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella, which dispatches here for an enum.
- [`#[derive(CgpRecord)]`](./derive_cgp_record.md) — the struct face.
- [`#[derive(HasFields)]`](./derive_has_fields.md), [`#[derive(FromVariant)]`](./derive_from_variant.md),
  and [`#[derive(ExtractField)]`](./derive_extract_field.md) — the three slices this emits.
- [`ExtractField`](../traits/variant/extract_field.md) — the extractor family it generates impls for.
- [`MapType`](../traits/type-level/map_type.md) — the `IsPresent`/`IsVoid` markers the companions are parameterized
  by.
- [`CanUpcast`](../traits/casting/can_upcast.md) and [`CanDowncast`](../traits/casting/can_downcast.md) — converting
  between two enums whose variants overlap, built on this machinery.
- [Dispatch combinators](../providers/dispatch/index.md) — the providers that route a variant to
  the implementation handling it.

The ideas behind it are explained on these concept pages:

- [Extensible variants](/docs/concepts/extensible-variants) — the enum half, and the extensible visitor
  pattern.
- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to a handler per variant.

## Source

The implementation is defined in these source files:

- Entry point: [`cgp_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_variant.rs)
- Variant codegen: [`cgp_data/variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/variant.rs)
- Runtime traits: [`cgp-field/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-field/src/traits)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
