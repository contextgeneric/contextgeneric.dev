---
title: 'HasFields — a type''s whole shape'
sidebar_label: 'HasFields'
sidebar_position: 1
description: 'Name a type''s whole shape as a single type, every field with its name and type, so generic code can walk a struct or enum it cannot name.'
---

# `HasFields`

A type's whole shape, as a single type.

## Overview

Some code needs one field of a type. Other code needs the *shape*, every field, its name, and its
type, so it can walk them: a serializer, a validator, a builder that merges two structs, a
dispatcher that routes an enum to a handler per variant. None of those can be written one field at a
time, and none can name the concrete type either.

`HasFields` gives a type that shape as a single associated type. Where
[`HasField<Tag>`](../field-access/has_field.md), the singular, answers *"give me this field"*,
`HasFields` answers *"describe all of them at once"*.

**The impls come from [`#[derive(HasFields)]`](../../derives/derive_has_fields.md).** What you write
is the derive; what you bound against is the trait.

## Definition

`HasFields` carries one associated type and no method:

```rust
pub trait HasFields {
    type Fields;
}
```

`Self` is the type being described, a struct or an enum, and `Fields` is its shape written as a
single type. For a struct that shape is a [`Product!`](../../macros/product.md) list with one
[`Field`](../../types/field.md) entry per field, each pairing a value type with its type-level name.
For an enum it is a [`Sum!`](../../macros/sum.md) instead, the same idea for a choice rather than a
combination. There is no method, because describing a type is not an operation, so the associated
type is the whole trait.

For `struct Person { name: String, age: u8 }` the shape is:

```rust
type Fields = Product![
    Field<Symbol!("name"), String>,
    Field<Symbol!("age"), u8>,
];
```

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

The shape's construction follows one rule everywhere: a named field or variant is keyed by
[`Symbol!`](../../macros/symbol.md), a positional field by [`Index<N>`](../../types/index_type.md),
a struct becomes a `Cons`/`Nil` product, and an enum becomes an `Either`/`Void` sum. A single-field
tuple struct is the one special case, with its `Fields` the inner type directly rather than a
one-element product. The [derive's page](../../derives/derive_has_fields.md) covers that and the
other shapes in full.

**Four companions turn the description into a two-way door**, each on its own page:
[`HasFieldsRef`](./has_fields_ref.md) names the borrowed shape, and [`ToFields`](./to_fields.md),
[`FromFields`](./from_fields.md), and [`ToFieldsRef`](./to_fields_ref.md) convert between a concrete
value and it. This trait names the shape; those move values through it.

### Bounding on the shape

Generic code bounds on `T: HasFields` and works with `T::Fields`, so it applies to any type that
derives the shape, including one declared in another crate. The bound can also pin the shape, which
is how the example below checks it:

```rust
pub fn assert_shape<T, Fields>()
where
    T: HasFields<Fields = Fields>,
{
}
```

Require this trait alone when the code only names the shape, and one of the conversions when values
have to move through it.

**`HasFields` accepts every enum variant shape**: unit, tuple, multi-field, and struct-style. A
variant with one unnamed field carries its payload directly, a unit variant carries `Nil`, and any
other variant carries its fields as a nested product. The derives that take an enum apart need one
unnamed payload or no fields per variant, and give a variant with no fields the same `Nil`; this one
only describes the variant, so it accepts every shape.

## Examples

The shapes a struct, an enum, and a one-field tuple struct derive, each checked by a generic bound
that pins `Fields`:

```rust
use cgp::prelude::*;

#[derive(HasField, HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

pub struct Circle {
    pub radius: f64,
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(HasFields)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

#[derive(HasFields)]
pub struct UserId(pub u64);

pub fn assert_shape<T, Fields>()
where
    T: HasFields<Fields = Fields>,
{
}

pub fn demo() {
    assert_shape::<Config, Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>]>(
    );
    assert_shape::<
        Shape,
        Sum![Field<Symbol!("Circle"), Circle>, Field<Symbol!("Rectangle"), Rectangle>],
    >();
    assert_shape::<UserId, u64>();
}
```

`assert_shape` compiles only if the type's `Fields` is exactly the second parameter, so each call is
a compile-time check of one shape. The listing spells the shapes out to show their encoding. Code
that only needs to name a shape writes it as a declaration body with
[`Struct!`](../../macros/struct.md) or [`Enum!`](../../macros/enum.md), so the first two checks
could pass `Struct! { host: String, port: u16 }` and
`Enum! { Circle(Circle), Rectangle(Rectangle) }`, which are the same types.

`Config` also derives [`HasField`](../field-access/has_field.md), the common pairing that gives one
struct both indexed and whole-shape access. A variant with one
unnamed field carries its payload directly, so `Shape`'s entries hold `Circle` and `Rectangle`, and
the one-field `UserId` has the bare `u64` as its shape rather than a one-element product.

## When to use it

**Bound on `HasFields` when code must process a type's whole shape; bound on
[`HasField`](../field-access/has_field.md) when it needs one named field.** That is the entire
distinction, and the two are complementary rather than ranked. Most types that need both derive both
in one `#[derive(...)]`.

- **`HasFields` alone** when the code only names the shape: a `where` clause, an associated-type
  projection, or a type-level computation like [`AppendProduct`](../type-level/append_product.md).
- **[`ToFields`](./to_fields.md) and [`FromFields`](./from_fields.md)** when values move through the
  shape in both directions.
- **[`ToFieldsRef`](./to_fields_ref.md)** when the value must not be consumed. Requiring `ToFields`
  where a borrow would do forces callers to clone.

Two things this is not. It is **not runtime reflection**: a type has a shape only because it opted
in with a derive, there is nothing to query at run time, and the shape is a type rather than data.
That buys static checking and no runtime cost, and it costs one thing: a foreign type you cannot
patch has no shape at all. And it is **not the whole extensible-data story**. The shape describes a
type, while building one up field by field or taking one apart variant by variant is the
[builder](../builder/has_builder.md) and [extractor](../variant/extract_field.md) families, bundled
with this one by [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md).

## Under the hood

The trait module defines only the bare trait; every impl comes from
[`#[derive(HasFields)]`](../../derives/derive_has_fields.md). `cargo cgp expand` on the example's
`Config` and `Shape` shows the types it puts in `Fields`:

```rust
impl HasFields for Config {
    type Fields = Product![
        Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>
    ];
}
impl HasFields for Shape {
    type Fields = Sum![
        Field<Symbol!("Circle"), Circle>, Field<Symbol!("Rectangle"), Rectangle>
    ];
}
```

`Product![A, B]` is `Cons<A, Cons<B, Nil>>` and `Sum![A, B]` is `Either<A, Either<B, Void>>`, so a
`Fields` type in an error message is a chain rather than the sugar, which the [type-level
lists](../../types/index.md) page covers reading. Because a shape is a type rather than a value,
nothing about it costs anything at run time.

## Common Mistakes

**`HasFields` and [`HasField`](../field-access/has_field.md) differ by one letter and do not
overlap.** The plural is the whole shape and takes structs and enums; the singular is per-field
access and takes only structs. Their derives are likewise distinct.

**A newtype's shape is the inner type, not a one-element product.** `struct Wrapper(String)` has
`Fields = String`. Generic code written against a `Cons` chain will not match it. The
[derive's page](../../derives/derive_has_fields.md) has the rule and the other shapes.

**Field order is declaration order and is part of the type.** Two structs with the same names in
different orders have different `Fields` types. Name-driven conversions cope; code written against a
literal `Cons` chain does not.

**Two enum variant names are reserved.** A variant called `Fields` or `FieldsRef` collides with
these associated types and the derive fails; the [derive's page](../../derives/derive_has_fields.md)
lists it with the other reserved names.

**There is no method.** `Fields` is a type. Getting a *value* in that shape is
[`ToFields`](./to_fields.md).

## Related constructs

- [`#[derive(HasFields)]`](../../derives/derive_has_fields.md): generates this impl and the four
  companions'; what you write.
- [`HasFieldsRef`](./has_fields_ref.md): the borrowed shape.
- [`ToFields`](./to_fields.md), [`FromFields`](./from_fields.md), and
  [`ToFieldsRef`](./to_fields_ref.md): the conversions between a value and its shape.
- [`HasField`](../field-access/has_field.md): the singular counterpart, per-field access.
- [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md): bundles this with the builder and extractor.
- [`Product!`](../../macros/product.md) and [`Sum!`](../../macros/sum.md): the list types a shape is
  built from.
- [`Struct!`](../../macros/struct.md) and [`Enum!`](../../macros/enum.md): name a shape as the body of a
  struct or enum declaration, as in `HasFields<Fields = Struct! { name: String }>`.
- [`Field`](../../types/field.md): one entry: a value paired with its type-level name.
- [Type-level lists](../../types/index.md): the `Cons`/`Nil` and `Either`/`Void` chains
  underneath.
- [`AppendProduct`](../type-level/append_product.md): the operations that compute new shapes from
  old ones.
- [`CanUpcast`](../casting/can_upcast.md) and [`CanBuildFrom`](../casting/can_build_from.md):
  conversions between two types whose shapes overlap, driven by this one.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): a struct as a product of named fields.
- [Extensible variants](/docs/concepts/extensible-variants): the enum half, and the expression
  problem.

## Source

- [`has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_fields.rs):
  `HasFields` and `HasFieldsRef`
- Derive codegen:
  [`cgp_data/derive_has_fields/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
