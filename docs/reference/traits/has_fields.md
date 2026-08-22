---
sidebar_label: 'HasFields'
---

# `HasFields`

A type's whole shape, as a single type.

## Overview

Some code needs one field of a type. Other code needs the *shape* — every field, its name, and its type
— so it can walk them: a serializer, a validator, a builder that merges two structs, a dispatcher that
routes an enum to a handler per variant. None of those can be written one field at a time, and none can
name the concrete type either.

`HasFields` gives a type that shape as a single associated type. Where
[`HasField<Tag>`](./has_field.md) — singular — answers *"give me this field"*, `HasFields` answers
*"describe all of them at once"*:

```rust
pub trait HasFields {
    type Fields;
}
```

For `struct Person { name: String, age: u8 }` that is:

```rust
type Fields = Product![
    Field<Symbol!("name"), String>,
    Field<Symbol!("age"), u8>,
];
```

The struct written as a type: a [`Product!`](../macros/product.md) list with one
[`Field`](../types/field.md) entry per field, each pairing a value type with its type-level name. For an
enum it is a [`Sum!`](../macros/sum.md) instead — the same idea for a choice rather than a combination.

**The impls come from [`#[derive(HasFields)]`](../derives/derive_has_fields.md).** What you write is the
derive; what you bound against is the trait.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough. There is no method, because describing a
type is not an operation — `Fields` is the whole trait.

The shape's construction follows one rule everywhere: a named field or variant is keyed by
[`Symbol!`](../macros/symbol.md), a positional field by [`Index<N>`](../types/index.md), a struct becomes
a `Cons`/`Nil` product, and an enum becomes an `Either`/`Void` sum. A single-field tuple struct is the one
special case — its `Fields` is the inner type directly rather than a one-element product — and the
[derive's page](../derives/derive_has_fields.md) covers that and the other shapes in full.

**Four companions turn the description into a two-way door**, each on its own page:
[`HasFieldsRef`](./has_fields_ref.md) names the borrowed shape, and [`ToFields`](./to_fields.md),
[`FromFields`](./from_fields.md), and [`ToFieldsRef`](./to_fields_ref.md) convert between a concrete
value and it. This trait names the shape; those move values through it.

### Bounding on the shape

The point of all this is a bound. Generic code writes `T: HasFields` and recurses over `T::Fields`, so it
applies to any type that derives the shape — including one declared in another crate:

```rust
fn describe<T>() -> &'static str
where
    T: HasFields,
{
    // recurse over T::Fields
    todo!()
}
```

Require this trait alone when the code only *names* the shape; require one of the conversions when values
have to move through it.

## Examples

Deriving it alongside [`HasField`](./has_field.md) is the common pairing, giving one struct both indexed
and structural access:

```rust
use cgp::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, HasField, HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}
```

`Config::Fields` is now `Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>]`, which is
what a generic routine recurses over.

An enum's shape is a sum rather than a product:

```rust
#[derive(HasFields)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

giving `Sum![Field<Symbol!("Circle"), Circle>, Field<Symbol!("Rectangle"), Rectangle>]`.

**`HasFields` is the one member of the extensible-data family that accepts every variant shape** — unit,
tuple, multi-field, and struct-style — because it only *describes* a variant rather than deconstructing
it. The derives that take an enum apart need exactly one unnamed payload per variant; this one does not.

## When to reach for it, and when not

**Bound on `HasFields` when code must process a type's whole shape; bound on
[`HasField`](./has_field.md) when it needs one named field.** That is the entire distinction, and the two
are complementary rather than ranked — most types that need both derive both in one `#[derive(...)]`.

- **`HasFields` alone** when the code only names the shape — a `where` clause, an associated-type
  projection, a type-level computation like [`AppendProduct`](./append_product.md).
- **[`ToFields`](./to_fields.md) and [`FromFields`](./from_fields.md)** when values move through the
  shape in both directions.
- **[`ToFieldsRef`](./to_fields_ref.md)** when the value must not be consumed. Requiring `ToFields` where
  a borrow would do forces callers to clone.

Two things this is not. It is **not runtime reflection**: a type has a shape only because it opted in
with a derive, there is nothing to query at run time, and the shape is a type rather than data. What that
buys is static checking and no runtime cost; what it costs is that a foreign type you cannot patch has no
shape at all. And it is **not the whole extensible-data story** — the shape describes a type, while
building one up field by field or taking one apart variant by variant is the
[builder](./has_builder.md) and [extractor](./extract_field.md) families, bundled with this one by
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md).

## Under the hood

:::note

### Advanced

This section shows what the shape expands to. You do not need it to bound on the trait, but the `Fields`
type appears verbatim in compiler errors about structural code, so reading one makes those errors
legible. `cargo cgp expand` prints it resugared for your own code.

:::

The trait module defines only the bare trait; every impl comes from
[`#[derive(HasFields)]`](../derives/derive_has_fields.md). The load-bearing part is what the derive puts
in `Fields`, and the sugar hides one level of structure:

```rust
impl HasFields for Person {
    type Fields = Product![
        Field<Symbol!("name"), String>,
        Field<Symbol!("age"), u8>,
    ];
}
```

`Product![A, B]` is `Cons<A, Cons<B, Nil>>`, so a `Fields` type in an error message is a `Cons` chain
rather than the sugar — the [type-level spines](../types/type_level_spines.md) page covers reading it.

An enum's shape is the dual: an `Either` chain terminated by `Void` rather than a `Cons` chain terminated
by `Nil`, with each arm tagged by the variant name and carrying that variant's own fields as a nested
product.

Because a shape is a type rather than a value, nothing about this costs anything at run time. It exists
to give the trait solver something to recurse over.

## Gotchas

**`HasFields` and [`HasField`](./has_field.md) differ by one letter and do not overlap.** The plural is
the whole shape and takes structs and enums; the singular is per-field access and takes only structs.
Their derives are likewise distinct.

**A newtype's shape is the inner type, not a one-element product.** `struct Wrapper(String)` has
`Fields = String`. Generic code written against a `Cons` chain will not match it — the
[derive's page](../derives/derive_has_fields.md) has the rule and the other shapes.

**Field order is declaration order and is part of the type.** Two structs with the same names in
different orders have different `Fields` types. Name-driven conversions cope; code written against a
literal `Cons` chain does not.

**Two enum variant names are reserved.** A variant called `Fields` or `FieldsRef` collides with these
associated types and the derive fails; the [derive's page](../derives/derive_has_fields.md) lists it with
the other reserved names.

**There is no method.** `Fields` is a type. Getting a *value* in that shape is
[`ToFields`](./to_fields.md).

## Related constructs

- [`#[derive(HasFields)]`](../derives/derive_has_fields.md) — generates this impl and the four
  companions'; what you write.
- [`HasFieldsRef`](./has_fields_ref.md) — the borrowed shape.
- [`ToFields`](./to_fields.md), [`FromFields`](./from_fields.md), and
  [`ToFieldsRef`](./to_fields_ref.md) — the conversions between a value and its shape.
- [`HasField`](./has_field.md) — the singular counterpart, per-field access.
- [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) — bundles this with the builder and extractor.
- [`Product!`](../macros/product.md) and [`Sum!`](../macros/sum.md) — the list types a shape is built
  from.
- [`Field`](../types/field.md) — one entry: a value paired with its type-level name.
- [Type-level spines](../types/type_level_spines.md) — the `Cons`/`Nil` and `Either`/`Void` chains
  underneath.
- [`AppendProduct`](./append_product.md) — the operations that compute new shapes from old ones.
- [`CanUpcast`](./can_upcast.md) and [`CanBuildFrom`](./can_build_from.md) — conversions between two
  types whose shapes overlap, driven by this one.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum half, and the expression problem.

## Source

- [`has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_fields.rs)
  — `HasFields` and `HasFieldsRef`
- Derive codegen: [`cgp_data/derive_has_fields/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
