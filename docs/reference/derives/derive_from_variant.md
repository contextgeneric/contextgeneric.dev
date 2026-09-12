---
sidebar_label: '#[derive(FromVariant)]'
sidebar_position: 8
---

# `#[derive(FromVariant)]`

`#[derive(FromVariant)]` lets generic code construct an enum variant selected by a type-level tag.

## Overview

Generic code needs a trait interface to construct a variant selected by a type parameter.
`Shape::Circle(circle)` fixes both the enum and variant at the call site.

`#[derive(FromVariant)]` generates a constructor for each variant, keyed by its name as a type-level
tag:

```rust
Shape::from_variant(PhantomData::<Symbol!("Circle")>, circle)
```

The tagged call constructs the same value as `Shape::Circle(circle)`. Because the tag can be a type
parameter, a function can select a variant generically. A bound on the enum type also lets the
function work across different enums.

The derive generates only per-variant trait implementations, without companion types or state
tracking. [`ExtractField`](./derive_extract_field.md) provides the reverse operation: extracting a
payload from an enum.

## Usage

Apply `FromVariant` to an enum without arguments or helper attributes:

```rust
use cgp::prelude::*;

#[derive(FromVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

Each variant's name becomes a [`Symbol!`](../macros/symbol.md) tag and its payload type becomes the
constructor's value type. Generic parameters, lifetimes, and a `where` clause are carried onto every generated
impl.

### Every variant needs exactly one unnamed payload

Every variant must contain exactly one unnamed field, whose type becomes the constructor's payload
type. Unit, multi-field, and struct-style variants fail with
`Expected variant to contain exactly one unnamed field`. Individual variants cannot opt out.

Wrap a richer payload in a dedicated struct so the variant contains a single payload type:

```rust
pub struct Circle {
    pub radius: f64,
}

#[derive(FromVariant)]
pub enum Shape {
    Circle(Circle),
}
```

[`HasFields`](./derive_has_fields.md) accepts every variant shape when only a structural
representation and whole-value conversions are needed. `FromVariant` accepts a variantless enum but
does not emit any implementations for it.

The derive parses an enum, so applying it to a struct fails at parse time. The struct analogue, setting one
field of a value being assembled, is [`#[derive(BuildField)]`](./derive_build_field.md).

### What it does *not* generate

`FromVariant` generates constructors without a structural representation or an extractor. Derive
[`HasFields`](./derive_has_fields.md) for the representation and
[`ExtractField`](./derive_extract_field.md) for extraction, or use
[`CgpData`](./derive_cgp_data.md) for all of them.

## Examples

A function can use `FromVariant<Tag>` to stay generic over the variant it constructs:

```rust
use cgp::prelude::*;

#[derive(FromVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

fn wrap<Tag>(tag: PhantomData<Tag>, value: <Shape as FromVariant<Tag>>::Value) -> Shape
where
    Shape: FromVariant<Tag>,
{
    Shape::from_variant(tag, value)
}
```

`wrap` constructs either variant, with the payload type determined by the tag:

```rust
let circle = wrap(PhantomData::<Symbol!("Circle")>, Circle { radius: 2.0 });
let rect = wrap(PhantomData::<Symbol!("Rectangle")>, Rectangle { width: 3.0, height: 4.0 });
```

The `FromVariant<Tag>` bound connects each tag to its payload type and constructor. Without that
interface, `Shape::Circle` and `Shape::Rectangle` remain separate expressions with different argument
types.

Upcasting lets an implementation construct a small local enum and convert it into a larger one.
The implementation only needs to know the variants Each generated implementation selects its constructor through the variant-name tag:

```rust
use cgp::core::field::impls::CanUpcast;   // not in the prelude

let expr = LispSubExpr::Ident(Ident("+".to_owned())).upcast(PhantomData::<LispExpr>);
```

An upcast succeeds when every source variant has a matching variant in the target enum. The compiler
checks this correspondence, so the implementation can name only the variants it needs. See
[`CanUpcast`](../traits/casting/can_upcast.md) for the conversion and its trait requirements.

## When to use it

Use `FromVariant` when generic code must construct a variant selected by a tag. Prefer a direct
constructor when the call site already knows the variant:

- **A variant selected by a type parameter:** use `FromVariant<Tag>` to connect the tag, payload type,
  and constructor.
- **An upcast target:** generic casts use these constructors to build the target enum's variants.
- **An ordinary constructor call:** use `Shape::Circle(circle)` when the variant is known directly.
- **Construction and extraction together:** use [`CgpData`](./derive_cgp_data.md) or
  [`CgpVariant`](./derive_cgp_variant.md) to include both operations and the representation.

`FromVariant` constructs an enum from a payload, while
[`ExtractField`](./derive_extract_field.md) extracts a payload from an enum. Generic pipelines that
do both commonly derive both.

## Under the hood

The derive emits one `FromVariant` implementation per variant. For this input:

```rust
#[derive(FromVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

it produces:

```rust
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;

    fn from_variant(_tag: PhantomData<Symbol!("Circle")>, value: Self::Value) -> Self {
        Self::Circle(value)
    }
}

impl FromVariant<Symbol!("Rectangle")> for Shape {
    type Value = Rectangle;

    fn from_variant(_tag: PhantomData<Symbol!("Rectangle")>, value: Self::Value) -> Self {
        Self::Rectangle(value)
    }
}
```

Each implementation calls the ordinary variant constructor. It exposes the payload type as
`<Shape as FromVariant<Tag>>::Value`, so a generic signature can name that type without selecting a
concrete variant.

`PhantomData<Tag>` selects the trait implementation without storing a runtime tag value. Specify a
tag such as `PhantomData::<Symbol!("Circle")>` when the call would otherwise be ambiguous.

The [`FromVariant`](../traits/variant/from_variant.md) trait itself is defined in the library; the derive supplies only
these per-variant impls. Each is aimed at the variant it came from, so a conflict with a hand-written impl
underlines that variant rather than the whole `#[derive(FromVariant)]`.

## Common Mistakes

**Every variant must carry exactly one unnamed payload.** Individual variants cannot opt out.
[`HasFields`](./derive_has_fields.md) accepts all variant shapes when only a representation is needed.

**Specify the tag when the payload does not determine the variant.** In
`Shape::from_variant(PhantomData::<Symbol!("Circle")>, value)`, the type argument selects `Circle`.
The payload type alone cannot distinguish variants that share that type.

**Variants sharing a payload type have distinct tags.** A tag that does not match an implemented
variant produces an unsatisfied `FromVariant` bound.

**A variant name is a `Symbol!` of its identifier, matched exactly.** `Symbol!("Circle")` and
`Symbol!("circle")` are unrelated types, so a case slip reports as an unsatisfied `FromVariant` bound.

**A variant named `Value` does not compile.** The generated body writes `Self::Value` to name the payload
type, and a variant of that name makes the path ambiguous. The compiler reports
`ambiguous associated item` with its headline on the derive and a note pointing at the offending variant.
`Value` is the only name this derive reserves; the [extractor](./derive_extract_field.md) reserves several
more, so an enum taking both should avoid all of them. Renaming the variant is the fix.

**A variantless enum is accepted without generating implementations.** It has nothing to construct;
code requiring a `FromVariant` implementation will fail at that use site.

**It does not accept a struct.** The struct-side analogue is
[`#[derive(BuildField)]`](./derive_build_field.md).

## Related constructs

These references cover the related derives, generated traits, and supporting types:

- [`#[derive(ExtractField)]`](./derive_extract_field.md) — the reverse operation, taking a variant out
  rather than putting one in; commonly derived alongside this one.
- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella derive, which includes this slice.
- [`#[derive(HasFields)]`](./derive_has_fields.md) — the whole-shape representation, which this derive does
  not generate, and the only derive that accepts every variant shape.
- [`#[derive(BuildField)]`](./derive_build_field.md) — the struct analogue: setting one field rather than
  choosing one variant.
- [`FromVariant`](../traits/variant/from_variant.md) — the trait this generates impls of.
- [`Symbol!`](../macros/symbol.md) — the tag that names a variant.
- [`CanUpcast`](../traits/casting/can_upcast.md) — widening a smaller enum into a larger one, built on these constructors.
- [Type-level lists](../types/index.md) — the `Either`/`Void` chain the constructed variants
  correspond to.

The ideas behind it are explained on these concept pages:

- [Extensible variants](/docs/concepts/extensible-variants) — construction by name, and building through a
  small local enum before widening.

## Source

The implementation is defined in these source files:

- Entry point: [`derive_from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_from_variant.rs)
- Codegen: [`cgp_data/derive_from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_from_variant.rs)
- Trait: [`from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/from_variant.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
