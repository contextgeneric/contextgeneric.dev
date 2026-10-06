---
title: 'SerializeVariantFields — write an enum with no serialization derive in cgp-serde'
sidebar_label: 'SerializeVariantFields'
sidebar_position: 14
description: 'The cgp-serde provider that writes any enum deriving CGP''s variant traits in Serde''s externally tagged form, asking the context to write the payload.'
---

# `SerializeVariantFields`

Write an enum as `{"Variant": payload}`, asking the context to write the payload.

## Overview

`SerializeVariantFields` serializes an enum that derives nothing from Serde. It reads the enum's
variants through CGP's field traits, which
[`#[derive(CgpVariant)]`](/docs/reference/derives/derive_cgp_variant) provides, and asks the
**context**, the type whose wiring holds the application's choices, to write the active variant's
payload. So one generic provider serves every such enum, and each payload is encoded however the
context encodes its type.

Every variant must hold one unnamed payload, as `Circle(Circle)` does, or no fields, as `Empty`
does, which is also what `CgpVariant` requires. A variant with no fields has the payload `Nil`, so
the context decides how it is written through its entry for `Nil`.

## Definition

```rust
pub struct SerializeVariantFields;

#[cgp_impl(SerializeVariantFields)]
impl<Value> ValueSerializer<Value>
where
    Value: ToFieldsRef,
    for<'a> Value::FieldsRef<'a>: VariantsSerializer<Self>,
{ ... }
```

[`ToFieldsRef`](/docs/reference/traits/shape/to_fields_ref) borrows the enum as a list of its
variants, each paired with its name. `VariantsSerializer` is a private trait that finds the active
variant in that list, and it requires, for each variant, that the context can write the payload's
type, `Nil` for a variant with no fields.

## Usage

Import it from `cgp_serde::providers`, and wire it beside an entry for each variant's payload type:

```rust
#[derive(CgpVariant)]
pub enum Shape {
    Circle(Circle),
    Label(String),
    Empty,
}

delegate_components! {
    App {
        open ValueSerializerComponent;

        @ValueSerializerComponent.[u64, String]: UseSerde,
        @ValueSerializerComponent.Nil: SerializeUnit,
        @ValueSerializerComponent.Circle: SerializeRecordFields,
        @ValueSerializerComponent.Shape: SerializeVariantFields,
    }
}
```

Here `Circle` is a struct with one `radius: u64` field, so its own entry needs `u64`, and
[`SerializeUnit`](./serialize_unit.md) writes the `Nil` of `Empty`.

## Behavior

The provider writes the active variant as a Serde newtype variant, with the variant's name and its
position in the declaration. With JSON, `Shape::Circle(Circle { radius: 3 })` is written as
`{"Circle":{"radius":3}}` and `Shape::Label("hi".into())` as `{"Label":"hi"}`, exactly what
Serde's derive writes for those variants. `Shape::Empty` is written as `{"Empty":null}`, because
`Nil` is wired to `SerializeUnit`. Every empty form is written that way, while Serde's derive
writes each form its own way:

| Variant | This provider | Serde's derive |
|---|---|---|
| `Empty` | `{"Empty":null}` | `"Empty"` |
| `Empty()` | `{"Empty":null}` | `{"Empty":[]}` |
| `Empty {}` | `{"Empty":null}` | `{"Empty":{}}` |

Serde's derive reads `{"Empty":null}` back only for the first form. In postcard the two agree,
since every form is written as the variant's position alone.

Other formats see the same Serde calls. RON writes `Label("x")`, and postcard writes the variant's
position followed by its payload. Both match Serde's derive, except where the payload is a struct:
structs are written as maps without a declared length, which RON shows in map syntax and postcard
rejects.

## Context dependencies

`CanSerializeValue<P>` for the payload type `P` of every variant, which is `Nil` for a variant with
no fields.

## Pairing

[`DeserializeVariantFields`](./deserialize_variant_fields.md) reads what it writes.

## When to use it

**Reach for `SerializeVariantFields` to write an enum whose payloads should follow the context's
choices.** Its limits are worth knowing first:

- **The enum must not borrow.** An enum with a lifetime, such as `Token<'a>` holding a `&'a str`,
  fails to compile with "the type `Token<'a>` does not fulfill the required lifetime", because of a
  current limitation in how Rust proves lifetime bounds. `Token<'static>` works, and reading a
  borrowing enum with [`DeserializeVariantFields`](./deserialize_variant_fields.md) works too.
- **Only Serde's default enum form is written.** There is no internally tagged, adjacently tagged,
  or untagged form, and a variant with no fields is written as `{"Empty":null}`, which does not
  match Serde's derive in JSON or RON.
- **An enum that contains itself**, such as an expression tree, fails to compile, as a struct that
  contains itself does.

An enum outside these limits keeps its own Serde impl and is wired to
[`UseSerde`](./use_serde.md).

## Related constructs

- [`SerializeRecordFields`](./serialize_record_fields.md) is the matching provider for a struct, and
  usually writes a variant's payload.
- [`SerializeUnit`](./serialize_unit.md) writes the `Nil` payload of a variant with no fields.

## The ideas behind it

- [Derive-free records](../../architecture/derive-free-records.md): serializing data through CGP's
  field traits instead of a serialization derive.
- [Extensible variants](/docs/concepts/extensible-variants): an enum seen as a list of named
  variants.

## Source

- [`crates/cgp-serde/src/providers/variant_fields.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/variant_fields.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
