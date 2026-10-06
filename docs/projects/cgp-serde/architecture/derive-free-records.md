---
sidebar_label: 'Derive-free records'
sidebar_position: 4
description: 'How cgp-serde serializes a struct that derives only CGP''s general field traits, so a data crate needs no serde dependency, and what that approach gives up.'
---

# Derive-free records

Can a struct be serialized if it derives nothing from Serde? [cgp-serde](../index.md) rebuilds
Serde's `Serialize` and `Deserialize` as components of [CGP](/docs/), and it serializes structs
that derive only CGP's general-purpose field traits. This page explains why that matters, what a
struct still has to derive, and what the approach gives up compared with Serde's derive.

## With Serde, the type's owner decides

A type is serializable with Serde only if its owner derives or implements Serde's traits for it.
Rust's orphan rule stops any other crate from implementing `Serialize` for it, so a library whose
types should be serializable has to depend on `serde` and derive the traits itself. An application
that wants a library's type encoded differently has to wrap it in a newtype or copy it.

## With cgp-serde, one general derive is enough

A struct that derives CGP's [`CgpData`](/docs/reference/derives/derive_cgp_data) exposes its fields
to generic code: the list of field names and types, a way to read each field, and a way to build the
struct one field at a time. This is CGP's [extensible records](/docs/concepts/extensible-records),
and it is not specific to serialization. The `basic` example's struct derives nothing else:

```rust
#[derive(Debug, Eq, PartialEq, CgpData)]
pub struct Payload {
    pub quantity: u64,
    pub message: String,
    pub data: Vec<u8>,
}
```

Two generic providers are written once against what `CgpData` exposes.
[`SerializeRecordFields`](../reference/providers/serialize_record_fields.md) walks the field list and writes the
struct as a map from each field's name to its value, asking the **context**, the type whose wiring
holds an application's choices, to serialize each value.
[`DeserializeRecordFields`](../reference/providers/deserialize_record_fields.md) reads the map back,
asking the context to deserialize each value, and fills the struct in field by field, reporting any
field the input left out. A context wires the struct to them like any other type:

```rust
@ValueSerializerComponent.Payload:
    SerializeRecordFields,

@ValueDeserializerComponent.Payload:
    DeserializeRecordFields,
```

So a crate that defines data types needs to depend only on `cgp`, and each application that uses
them decides, in its own wiring, how they are encoded.

## What a struct still derives

The approach removes the serialization derive, not every derive. Each direction needs particular
field traits: writing needs the field list, from
[`HasFields`](/docs/reference/traits/shape/has_fields), and access to each field, from
[`HasField`](/docs/reference/traits/field-access/has_field); reading needs the field list and the
field-by-field builder, from [`BuildField`](/docs/reference/traits/builder/build_field). `CgpData`
derives all three. A struct that derives only the narrower ones, as the arena tests' structs derive
`HasFields` and `BuildField` for reading, works in the directions they cover.

The same derive serves every other generic CGP code that works over fields, such as builders and
conversions between struct [shapes](/docs/reference/glossary#shape), so one opt-in covers more than serialization. A type whose owner
has not derived the field traits, including any type from a crate that does not use CGP, cannot use
the struct providers; it is encoded through its own Serde impl with
[`UseSerde`](../reference/providers/use_serde.md) instead.

## What it gives up

The generic providers know only what the field list tells them: each field's name, its type, and
their order. They have no equivalent of the attributes Serde's derive reads, so every field is
written under its Rust name, and a field cannot be renamed, skipped, flattened, or given a default
when it is missing. Choices are made per type instead, through the context: a `Vec<u8>` field is
encoded however the context encodes `Vec<u8>`.

The providers also cover one shape of type: structs with named fields. Tuple structs and enums are
not handled by them, and an enum is encoded through its own Serde impl, with `UseSerde`. The
[comparison with Serde](../serde-comparison.md) lists what Serde's derive does that cgp-serde's
struct providers do not.

## What it costs

A derive-free record trades Serde's per-field control for per-application control. Where Serde lets
the type's author annotate each field once, cgp-serde lets each application choose each type's
encoding, and that suits types whose encoding is the application's business rather than the
author's. For a type that has one right encoding, the author's annotations are the simpler place to
say it.

The generic code is also compiled for each struct, as Serde's derived code is, so the approach saves
writing the code rather than compiling it.

## Where to go next

- [Context services](./context-services.md): deserializers that take something from the context
  while they work.
- [`basic`](../examples/basic.md): a derive-free struct written to JSON and read back.
- [Extensible records](/docs/concepts/extensible-records): the CGP idea behind reading and building
  a struct generically.
- [Reflection](/docs/comparisons/reflection): how this compares with Serde's derive and with
  reflection in other languages.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
