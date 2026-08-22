---
sidebar_label: '#[derive(FromVariant)]'
---

# `#[derive(FromVariant)]`

Generic construction of an enum from one of its named variants.

## Overview

`Shape::Circle(circle)` names two things: the enum, and the variant. That is fine at a concrete call site and
useless to code that knows neither — a routine that has been handed a value and told which variant to wrap it
in cannot write that expression.

`#[derive(FromVariant)]` gives it a way to. The derive adds one constructor per variant, addressed by the
variant's *name as a type*, so the choice of variant becomes a parameter rather than syntax:

```rust
Shape::from_variant(PhantomData::<Symbol!("Circle")>, circle)
```

That call is equivalent to `Shape::Circle(circle)`. The difference is that the tag can come from a type
parameter, so one function can build whichever variant it was asked for, of whichever enum.

**This is the simplest derive in the extensible-data family.** It generates no companion type, no state
tracking, and no traits of its own — just a constructor per variant. It is the counterpart to
[`#[derive(ExtractField)]`](./derive_extract_field.md), which takes a variant out; this one puts a variant in.

## Usage

The macro is a plain derive on an enum. It takes no arguments and has no helper attributes:

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

**The derive shares the extractor's one requirement.** A constructor has to take a single value, so a unit
variant, a multi-field tuple variant, and a struct-style variant all fail with
`Expected variant to contain exactly one unnamed field`. There is no per-variant opt-out, so an enum that
mixes shapes cannot take this derive.

Wrapping the payload in its own struct is the fix, and it is what idiomatic CGP enums do anyway:

```rust
pub struct Circle {
    pub radius: f64,
}

#[derive(FromVariant)]
pub enum Shape {
    Circle(Circle),
}
```

If the enum only needs to be *described* rather than constructed,
[`#[derive(HasFields)]`](./derive_has_fields.md) accepts all four variant shapes instead. A variantless enum
is accepted here and simply produces no impls.

The derive parses an enum, so applying it to a struct fails at parse time. The struct analogue — setting one
field of a value being assembled — is [`#[derive(BuildField)]`](./derive_build_field.md).

### What it does *not* generate

Deriving this slice alone gives you construction and nothing else: **no** whole-shape representation and
**no** extractor. Those come from [`#[derive(HasFields)]`](./derive_has_fields.md) and
[`#[derive(ExtractField)]`](./derive_extract_field.md), and the umbrella
[`#[derive(CgpData)]`](./derive_cgp_data.md) includes all three.

## Examples

The capability is a function that stays generic over the variant it builds:

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

One `wrap` builds either variant, and its payload type is *derived from the tag* rather than fixed:

```rust
let circle = wrap(PhantomData::<Symbol!("Circle")>, Circle { radius: 2.0 });
let rect = wrap(PhantomData::<Symbol!("Rectangle")>, Rectangle { width: 3.0, height: 4.0 });
```

No hand-written function can do that, because `Shape::Circle` and `Shape::Rectangle` are different
expressions taking different types.

Where this earns its keep in practice is building a value using only the variants an implementation knows
about, then widening it. A routine that only produces two of a large enum's variants declares a small local
enum, constructs into that, and lifts the result into the full type with an upcast:

```rust
use cgp::core::field::impls::CanUpcast;   // not in the prelude

let expr = LispSubExpr::Ident(Ident("+".to_owned())).upcast(PhantomData::<LispExpr>);
```

The upcast always succeeds, because every variant of the smaller enum has a home in the larger one. That is
the construction-side counterpart of reading a field through a getter: the implementation names only what it
needs, and the widening is checked. Upcasting is documented with the other
[structural casts](../traits/can_upcast.md), and it is built on the same per-variant machinery as this derive.

## When to reach for it, and when not

**Derive it when code that does not name a variant has to construct one.** That is the whole test, and it is
a narrower need than deconstruction — most code decides which variant to build at a site that can just name
it.

- **Reach for it when the variant is chosen by a type parameter.** A routine parameterized over the variant
  it produces has no other option.
- **Reach for it to make a smaller enum upcastable into a larger one.** Casting between enums is built on
  these constructors, so this derive is what lets an implementation work in a narrow local enum and widen the
  result.
- **Do not reach for it for an ordinary constructor call.** `Shape::Circle(circle)` is shorter, clearer, and
  generates nothing. This derive adds a *second* way to do the same thing, for callers that cannot use the
  first.
- **Do not derive it alone if you also want to take the enum apart**, which is the usual case — reach for
  [`#[derive(CgpData)]`](./derive_cgp_data.md) or [`#[derive(CgpVariant)]`](./derive_cgp_data.md), which
  bundle construction, deconstruction, and the representation.

Between the constructor and the extractor the split is exactly what the names say: this derive puts a value
into an enum, [`#[derive(ExtractField)]`](./derive_extract_field.md) gets one out, and they are commonly
derived together because a generic pipeline usually does both.

## Under the hood

The derive emits **one impl per variant and nothing else** — no companion type, no markers, no state. From:

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

Each body is the plain constructor call. What the impl adds is that the *choice* between them is now a type
argument, and that the payload type is reachable as `<Shape as FromVariant<Tag>>::Value` — which is what lets
a generic signature name it without knowing which variant is in play.

The `PhantomData<Tag>` parameter carries no value. It exists so a call site can say which impl it means when
several are in scope, which is why the tag is passed as `PhantomData::<Symbol!("Circle")>` rather than
inferred.

The [`FromVariant`](../traits/from_variant.md) trait itself is defined in the library; the derive supplies only
these per-variant impls. Each is aimed at the variant it came from, so a conflict with a hand-written impl
underlines that variant rather than the whole `#[derive(FromVariant)]`.

## Common Mistakes

**Every variant must carry exactly one unnamed payload**, the same requirement the extractor has, with no
per-variant opt-out. [`#[derive(HasFields)]`](./derive_has_fields.md) is the derive that accepts all four
shapes.

**The tag has to be written out, not inferred.** `Shape::from_variant(PhantomData::<Symbol!("Circle")>, value)`
needs the turbofish, because nothing in the value determines which variant to build when two variants could
share a payload type.

**Two variants with the same payload type are still distinguishable, and only by the tag.** That is the reason
for the previous point, and it means a mistyped tag is a missing-impl error rather than a type mismatch.

**A variant name is a `Symbol!` of its identifier, matched exactly.** `Symbol!("Circle")` and
`Symbol!("circle")` are unrelated types, so a case slip reports as an unsatisfied `FromVariant` bound.

**A variant named `Value` does not compile.** The generated body writes `Self::Value` to name the payload
type, and a variant of that name makes the path ambiguous — the compiler reports
`ambiguous associated item` with its headline on the derive and a note pointing at the offending variant. `Value` is the only name
this derive reserves; the [extractor](./derive_extract_field.md) reserves several more, so an enum taking
both should avoid all of them. Renaming the variant is the fix.

**A variantless enum produces nothing**, silently. The derive succeeds and emits no impls, which is correct
and, as with the other empty shapes, means a mistake shows up later rather than here.

**It does not accept a struct.** The struct-side analogue is
[`#[derive(BuildField)]`](./derive_build_field.md).

## Related constructs

- [`#[derive(ExtractField)]`](./derive_extract_field.md) — the reverse operation, taking a variant out
  rather than putting one in; commonly derived alongside this one.
- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella derive, which includes this slice.
- [`#[derive(HasFields)]`](./derive_has_fields.md) — the whole-shape representation, which this derive does
  not generate, and the only derive that accepts every variant shape.
- [`#[derive(BuildField)]`](./derive_build_field.md) — the struct analogue: setting one field rather than
  choosing one variant.
- [`FromVariant`](../traits/from_variant.md) — the trait this generates impls of.
- [`Symbol!`](../macros/symbol.md) — the tag that names a variant.
- [`CanUpcast`](../traits/can_upcast.md) — widening a smaller enum into a larger one, built on these constructors.
- [Type-level spines](../types/type_level_spines.md) — the `Either`/`Void` chain the constructed variants
  correspond to.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — construction by name, and building through a
  small local enum before widening.

## Source

- Entry point: [`derive_from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_from_variant.rs)
- Codegen: [`cgp_data/derive_from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_from_variant.rs)
- Trait: [`from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/from_variant.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
