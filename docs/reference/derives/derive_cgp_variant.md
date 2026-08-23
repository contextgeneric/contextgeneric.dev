---
sidebar_label: '#[derive(CgpVariant)]'
sidebar_position: 5
---

# `#[derive(CgpVariant)]`

The extensible-data derive for an enum.

## Overview

A plain Rust enum is opaque to generic code. There is no way to refer to "the `Circle` variant" through
a type parameter, so anything that must work across several enums ends up written once per enum, and
the `match` that would make it exhaustive can only be written where the concrete enum is named.

`#[derive(CgpVariant)]` turns an enum into **extensible data**: a type whose variants generic code can
name, construct, and take apart without ever mentioning the concrete type. It produces the whole
variant half of the family in one line: the whole-shape variant list, a constructor per variant, and an
incremental extractor that removes variants one at a time while tracking which are still possible.

It is the enum-only face of [`#[derive(CgpData)]`](./derive_cgp_data.md). The two run the same code and
emit the same output on an enum. The difference is that this one **rejects a struct at parse time**.

## Usage

The derive takes no arguments and has no helper attributes:

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

**This is the family's one real restriction.** It comes from the two slices that take an enum apart:
each has to name a single type per variant, and a unit, multi-field, or struct-style variant gives it
none or several. Such a variant fails with `Expected variant to contain exactly one unnamed field`, and
there is **no way to opt one variant out**. An enum that mixes shapes cannot take this derive at all.

The fix is to wrap the richer payload in its own struct, so the variant's value stays a single nameable
type:

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

That is idiomatic rather than a workaround. The payload struct usually wants to be a type in its own
right, and it can derive [`#[derive(CgpRecord)]`](./derive_cgp_record.md) too, which is how a nested
shape becomes reachable.

One exception is worth knowing: [`#[derive(HasFields)]`](./derive_has_fields.md) accepts **all four**
variant shapes, because it only describes a variant rather than deconstructing it. So an enum with mixed
variants can still have a structural representation, but not the generic constructor or the extractor.

### What it generates

Three groups, each of which is also available as a derive of its own:

| Group | The slice on its own |
|---|---|
| The representation — the enum as a sum of named entries, with conversions | [`#[derive(HasFields)]`](./derive_has_fields.md) |
| The constructors — one per variant | [`#[derive(FromVariant)]`](./derive_from_variant.md) |
| The extractor — two partial companion enums and the impls that narrow them | [`#[derive(ExtractField)]`](./derive_extract_field.md) |

## Examples

The variant machinery is most useful for matching generically with a guaranteed-exhaustive end. Each
extraction either yields the payload or hands back a *remainder* whose type has that variant ruled out:

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

There is no wildcard arm and no `unreachable!()`. After the second extraction the remainder's type has
both variants ruled out, which makes it uninhabited, and
[`finalize_extract_result`](../traits/finalize_extract_result.md) discharges a value that cannot exist.
Add a third variant to `Shape` and this function stops compiling until it is handled, which is the same
guarantee a concrete `match` gives, recovered for code that never names the enum.

## When to reach for it, and when not

**Reach for it when generic code has to work over the enum's own structure**, and prefer
[`#[derive(CgpData)]`](./derive_cgp_data.md) unless naming the shape earns its keep as documentation.

- **Use `#[derive(CgpVariant)]`** to state that a type is always an enum, and to have a later change to
  a struct fail at the derive.
- **Use [`#[derive(CgpData)]`](./derive_cgp_data.md)** as the default.
- **Prefer a `match`** for a closed enum with fixed operations. It is clearer than any machinery and it
  already gives exhaustiveness. Reach for the variant derives when the variant set is open, or when
  independent modules must each contribute a variant or an operation over one.
- **Derive the slice you want**: [`FromVariant`](./derive_from_variant.md) alone if code only needs to
  *construct*, [`ExtractField`](./derive_extract_field.md) alone if it only needs to deconstruct, and
  [`HasFields`](./derive_has_fields.md) alone for the shape, which is the only one of the three that
  accepts every variant shape.

The full argument for when a type earns the extensible-data machinery at all is on the
[umbrella page](./derive_cgp_data.md#when-to-reach-for-it-and-when-not).

## Under the hood

The derive emits three groups in order. From:

```rust
#[derive(CgpVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

it first emits the **representation**, over a sum rather than a product:

```rust
impl HasFields for Shape {
    type Fields = Sum![
        Field<Symbol!("Circle"), Circle>,
        Field<Symbol!("Rectangle"), Rectangle>,
    ];
}

// plus HasFieldsRef, FromFields, ToFields, ToFieldsRef
```

Then the **constructors**, one per variant, which is
[`#[derive(FromVariant)]`](./derive_from_variant.md)'s output:

```rust
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;

    fn from_variant(_tag: PhantomData<Symbol!("Circle")>, value: Circle) -> Self {
        Self::Circle(value)
    }
}
```

Then the **extractor**, which is [`#[derive(ExtractField)]`](./derive_extract_field.md)'s output and
where two companion enums appear: `__PartialShape` for owned extraction and `__PartialRefShape` for
borrowed. A variant's payload is wrapped in a [`MapType`](../traits/map_type.md) marker that is either
`IsPresent` or `IsVoid`, the latter mapping it to the uninhabited `Void`:

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

**Note the asymmetry with the record side, which is the detail most worth carrying away: a record uses
`IsNothing` for a field that is not there, and a variant uses `IsVoid` for one that has been ruled out.**
`IsNothing` maps a type to `()`, which is inhabited; `IsVoid` maps it to `Void`, which is not. That is
exactly why extraction ends the way it does: once every variant is `IsVoid` the whole companion enum is
uninhabited, so a value of it cannot exist and can be discharged to produce anything.

The companion names are reserved: `__Partial{Name}` and `__PartialRef{Name}`. They keep the original
type's visibility. Each generated impl is aimed at the token it came from, so a conflict with a
hand-written impl underlines that variant rather than the whole derive.

## Common Mistakes

**Every variant needs exactly one unnamed payload**, with no per-variant opt-out.
[`#[derive(HasFields)]`](./derive_has_fields.md) is the one derive in the family that accepts all four
shapes, so an enum with mixed variants gets that derive instead.

**Seven variant names are reserved.** Because the generated impls name their associated types through
`Self::…`, a variant called `Fields`, `FieldsRef`, `Value`, `Remainder`, `Extractor`, `ExtractorRef`, or
`ExtractorMut` makes that path ambiguous and the derive fails with `ambiguous associated item`, headlined
at the derive. Whether the message also names the variant depends on which part collided: the
representation and constructor impls are generated for your enum and point a note at the real variant,
while the extractor's are generated for the companion enums and point back at the derive. Renaming the
variant is the fix. Record field names are unaffected, since a field is not in the same namespace as an
associated type.

**The companion enums carry none of your attributes.** The derive clears them, so an extraction
remainder is neither `Debug` nor `PartialEq` however the enum is derived. `assert_eq!` over a
`Result<Payload, Remainder>` does not compile. Reach for `.ok()`, `.is_ok()`, or a `match`.

**Absence is spelled differently on the two sides.** `IsVoid` for a ruled-out variant, `IsNothing` for a
missing record field, and they are not interchangeable. An error naming the wrong one usually means
record and variant machinery have been crossed.

**A variantless enum compiles and does nothing useful.** It yields bare companion enums.

**A struct is rejected**, which is the point of the name. Reach for
[`#[derive(CgpRecord)]`](./derive_cgp_record.md) or the umbrella.

## Related constructs

- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella, which dispatches here for an enum.
- [`#[derive(CgpRecord)]`](./derive_cgp_record.md) — the struct face.
- [`#[derive(HasFields)]`](./derive_has_fields.md), [`#[derive(FromVariant)]`](./derive_from_variant.md),
  and [`#[derive(ExtractField)]`](./derive_extract_field.md) — the three slices this emits.
- [`ExtractField`](../traits/extract_field.md) — the extractor family it generates impls for.
- [`MapType`](../traits/map_type.md) — the `IsPresent`/`IsVoid` markers the companions are parameterized
  by.
- [`CanUpcast`](../traits/can_upcast.md) and [`CanDowncast`](../traits/can_downcast.md) — converting
  between two enums whose variants overlap, built on this machinery.
- [Dispatch combinators](../providers/dispatch_combinators.md) — the providers that route a variant to
  the implementation handling it.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — the enum half, and the extensible visitor
  pattern.
- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to a handler per variant.

## Source

- Entry point: [`cgp_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_variant.rs)
- Variant codegen: [`cgp_data/variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/variant.rs)
- Runtime traits: [`cgp-field/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-field/src/traits)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
