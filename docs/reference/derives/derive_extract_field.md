---
sidebar_label: '#[derive(ExtractField)]'
sidebar_position: 7
---

# `#[derive(ExtractField)]`

Extractor support for a variant.

## Overview

A `match` on an enum is checked for exhaustiveness, which is one of Rust's best properties. But it only
works where the concrete enum is named. Code that is generic over the enum cannot write a `match`, so it
falls back on a wildcard arm and an `unreachable!()`, discarding exactly the guarantee you wanted to
keep.

`#[derive(ExtractField)]` recovers it. It lets an enum be taken apart one variant at a time, and it tracks
which variants are still possible **in the type**. Each attempt either yields the payload or hands back a
*remainder* whose type has that variant ruled out:

```rust
match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
    Ok(circle) => /* it was a Circle */,
    Err(remainder) => /* it wasn't; `remainder` can no longer be a Circle */,
}
```

Keep going and the remainder narrows each time. Once every variant has been ruled out, the remainder's type
is **uninhabited** (a value of it cannot exist), and that closes the chain with no wildcard arm and no panic
path. Add a variant to the enum and the final remainder becomes inhabited again, so the code stops compiling
until the new variant is handled. The exhaustiveness check survives, for code that never names the enum.

The struct-side counterpart is [`#[derive(BuildField)]`](./derive_build_field.md), which tracks which
fields are present rather than which variants are possible. Both show what "extensible" means in
[extensible data](/docs/concepts/extensible-variants): the type carries the progress, so the compiler
checks it.

## Usage

The macro is a plain derive on an enum. It takes no arguments and has no helper attributes:

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

**This is the derive's one requirement, and it is strict.** Extraction has to name a single type per variant,
so a unit variant, a multi-field tuple variant, and a struct-style variant all fail with
`Expected variant to contain exactly one unnamed field`. There is no way to opt one variant out, so an enum
that mixes shapes cannot derive the extractor at all.

The fix is to give the payload its own struct:

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

That is idiomatic rather than a concession. The payload usually deserves to be a type, and it can derive the
extensible-data machinery itself. Note that even a fieldless case has to be given a payload struct, since a
bare `Empty` is a unit variant.

If you want the enum described structurally without deriving the extractor,
[`#[derive(HasFields)]`](./derive_has_fields.md) accepts all four variant shapes. A variantless enum is
accepted here too, and degenerates: the companion enums take no parameters at all.

The derive parses an enum, so applying it to a struct fails at parse time. The struct analogue is
[`#[derive(BuildField)]`](./derive_build_field.md).

### What it does *not* generate

Deriving this slice alone gives you deconstruction and nothing else: **no** whole-shape representation and
**no** generic constructors. Those come from [`#[derive(HasFields)]`](./derive_has_fields.md) and
[`#[derive(FromVariant)]`](./derive_from_variant.md), and the umbrella
[`#[derive(CgpData)]`](./derive_cgp_data.md) includes all three. So an enum deriving only `ExtractField` is
taken apart generically but still constructed with its ordinary `Shape::Circle(..)` constructor.

### The three extractors

The derive generates three ways in, differing only in how the payload is held.

**`to_extractor()`** consumes the value and yields payloads by value. This is the one most code uses.

**`extractor_ref()`** borrows, so each payload comes out as a shared reference and the original value is
untouched. Use it for a read-only operation over an enum you do not own.

**`extractor_mut()`** borrows mutably, so a payload can be modified in place.

All three then drive the same `extract_field` chain, and `from_extractor` converts an owned extractor back
into the enum.

## Examples

The chain reads as a sequence of attempts, each handling one variant, with the last one closed by
`finalize_extract_result`:

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

The second `extract_field` returns a `Result` whose error type is uninhabited, so
`finalize_extract_result` collapses it to the value with nothing to handle. That is not a convention. It is
the type. Try to finalize after only the first extraction and it does not compile, because `Rectangle` is
still possible.

Reading through a borrow leaves the value intact:

```rust
let radius = shape
    .extractor_ref()
    .extract_field(PhantomData::<Symbol!("Circle")>)
    .map(|circle| circle.radius)
    .ok();
```

And the mutable form modifies a payload in place:

```rust
if let Ok(circle) = shape
    .extractor_mut()
    .extract_field(PhantomData::<Symbol!("Circle")>)
{
    circle.radius = 5.0;
}
```

In practice you rarely write these chains by hand. The
[dispatch combinators](../providers/dispatch/index.md) build them for you from a set of per-variant
implementations, which is the extensible visitor pattern: a chain like the one above, generated, with one
implementation per variant chosen by wiring.

## When to use it

**Derive an extractor when independent code has to handle one variant each, or when the code doing the
matching cannot name the enum.** Those are the two cases, and outside them a `match` wins on every count.

- **Reach for it for the extensible visitor pattern.** When variants and the operations over them both need
  to grow without editing each other, this derive supports per-variant implementations plus a dispatcher.
- **Reach for it when the enum is a type parameter.** Generic code cannot `match`, so the extractor is the
  only route to an exhaustiveness guarantee.
- **Do not reach for it for a closed enum with fixed operations.** A `match` is shorter, reads better,
  already gives exhaustiveness, and generates nothing. This derive is not an improvement on `match`; you use
  it when `match` is unavailable.
- **Do not reach for it to test which variant a value holds.** `matches!` or an `if let` answers that in one
  line. The extractor's value is in the *chain* and what the chain proves.

Between this derive and its neighbours:

- **[`#[derive(CgpData)]`](./derive_cgp_data.md) or [`#[derive(CgpVariant)]`](./derive_cgp_variant.md)** if the
  enum also needs a structural representation or generic construction, which is the common case. Those
  include this output.
- **`#[derive(ExtractField)]` alone** when the enum is only ever deconstructed generically.
- **[`#[derive(FromVariant)]`](./derive_from_variant.md)** is the opposite direction, and the two are often
  wanted together.

One honest limit: the single-payload requirement is real friction. An enum whose variants carry no payload,
or several, has to be restructured before it can take this derive, and that restructuring reaches every
construction site. It is worth doing when the visitor pattern is the goal and not worth doing to satisfy a
derive you did not need.

## Under the hood

The derive centres on **two** companion enums. `__Partial{Name}` is your enum with one
[`MapType`](../traits/type-level/map_type.md) parameter added per variant and each payload wrapped in that parameter's
projection, where `IsPresent` keeps the payload and `IsVoid` maps it to the uninhabited `Void`.
`__PartialRef{Name}` adds a reserved `'__a__` lifetime and a second marker that selects a shared or a mutable
borrow of each payload. From:

```rust
#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

it emits:

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

Around them come the three entry points and the exit. `HasExtractor` yields an owned extractor with every
variant present; the borrowed pair yield the ref enum with the borrow marker fixed to `IsRef` or `IsMut`:

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

The exit is the impl the whole design turns on. It exists only for the all-`IsVoid` configuration, and
because that configuration is uninhabited its body is an empty `match`, which is why it can claim to return
*any* type:

```rust
impl FinalizeExtract for __PartialShape<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
```

Then, per variant, an `ExtractField` impl in scope only when that variant's marker is `IsPresent`. It returns
`Ok` on a match and, on a miss, an `Err` whose type has that one marker flipped to `IsVoid`:

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

Note that only the extracted variant's marker changes. The rest stay generic, which lets the extractions
happen in any order. The same impls are emitted over the ref enum, so a borrowed chain narrows identically.

`FinalizeExtract` and its companion `FinalizeExtractResult` are library traits; the derive supplies only the
all-void impl. `finalize_extract_result` calls `FinalizeExtractResult` to collapse a
`Result<T, Uninhabited>` into `T`.

**The variantless enum is genuinely special-cased**, and the reason is a Rust subtlety worth knowing. Such an
enum borrows nothing, so the ref companion is emitted as a bare empty enum with neither the lifetime nor the
borrow marker. Leaving them in would make both unused parameters. And every borrowed accessor matches
`*self` rather than `self`, because a reference is always considered inhabited even when its target is not, so
`match self {}` on a `&Self` would be rejected as non-exhaustive.

Each generated impl is aimed at the token it came from: a per-variant impl at its variant, a whole-enum impl
at the enum name. The companion enums are cloned from yours, so their tokens already carry meaningful spans.

## Common Mistakes

**Every variant must carry exactly one unnamed payload.** A unit, multi-field, or struct-style variant fails,
with no per-variant opt-out. Wrap the payload in its own struct, including for a case that carries nothing,
which still needs a payload type. [`#[derive(HasFields)]`](./derive_has_fields.md) is the derive that accepts
all four shapes.

**Absence here is `IsVoid`, not `IsNothing`.** The record side uses `IsNothing`, which maps a type to `()`
and is inhabited; a ruled-out variant uses `IsVoid`, which maps to the uninhabited `Void`. That difference
matters rather than being cosmetic: it is why a fully-narrowed remainder cannot exist and can be discharged.
An error naming the wrong marker usually means record and variant machinery have been crossed.

**Five variant names are reserved, and using one does not compile.** The generated impls write
`Self::Value`, `Self::Remainder`, `Self::Extractor`, `Self::ExtractorRef`, and `Self::ExtractorMut` to name
their associated types, so a variant with any of those names makes the path ambiguous. The compiler reports
`ambiguous associated item`, and this is the opaque one in the family: both the headline *and* the
"could refer to the variant defined here" note land on the derive, so nothing in the output names the
variant to rename. That is because these impls are generated for the companion enums rather than for
yours. Check your variant names against the five above. Renaming the variant is the fix.
[`#[derive(HasFields)]`](./derive_has_fields.md) reserves `Fields` and `FieldsRef` for the same reason, so an
enum deriving the whole family should avoid all seven.

**A remainder carries none of the enum's attributes.** Like the record side's companion, the partial enums
are generated without your derives, so a `Result<Payload, Remainder>` cannot be compared or printed as a
whole. Reach for `.ok()`, `.is_ok()`, or a `match` rather than `assert_eq!` on the result.

**`FinalizeExtractResult` is not in the prelude.** Import it from `cgp::core::field::traits` to call
`finalize_extract_result`.

**Finalizing early does not compile, and the error names the companion.** The all-void impl does not apply
while any marker is still `IsPresent`, so the compiler reports a missing method. Read the companion type in
the message to see which variants are still possible.

**The order of extraction is free but the set is not.** Each step only changes its own variant's marker, so
you may extract in any order. But you must extract *every* variant before the remainder can be finalized.

**Adding a variant breaks every hand-written chain, by design.** That is the guarantee, and it is the reason
to prefer the [dispatch combinators](../providers/dispatch/index.md), which derive the chain from the
enum's own variant list rather than repeating it at each site.

**It does not accept a struct.** The struct analogue is
[`#[derive(BuildField)]`](./derive_build_field.md).

## Related constructs

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
- [Type-level spines](../types/type_level_spines.md) — the `Either`/`Void` chain an enum's shape is built
  from.
- [Dispatch combinators](../providers/dispatch/index.md) — the providers that build an extraction chain
  for you, one implementation per variant.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — partial variants, the exhaustiveness argument,
  and the extensible visitor pattern.
- [Dispatching](/docs/concepts/dispatching) — routing a variant to the implementation that handles it.

## Source

- Entry point: [`derive_extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_extract_field.rs)
- Codegen: [`cgp_data/derive_extractor/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_extractor)
- Traits: [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)
  and [`map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_type_ref.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
