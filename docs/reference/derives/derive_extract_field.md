---
sidebar_label: '#[derive(ExtractField)]'
sidebar_position: 7
---

# `#[derive(ExtractField)]`

`#[derive(ExtractField)]` generates an enum extractor that tracks remaining variants in its type.

## Overview

Generic code cannot match directly on an enum whose concrete type it does not know. It needs a trait
interface that preserves Rust's exhaustiveness checking while handling variants individually.

`#[derive(ExtractField)]` provides that interface by tracking the remaining variants in the
extractor's type. Each extraction either returns the payload or returns a **remainder** whose type
rules out that variant:

```rust
match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
    Ok(circle) => /* it was a Circle */,
    Err(remainder) => /* it wasn't; `remainder` can no longer be a Circle */,
}
```

The chain is exhaustive when every variant has been ruled out. At that point, the remainder is
**uninhabited**, meaning a value of its type cannot exist, so the chain can finish without a wildcard
arm or a panic. Adding a variant makes the final remainder inhabited again and prevents finalization
until the new variant is handled.

[`BuildField`](./derive_build_field.md) provides the corresponding operation for structs: it tracks
which fields are present during construction. Both use types to record progress so the compiler can
check completeness.

## Usage

Apply `ExtractField` to an enum without arguments or helper attributes:

```rust
use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

Each variant's name becomes a [`Symbol!`](../macros/symbol.md) tag and its payload type becomes the value
that tag yields. Generic parameters, lifetimes, and a `where` clause are carried onto both companion types and
every generated impl.

### Every variant needs exactly one unnamed payload

Every variant must contain exactly one unnamed field. The extractor uses that field's type as its
payload type. Unit, multi-field, and struct-style variants fail with
`Expected variant to contain exactly one unnamed field`. Individual variants cannot opt out.

Wrap the payload in a dedicated struct to give each variant a single payload type:

```rust
// rejected: no single payload type
pub enum Shape {
    Circle { radius: f64 },
    Empty,
}

// accepted
pub struct Circle {
    pub radius: f64,
}

pub struct Empty;

#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Empty(Empty),
}
```

Payload structs can derive extensible-data operations themselves. A fieldless case also needs a
payload type, such as the unit struct `Empty` above; a bare `Empty` variant does not meet the
single-field requirement.

[`HasFields`](./derive_has_fields.md) accepts every variant shape when only a structural
representation and whole-value conversions are needed. `ExtractField` also accepts a variantless
enum; its companion enums do not need field-state parameters.

The derive parses an enum, so applying it to a struct fails at parse time. The struct analogue is
[`#[derive(BuildField)]`](./derive_build_field.md).

### What it does *not* generate

`ExtractField` generates extraction support without a structural representation or generic
constructors. Derive [`HasFields`](./derive_has_fields.md) for the representation and
[`FromVariant`](./derive_from_variant.md) for constructors, or use
[`CgpData`](./derive_cgp_data.md) for all of them. An enum deriving only `ExtractField` still uses
ordinary constructors such as `Shape::Circle(..)`.

### The three extractors

Choose an extractor according to how the operation needs to access the payload:

- **`to_extractor()`**: consume the enum and return payloads by value.
- **`extractor_ref()`**: borrow the enum and return shared references to payloads.
- **`extractor_mut()`**: borrow the enum mutably so payloads can be modified in place.

Each form supports the same `extract_field` chain. `from_extractor` converts an all-present owned
extractor back into the original enum.

## Examples

`finalize_extract_result` returns the last payload once the chain has ruled out every other variant:

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
                .finalize_extract_result();   // remainder is now empty; this cannot fail
            rect.width * rect.height
        }
    }
}
```

The final `extract_field` returns a `Result` with an uninhabited error type, so
`finalize_extract_result` can return its value directly. Finalizing after only the `Circle` attempt
fails to compile because `Rectangle` is still possible.

`extractor_ref()` lets the operation read a payload while preserving the original value:

```rust
let radius = shape
    .extractor_ref()
    .extract_field(PhantomData::<Symbol!("Circle")>)
    .map(|circle| circle.radius)
    .ok();
```

`extractor_mut()` lets the operation modify a payload in place:

```rust
if let Ok(circle) = shape
    .extractor_mut()
    .extract_field(PhantomData::<Symbol!("Circle")>)
{
    circle.radius = 5.0;
}
```

The [dispatch combinators](../providers/dispatch/index.md) generate extraction chains from
per-variant implementations. This supports the extensible visitor pattern: each implementation
handles a variant, and wiring selects which implementation to use.

## When to use it

Use `ExtractField` when independent implementations handle individual variants or when generic code
cannot name the enum. Prefer ordinary matching when the concrete enum is available:

- **The extensible visitor pattern:** combine independent per-variant implementations through a
  dispatcher when variants and operations need to grow separately.
- **A generic enum type:** use the extractor's trait interface to check exhaustiveness without naming
  the concrete variants in a `match`.
- **A closed enum with fixed operations:** use a `match`, which already checks exhaustiveness.
- **A single variant test:** use `matches!` or `if let` when a complete extraction chain is unnecessary.

Choose the derive according to the operations the enum needs:

- **[`CgpData`](./derive_cgp_data.md) or [`CgpVariant`](./derive_cgp_variant.md)**: include the
  representation and generic constructors alongside extraction.
- **`ExtractField` alone**: provide only generic extraction.
- **[`FromVariant`](./derive_from_variant.md)**: provide generic construction, often needed alongside
  extraction.

The single-payload requirement can require changes at every construction site. An enum with unit,
multi-field, or struct-style variants must be restructured before it can derive the extractor.
Consider that cost when deciding whether independent variant handling is needed.

## Under the hood

The derive generates an owned companion enum and a borrowed companion enum. `__Partial{Name}` adds
one [`MapType`](../traits/type-level/map_type.md) parameter per variant: `IsPresent` preserves the
payload and `IsVoid` replaces it with the uninhabited `Void`. `__PartialRef{Name}` also adds the
reserved lifetime `'__a__` and a marker selecting shared or mutable references. For this input:

```rust
#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

The companion enums wrap each payload in the corresponding marker projections:

```rust
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Rectangle(<__F1__ as MapType>::Map<Rectangle>),
}

pub enum __PartialRefShape<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Circle>>),
    Rectangle(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Rectangle>>),
}
```

`HasExtractor` supplies an owned extractor with every variant still possible. `HasExtractorRef` and
`HasExtractorMut` use the borrowed companion with its borrow marker set to `IsRef` or `IsMut`:

```rust
impl HasExtractor for Shape {
    type Extractor = __PartialShape<IsPresent, IsPresent>;   // every variant still possible
    // to_extractor / from_extractor map each concrete variant across, and back
}

impl HasExtractorRef for Shape {
    type ExtractorRef<'__a__> = __PartialRefShape<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    // ...
}

// plus HasExtractorMut over __PartialRefShape<'__a__, IsMut, IsPresent, IsPresent>
```

`FinalizeExtract` is implemented only for the all-`IsVoid` configuration. That configuration is
uninhabited, so its implementation can return any type through an empty match:

```rust
impl FinalizeExtract for __PartialShape<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
```

Each variant gets an `ExtractField` implementation available when its marker is `IsPresent`.
A match returns `Ok(payload)`; a miss returns `Err(remainder)` with that marker changed to `IsVoid`:

```rust
impl<__F1__: MapType> ExtractField<Symbol!("Circle")> for __PartialShape<IsPresent, __F1__> {
    type Value = Circle;
    type Remainder = __PartialShape<IsVoid, __F1__>;   // Circle ruled out; __F1__ untouched

    fn extract_field(self, _: PhantomData<Symbol!("Circle")>) -> Result<Circle, Self::Remainder> {
        match self {
            __PartialShape::Circle(value) => Ok(value),
            __PartialShape::Rectangle(value) => Err(__PartialShape::Rectangle(value)),
        }
    }
}
```

Extractions can run in any order because each step changes only its own variant's marker. The other
markers remain generic. The borrowed companion receives equivalent implementations, so borrowed
extraction narrows the remainder in the same way.

`FinalizeExtract` and its companion `FinalizeExtractResult` are library traits; the derive supplies only the
all-void impl. `finalize_extract_result` calls `FinalizeExtractResult` to collapse a
`Result<T, Uninhabited>` into `T`.

A variantless enum produces an empty borrowed companion without a lifetime or borrow marker. It
borrows nothing, so those parameters would be unused. Its borrowed accessors match `*self` rather
than `self`: Rust considers a reference inhabited even when its target is uninhabited, so an empty
`match self {}` on `&Self` would fail the exhaustiveness check.

Each generated impl is aimed at the token it came from: a per-variant impl at its variant, a whole-enum impl
at the enum name. The companion enums are cloned from yours, so their tokens already carry meaningful spans.

## Common Mistakes

**Every variant must carry exactly one unnamed payload.** Individual variants cannot opt out. Wrap
richer payloads in structs, and give a fieldless case a payload type too.
[`HasFields`](./derive_has_fields.md) accepts all variant shapes if only a representation is needed.

**A ruled-out variant uses `IsVoid`.** It maps the payload to uninhabited `Void`, allowing a fully
narrowed remainder to be finalized. The record marker `IsNothing` maps to inhabited `()`, so it cannot
serve that purpose. An error naming `IsNothing` usually means record and variant operations have
been mixed.

**Reserved variant names make generated associated-type paths ambiguous.** Avoid `Value`, `Remainder`,
`Extractor`, `ExtractorRef`, and `ExtractorMut`. The generated implementations use those names through
`Self::…`, which conflicts with identically named variants and produces `ambiguous associated item`.

Extractor diagnostics may leave the offending variant unnamed. The implementations target generated
companion enums, so both the headline and the note can point to the derive. Check the names above and
rename the conflicting variant. [`HasFields`](./derive_has_fields.md) also reserves `Fields` and
`FieldsRef`, so an enum deriving the full family must avoid those too.

**The remainder does not inherit the enum's attributes.** Deriving `Debug` or `PartialEq` on the
enum does not implement them for the companion. Use `.ok()`, `.is_ok()`, or a `match` to inspect the
result instead of printing or comparing the whole `Result<Payload, Remainder>`.

**`FinalizeExtractResult` is not in the prelude.** Import it from `cgp::core::field::traits` to call
`finalize_extract_result`.

**Finalizing early does not compile, and the error names the companion.** The all-void impl does not apply
while any marker is still `IsPresent`, so the compiler reports a missing method. Read the companion type in
the message to see which variants are still possible.

**Every variant must be ruled out before finalizing the remainder.** Extraction order is unrestricted
because each step changes only its own variant's marker.

**Adding a variant makes an existing exhaustive chain incomplete.** Update hand-written chains to
handle the new variant, or use [dispatch combinators](../providers/dispatch/index.md) to generate
chains from the enum's variant list.

**It does not accept a struct.** The struct analogue is
[`#[derive(BuildField)]`](./derive_build_field.md).

## Related constructs

These references cover the related derives, generated traits, and supporting types:

- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella derive, which includes this slice.
- [`#[derive(FromVariant)]`](./derive_from_variant.md) — the opposite direction, generic construction, which
  this derive does not generate.
- [`#[derive(HasFields)]`](./derive_has_fields.md) — the whole-shape representation, which this derive does
  not generate either, and the one derive that accepts every variant shape.
- [`#[derive(BuildField)]`](./derive_build_field.md) — the struct analogue: incremental construction rather
  than deconstruction.
- [`ExtractField`](../traits/variant/extract_field.md) — the extractor trait family, including `HasExtractor`,
  `FinalizeExtract`, and `FinalizeExtractResult`.
- [`MapType`](../traits/type-level/map_type.md) — the `IsPresent`/`IsVoid` markers the companion enums are parameterized
  by.
- [`CanUpcast`](../traits/casting/can_upcast.md) — upcasting and downcasting between enums, which reuse this recursion.
- [Type-level lists](../types/index.md) — the `Either`/`Void` chain an enum's shape is built
  from.
- [Dispatch combinators](../providers/dispatch/index.md) — the providers that build an extraction chain
  for you, one implementation per variant.

The ideas behind it are explained on these concept pages:

- [Extensible variants](/docs/concepts/extensible-variants) — partial variants, the exhaustiveness argument,
  and the extensible visitor pattern.
- [Dispatching](/docs/concepts/dispatching) — routing a variant to the implementation that handles it.

## Source

The implementation is defined in these source files:

- Entry point: [`derive_extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_extract_field.rs)
- Codegen: [`cgp_data/derive_extractor/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_extractor)
- Traits: [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)
  and [`map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_type_ref.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
