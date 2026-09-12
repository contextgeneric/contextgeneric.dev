---
sidebar_label: '#[derive(HasFields)]'
sidebar_position: 2
---

# `#[derive(HasFields)]`

`#[derive(HasFields)]` generates a structural representation and conversions for a struct or enum.

## Overview

Generic serializers, validators, record conversions, and enum dispatchers need a description of the
whole type. Individual field access does not tell them which fields or variants exist or how to
traverse them.

`#[derive(HasFields)]` describes that structure through a single associated type, `Fields`.
[`HasField`](./derive_has_field.md) supplies access to one field selected by a tag; `HasFields`
describes all fields together, for example:

```rust
type Fields = Product![
    Field<Symbol!("name"), String>,
    Field<Symbol!("age"), u8>,
];
```

A struct's representation combines its fields in a [`Product!`](../macros/product.md), with a
[`Field`](../types/field.md) entry pairing each value type with its tag. An enum uses a
[`Sum!`](../macros/sum.md) to represent a choice of variants. Generic code can recurse over these
representations, whose types are resolved during compilation.

The derive also supplies conversions to and from the representation. Generic code can decompose a
concrete value, process its structural form, and reconstruct the concrete type.

## Usage

Apply `HasFields` to a struct or enum without arguments or helper attributes:

```rust
#[derive(HasFields)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

#[derive(HasFields)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

For a struct the shape is a product; for an enum it is a sum. Naming follows the same rules everywhere: a
named field or a variant is keyed by [`Symbol!`](../macros/symbol.md), a positional field by
[`Index<N>`](../types/index_type.md). Applying the derive to anything other than a struct or an enum is a
compile error.

### Struct shapes

Every struct shape is accepted, with its representation determined by its fields:

- **Named fields:** a product of `Field<Symbol!("name"), T>` entries in declaration order.
- **Multiple tuple fields:** a product with fields keyed by `Index<N>`.
- **Unit struct:** the empty product `Nil`, with conversions to and from the unit value.
- **Single tuple field (newtype):** the inner type directly, without a one-element product wrapper.

Adding a second field to a newtype changes the kind of representation. `struct Wrapper(String)` has
`Fields = String`; a tuple struct with multiple fields has a product of indexed entries. Generic code
that depends on the original representation must account for that change.

### Variant shapes

`HasFields` accepts unit, newtype, multi-field tuple, and named-field variants. Its representation
applies the struct field rules within each variant. In contrast, the per-variant constructor and
extractor derives ([`FromVariant`](./derive_from_variant.md) and
[`ExtractField`](./derive_extract_field.md), also included by [`CgpData`](./derive_cgp_data.md))
require exactly one unnamed payload per variant.

```rust
#[derive(HasFields)]
pub enum Shape {
    Empty,
    Circle(u32),
    Rectangle(u32, u32),
    Triangle { base: u32, height: u32 },
}
```

The variants above map to these entries in the sum:

| Variant | Its entry in the sum |
|---|---|
| `Empty` | `Field<Symbol!("Empty"), Nil>` |
| `Circle(u32)` | `Field<Symbol!("Circle"), u32>` |
| `Rectangle(u32, u32)` | `Field<Symbol!("Rectangle"), Product![Field<Index<0>, u32>, Field<Index<1>, u32>]>` |
| `Triangle { base, height }` | `Field<Symbol!("Triangle"), Product![Field<Symbol!("base"), u32>, Field<Symbol!("height"), u32>]>` |

Every listed shape supports conversion to and from the representation. A unit variant uses `Nil`, a
newtype variant uses its payload type directly, and multi-field variants use products keyed by
position or name.

An enum with mixed variant shapes can derive `HasFields` but cannot derive `CgpData`. It gains a
structural representation and whole-value conversions without per-variant constructors or an
incremental extractor.

### Generic types

Generic parameters, lifetimes, and a `where` clause are carried onto every generated impl. A borrowed
field type appears verbatim in the shape, and the borrowed view of the shape layers its own borrow on top
of it: a field of type `&'a Name` appears as `&'__a &'a Name` in the borrowed form, where `'__a` is the
reserved lifetime the derive introduces.

## Examples

Deriving `HasFields` alongside [`HasField`](./derive_has_field.md) is the common pairing, giving one
struct both indexed and structural access:

```rust
use cgp::prelude::*;

#[derive(HasField, HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}
```

`Config::Fields` is now `Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>]`, and a
value round-trips through it:

```rust
let config = Config { host: "localhost".to_owned(), port: 8080 };

let fields = config.to_fields();                 // Config -> the product
let config_again = Config::from_fields(fields);  // the product -> Config
```

`to_fields_ref()` borrows the fields so generic code can read them without consuming the value:

```rust
let config = Config { host: "localhost".to_owned(), port: 8080 };

let fields_ref = config.to_fields_ref();  // borrows each field in place
```

An enum's shape is a sum rather than a product, and the same conversions apply:

```rust
#[derive(HasFields)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

`Shape::Fields` is `Sum![Field<Symbol!("Circle"), Circle>, Field<Symbol!("Rectangle"), Rectangle>]`.

Generic code can operate on `Self::Fields` under a `Self: HasFields` bound. An implementation that
supports the resulting representation can work with `Config`, `Person`, or a struct in another crate
that supplies the same traits.

## When to use it

Use `HasFields` when code must process a type's whole structure. Use
[`HasField`](./derive_has_field.md) for access to an individual field. Derive both when both forms of
access are needed.

Choose the extent of the generated support according to the operations you need:

- **`HasFields` alone:** describe and convert a type for serialization, structural reads, or dispatch.
  This also supports enums with mixed variant shapes.
- **[`CgpData`](./derive_cgp_data.md):** include incremental building or extraction alongside the
  representation. For structs, it also includes per-field access.
- **[`HasField`](./derive_has_field.md) alone:** read individual fields when the whole representation
  is unnecessary. `HasFields` generates five implementations, so omit it when nothing uses them.

The representation is a type-level description, not runtime reflection. Generic code uses trait
bounds to access the shape during compilation; it does not query field metadata at runtime. A type
must supply the representation traits before it can participate. If you cannot implement those traits
for a type, this derive cannot expose its shape.

## Under the hood

The derive preserves the type definition and adds implementations of `HasFields`, `HasFieldsRef`,
`ToFields`, `FromFields`, and `ToFieldsRef`. For this named-field struct:

```rust
#[derive(HasFields)]
pub struct Person {
    pub name: String,
    pub age: u8,
}
```

The shape implementations name the product and its borrowed counterpart:

```rust
impl HasFields for Person {
    type Fields = Product![
        Field<Symbol!("name"), String>,
        Field<Symbol!("age"), u8>,
    ];
}

impl HasFieldsRef for Person {
    type FieldsRef<'__a> = Product![
        Field<Symbol!("name"), &'__a String>,
        Field<Symbol!("age"), &'__a u8>,
    ]
    where
        Self: '__a;
}
```

The conversion implementations move or borrow values between `Person` and that product:

```rust
impl ToFields for Person {
    fn to_fields(self) -> Self::Fields {
        Cons(self.name.into(), Cons(self.age.into(), Nil))
    }
}

impl FromFields for Person {
    fn from_fields(Cons(name, Cons(age, Nil)): Self::Fields) -> Self {
        Self { name: name.value, age: age.value }
    }
}

impl ToFieldsRef for Person {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.name).into(), Cons((&self.age).into(), Nil))
    }
}
```

`Product![A, B]` is sugar for `Cons<A, Cons<B, Nil>>`, which is why the bodies build a `Cons` chain. The
[type-level lists](../types/index.md) page covers that shape, and it is the form printed in
an error.

An enum's sum representation is an [`Either`](../types/either.md) chain terminated by `Void`.
Each arm carries a field tagged by the variant name:

```rust
impl HasFields for Shape {
    type Fields = Either<
        Field<Symbol!("Circle"), Circle>,
        Either<Field<Symbol!("Rectangle"), Rectangle>, Void>,
    >;
}
```

The enum conversions map each variant to its corresponding sum arm and back. Each variant's payload
representation follows the struct rules: `Nil` for a unit variant, the payload itself for a newtype
variant, or a product keyed by `Index` or `Symbol!` for multiple fields.

A unit struct uses the empty product `Nil` as its representation:

```rust
impl HasFields for Unit {
    type Fields = Nil;
}
```

A variantless enum uses `Void` as its representation. Its conversions use empty matches because the
value is uninhabited.

Each generated impl is aimed at the type name the user wrote, so a conflict with a hand-written
`HasFields` impl underlines the struct or enum rather than the whole `#[derive(HasFields)]`.

## Common Mistakes

**A newtype's `Fields` is the inner type, not a one-element product.** `struct Wrapper(String)` has
`Fields = String`. Generic code written against `Cons<Field<Index<0>, _>, Nil>` will not match it, and
adding a second field changes the shape rather than lengthening it.

**Named-field and newtype variants have different representations.** `Circle { radius: f64 }` uses a
one-entry product, while `Circle(Circle)` uses its payload type directly.

**Only `HasFields` accepts every variant shape in this family.** Unit, multi-field, and struct-style
variants are rejected by `CgpData`, `CgpVariant`, `ExtractField`, and `FromVariant`. Wrap richer
payloads in dedicated structs to use those derives.

**The borrowed shape adds a reference to each field's declared type.** A field of type `&'a Name`
appears as `&'__a &'a Name` in the borrowed form. The derive introduces the reserved lifetime `'__a`
for that additional borrow.

**Field order is declaration order, and it is part of the type.** Two structs with the same field names in
different orders have different `Fields` types. Structural conversions that match on names cope with that;
code written against a literal `Cons` chain does not.

**Variants named `Fields` or `FieldsRef` make generated paths ambiguous.** The implementations use
`Self::Fields` and `Self::FieldsRef`, which conflict with variants of those names. The compiler
reports `ambiguous associated item` at the derive and points to the offending variant in a note.
Rename that variant. The [extractor](./derive_extract_field.md) and
[constructor](./derive_from_variant.md) derives reserve additional names, so check their restrictions
when deriving the full family.

**`HasField` and `HasFields` generate different interfaces.** The singular derive provides per-field
access for structs. The plural derive provides a whole-type representation for structs and enums.

## Related constructs

These references cover the related derives, generated traits, and supporting types:

- [`#[derive(HasField)]`](./derive_has_field.md) — the singular counterpart, per-field access, commonly
  derived alongside this one.
- [`HasFields`](../traits/shape/has_fields.md) — the traits this generates and the conversions between a value
  and its shape.
- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella derive, which includes this one plus the
  incremental machinery.
- [`Product!`](../macros/product.md) and [`Sum!`](../macros/sum.md) — the list types a struct and an enum
  shape are built from.
- [`Field`](../types/field.md) — one entry: a value paired with its type-level name.
- [`Symbol!`](../macros/symbol.md) and [`Index`](../types/index_type.md) — the tags that name an entry.
- [Type-level lists](../types/index.md) — the `Cons`/`Nil` and `Either`/`Void` chains the
  sugar expands to.
- [`AppendProduct`](../traits/type-level/append_product.md) — operations over a shape once you have one.
- [`CanUpcast`](../traits/casting/can_upcast.md) — converting between two types whose shapes overlap.

The ideas behind it are explained on these concept pages:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields, and
  what generic code does with one.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum half, and the expression problem it
  addresses.

## Source

The implementation is defined in these source files:

- Entry point: [`derive_has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_has_fields.rs)
- Codegen: [`cgp_data/derive_has_fields/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields)
- Traits: [`has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_fields.rs),
  [`to_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/to_fields.rs),
  and [`from_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/from_fields.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
