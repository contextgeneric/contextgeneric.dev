---
title: 'SerializeFields — write a struct with no serialization derive in cgp-serde'
sidebar_label: 'SerializeFields'
sidebar_position: 12
description: 'The cgp-serde provider that writes any struct deriving CGP''s field traits as a map from field names to values, asking the context to write each field.'
---

# `SerializeFields`

Write a struct as a map from each field's name to its value, asking the context to write each value.

## Overview

`SerializeFields` serializes a struct that derives nothing from Serde. It reads the struct's field
list through CGP's field traits, which
[`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data) provides, and asks the **context**,
the type whose wiring holds the application's choices, to write each field's value. So one generic
provider serves every struct, and each field is encoded however the context encodes its type.

## Definition

```rust
#[cgp_impl(new SerializeFields)]
impl<Value> ValueSerializer<Value>
where
    Value: HasFields,
    Value::Fields: FieldsSerializer<Self, Value>,
{ ... }
```

[`HasFields`](/docs/reference/traits/shape/has_fields) gives the struct's field list as a type.
`FieldsSerializer` is a private trait that walks that list, and it requires, for each field, that
the struct exposes it through [`HasField`](/docs/reference/traits/field-access/has_field) and that
the context can serialize the field's type.

## Usage

Import it from `cgp_serde::providers`. Derive the field traits on the struct, and wire it beside an
entry for each field's type, as in the [`basic`](../../examples/basic.md) example:

```rust
#[derive(Debug, Eq, PartialEq, CgpData)]
pub struct Payload {
    pub quantity: u64,
    pub message: String,
    pub data: Vec<u8>,
}

// in the context's table:
@ValueSerializerComponent.u64: UseSerde,
@ValueSerializerComponent.String: SerializeString,
@ValueSerializerComponent.Vec<u8>: SerializeHex,
@ValueSerializerComponent.Payload: SerializeFields,
```

That context writes a `Payload` as `{"quantity":42,"message":"hello","data":"010203"}`.

## Behavior

The provider starts a map and writes one entry per field, in declaration order. Each key is the
Rust field name exactly as written, and each value is the field wrapped with the context in
[`SerializeWithContext`](../types/serialize_with_context.md). The provider writes a map, with
`serialize_map`, where a derived `Serialize` impl writes a struct, and starts it without a declared
length. JSON shows no difference; see [using a
format](../../guides/formats.md#choose-a-format-that-fits-the-output) for formats that do.

Only structs with named fields work, since a field's name is its key.

## Context dependencies

`CanSerializeValue<F>` for the type `F` of every field: an entry in the context's table for each
field type the struct uses.

## Pairing

[`DeserializeRecordFields`](./deserialize_record_fields.md) reads the map back. The two agree on the
shape: a map keyed by Rust field names.

## When to use it

**Reach for `SerializeFields` for a struct whose fields should follow the context's choices**, and
for any struct whose crate should not depend on `serde`. For a struct that already derives Serde's
`Serialize` and needs no per-application choice, [`UseSerde`](./use_serde.md) uses that derive.

## Related constructs

- [`SerializeIterator`](./serialize_iterator.md) is the matching provider for a collection.

## The ideas behind it

- [Derive-free records](../../architecture/derive-free-records.md): serializing a struct through
  CGP's field traits.
- [Extensible records](/docs/concepts/extensible-records): the CGP idea of treating a struct's
  fields generically.

## Source

- [`crates/cgp-serde/src/providers/fields.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/fields.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
