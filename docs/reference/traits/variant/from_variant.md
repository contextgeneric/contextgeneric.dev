---
title: 'FromVariant — build an enum from one variant'
sidebar_label: 'FromVariant'
sidebar_position: 7
description: 'Construct an enum from a single variant chosen by its type-level name, so generic code can build a variant it cannot name as a constructor.'
---

# `FromVariant`

Generic construction of an enum from a named variant.

## Overview

`Shape::Circle(circle)` names two things: the enum and the variant. That is fine where both are known and
useless to code that knows neither: a routine handed a payload and told which variant to wrap it in cannot
write that expression.

`FromVariant<Tag>` gives it a way to. The variant is selected by its *name as a type*, so the choice becomes a
parameter rather than syntax:

```rust
Shape::from_variant(PhantomData::<Symbol!("Circle")>, circle)
```

That call is exactly `Shape::Circle(circle)`. The difference is that the tag can come from a type parameter, so
one function can build whichever variant it was asked for, on whatever enum implements the trait for that tag.

**This is the smallest trait in the extensible-data family.** It needs neither a companion type nor
any state, because building a single variant leaves nothing to track. It is the construction
counterpart to [`ExtractField`](./extract_field.md)'s deconstruction, and the impls come from
[`#[derive(FromVariant)]`](../../derives/derive_from_variant.md), one per variant.

## Definition

`FromVariant<Tag>` carries the variant's payload type as an associated `Value` and one associated
function:

```rust
pub trait FromVariant<Tag> {
    type Value;

    fn from_variant(_tag: PhantomData<Tag>, value: Self::Value) -> Self;
}
```

`Self` is the enum. `Tag` is the variant's name as a [`Symbol!`](../../macros/symbol.md) type-level
string, and `Value` is that variant's payload type. `from_variant` wraps a payload into the enum as
the chosen variant. The `PhantomData<Tag>` argument carries nothing but the tag; it lets a caller
pick which variant to build when several impls, one per variant, are in scope on the same enum.
Because the trait is implemented once per variant, each impl fixing its own `Tag` and `Value`,
choosing the impl **is** choosing the variant.

## Usage

The trait is in the prelude, so `use cgp::prelude::*;` names it. You rarely write a `FromVariant` impl
yourself: the derive supplies one per variant, and you either call
`T::from_variant(PhantomData::<Symbol!("Variant")>, value)` at a concrete site or bound on the trait in
code generic over the tag.

The associated `Value` makes a generic signature possible: a function that does not know the variant
still needs to name the type it takes, and projects it through the trait as
`<Shape as FromVariant<Tag>>::Value`, the payload type *derived from the tag*, as `wrap` does in the
[example](#examples). Without it there would be nothing to write in the parameter position, which is
the whole reason the trait carries an associated type rather than taking the payload as a second
parameter.

## Examples

One `wrap` builds either variant, with its payload type following the tag, and a narrow enum widens
into `Shape`:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::CanUpcast;

#[derive(Debug, PartialEq)]
pub struct Circle {
    pub radius: f64,
}

#[derive(Debug, PartialEq)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, PartialEq, FromVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

pub fn wrap<Tag>(tag: PhantomData<Tag>, value: <Shape as FromVariant<Tag>>::Value) -> Shape
where
    Shape: FromVariant<Tag>,
{
    Shape::from_variant(tag, value)
}

// A routine that only ever produces circles works in a one-variant enum.
#[derive(HasFields, ExtractField)]
pub enum RoundShape {
    Circle(Circle),
}

pub fn unit_circle() -> Shape {
    RoundShape::Circle(Circle { radius: 1.0 }).upcast(PhantomData::<Shape>)
}

pub fn demo() {
    let circle = wrap(PhantomData::<Symbol!("Circle")>, Circle { radius: 2.0 });
    assert_eq!(circle, Shape::Circle(Circle { radius: 2.0 }));

    let rect = wrap(
        PhantomData::<Symbol!("Rectangle")>,
        Rectangle { width: 3.0, height: 4.0 },
    );
    assert_eq!(rect, Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 }));

    assert_eq!(unit_circle(), Shape::Circle(Circle { radius: 1.0 }));
}
```

A hand-written function cannot do what `wrap` does, because `Shape::Circle` and `Shape::Rectangle`
are different expressions taking different types.

`unit_circle` shows where this matters in practice, **building through a narrow enum and widening**.
A routine that only produces some of a large enum's variants declares a small local enum, constructs
into that, and lifts the result. The upcast always succeeds, because every variant of the smaller
enum has a home in the larger one, and `FromVariant` rebuilds each variant into the target, which is
why `Shape` derives nothing else. Upcasting is documented with the other [structural
casts](../casting/can_upcast.md).

## When to use it

**Bound on `FromVariant` when the variant to build is decided by a type parameter.** That is the
whole test, and it is narrower than the extractor's, because most code decides which variant to
build at a site that can name it.

- **Bound on it in a routine parameterized over the variant it produces.** Nothing else can do it.
- **Derive it to make a smaller enum upcastable into a larger one.** Casting between enums is built on these
  constructors, so this lets an implementation work in a narrow local enum and widen the result.
- **Do not reach for it for an ordinary constructor call.** `Shape::Circle(circle)` is shorter, clearer, and
  generates nothing. The trait adds a *second* way to do the same thing, for callers that cannot use the first.
- **Do not derive it alone if you also take the enum apart**, which is the usual case; reach for
  [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md), which bundles construction, deconstruction, and the
  shape.

The names capture the split exactly: this trait puts a value *into* an enum,
[`ExtractField`](./extract_field.md) gets one *out*. For structs, the analogous field-setting primitive is
[`BuildField`](../builder/has_builder.md).

## Under the hood

The derive emits **one impl per variant and nothing else**. `cargo cgp expand` on the example's
`Shape` shows them:

```rust
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Circle")>,
        value: Self::Value,
    ) -> Self {
        Self::Circle(value)
    }
}
impl FromVariant<Symbol!("Rectangle")> for Shape {
    type Value = Rectangle;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Rectangle")>,
        value: Self::Value,
    ) -> Self {
        Self::Rectangle(value)
    }
}
```

Each body is the plain constructor call, without an intermediate type, a marker, or any check beyond
the type system's own that the payload matches. Because the impls are distinguished *only* by their
`Tag` parameter, resolving a `from_variant` call comes down to which `Symbol!` the caller names: the
compiler picks the matching impl and inlines it to the corresponding constructor. So the generic
call costs exactly what the concrete one does.

The trait itself is defined in the library; the derive supplies only these per-variant impls. Each
is spanned at the variant it came from, so a conflict with a hand-written impl underlines that
variant rather than the whole derive, as [Common Mistakes](#common-mistakes) shows.

## Common Mistakes

**The tag must be written out whenever the enum has two or more variants.** With a bare
`PhantomData`:

```rust
let _ = Shape::from_variant(PhantomData, Circle { radius: 1.0 });
```

rustc does not pick the impl from the payload's type, since two variants could share one:

```text
error[E0283]: type annotations needed
...
note: multiple `impl`s satisfying `Shape: cgp::prelude::FromVariant<_>` found
```

A one-variant enum has a single impl, so there the tag is inferred.

**Two variants with the same payload type are distinguishable only by tag.** That is the reason for the previous
point, and it means a mistyped tag is a missing-impl error rather than a type mismatch.

**A variant name is matched exactly.** `Symbol!("Circle")` and `Symbol!("circle")` are unrelated
types, so `Shape::from_variant(PhantomData::<Symbol!("circle")>, circle)` reports an unsatisfied
bound:

```text
error[E0277]: the trait bound `Shape: cgp::prelude::FromVariant<cgp::prelude::Symbol<6, cgp::prelude::Chars<'c', cgp::prelude::Chars<'i', cgp::prelude::Chars<'r', cgp::prelude::Chars<'c', cgp::prelude::Chars<'l', cgp::prelude::Chars<'e', Nil>>>>>>>>` is not satisfied
```

**A hand-written impl for a variant the derive covers conflicts with it.** Adding
`impl FromVariant<Symbol!("Circle")> for Shape` beside the derive fails with `E0119`, and the
primary label sits on the variant:

```text
error[E0119]: conflicting implementations of trait `cgp::prelude::FromVariant<cgp::prelude::Symbol<6, cgp::prelude::Chars<'C', cgp::prelude::Chars<'i', cgp::prelude::Chars<'r', cgp::prelude::Chars<'c', cgp::prelude::Chars<'l', cgp::prelude::Chars<'e', Nil>>>>>>>>` for type `Shape`
...
14 |     Circle(Circle),
   |     ^^^^^^ conflicting implementation for `Shape`
```

**A variant named `Value` does not compile**, because the generated signature names the payload as
`Self::Value`. The [derive's page](../../derives/derive_from_variant.md) covers this with the family's other
reserved names.

**Every variant needs exactly one unnamed payload**, which is the derive's requirement rather than the trait's:
a unit, multi-field, or struct-style variant cannot be given a single `Value`. Wrap the payload in its own
struct.

## Related constructs

- [`#[derive(FromVariant)]`](../../derives/derive_from_variant.md): generates the per-variant impls; what you
  write.
- [`ExtractField`](./extract_field.md): the reverse operation, and the trait most often derived alongside.
- [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md): bundles this with the extractor and the shape.
- [`HasBuilder`](../builder/has_builder.md): the struct analogue: setting one field rather than choosing one variant.
- [`Symbol!`](../../macros/symbol.md): the tag that names a variant.
- [`CanUpcast`](../casting/can_upcast.md): widening a narrow enum into a wider one, built on these constructors.
- [`HasFields`](../shape/has_fields.md): the variant shape a cast walks while rebuilding.
- [Dispatch combinators](../../providers/dispatch/index.md): where variant construction meets routing.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants): construction by name, and building through a
  small local enum before widening.

## Source

- Trait: [`from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/from_variant.rs)
- Derive codegen: [`cgp_data/derive_from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_from_variant.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
