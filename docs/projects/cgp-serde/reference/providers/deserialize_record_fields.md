---
title: 'DeserializeRecordFields — read a struct with no serialization derive in cgp-serde'
sidebar_label: 'DeserializeRecordFields'
sidebar_position: 13
description: 'The cgp-serde provider that reads a struct deriving CGP''s field traits from a map, reading each field through the context.'
---

# `DeserializeRecordFields`

Read a struct from a map, reading each field through the context and building the struct field by
field.

## Overview

`DeserializeRecordFields` is the reading half of
[`SerializeRecordFields`](./serialize_record_fields.md). It reads a map, asks the **context**, the
type whose wiring holds the application's choices, to read each field's value, and collects the
values in CGP's builder for the struct until every field is present. The struct derives only CGP's
field traits, through [`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data) or the
narrower [`HasFields`](/docs/reference/traits/shape/has_fields) and
[`BuildField`](/docs/reference/traits/builder/build_field) derives.

## Definition

```rust
pub struct DeserializeRecordFields;

#[cgp_impl(DeserializeRecordFields)]
impl<'de, Record, Builder> ValueDeserializer<'de, Record>
where
    Record: HasOptionalBuilder<Builder = Builder> + HasFields,
    Record::Fields: HandleMapEntry<'de, Self, Builder>,
    Builder: FinalizeOptional<Target = Record>,
{ ... }
```

[`HasOptionalBuilder`](/docs/reference/traits/optional/has_optional_builder) gives a builder that
holds each field as an `Option`, so fields can arrive in any order, and
[`FinalizeOptional`](/docs/reference/traits/optional/finalize_optional) turns it into the struct
once every field is set. `HandleMapEntry` is a private trait that matches a key against the field
list, and it requires, for each field, that the context can read the field's type.

## Usage

Import it from `cgp_serde::providers`, and wire it beside an entry for each field's type. From the
[`basic`](../../examples/basic.md) example:

```rust
@ValueDeserializerComponent.[
    u64,
    String,
]:
    UseSerde,

@ValueDeserializerComponent.Payload:
    DeserializeRecordFields,

@ValueDeserializerComponent.Vec<u8>: SerializeHex,
```

## Behavior

The provider asks the format for a map and reads it entry by entry. It compares each key with the
field names, and on a match reads the value through the context and stores it in the builder. A key
that matches no field is skipped, with its value. When the map ends, the builder is finished, which
succeeds only if every field was set. The order of the keys does not matter.

The provider reports three failures, each through Serde's `Error::custom`, so the format adds its
position to the message:

- **A field is missing**, after the whole map is read. With JSON, `{"a":1}` read as a struct with
  fields `a` and `b` fails with `missing field: b at line 1 column 7`. A field whose type is an
  `Option` is no exception: it may be `null`, but it must be present.
- **A field appears twice**, at the second occurrence: `{"a":1,"a":2,"b":3}` fails with
  `duplicate field: a at line 1 column 12`.
- **The input is not a map**, as the format's type error; with JSON, an array fails with
  `invalid type: sequence, expected map`.

Only structs with named fields work, since a field's name is its key. A struct with a lifetime works
the same way: the arena example's `Payload<'a>` is read with this provider, its `Vec<&'a Coord>`
field through [`DeserializeExtend`](./deserialize_extend.md).

## Context dependencies

`CanDeserializeValue<'de, F>` for the type `F` of every field.

## Pairing

[`SerializeRecordFields`](./serialize_record_fields.md) writes the map it reads.

## When to use it

**Reach for `DeserializeRecordFields` to read a struct whose fields should follow the context's
choices.** Every field is required: a field wired to
[`DeserializeDefault`](./deserialize_default.md) still has to appear in the input, since that
provider replaces a null value, not a missing one.

## Related constructs

- [`DeserializeExtend`](./deserialize_extend.md) is the matching provider for a collection.

## The ideas behind it

- [Derive-free records](../../architecture/derive-free-records.md): reading a struct through CGP's
  field traits.
- [Extensible records](/docs/concepts/extensible-records): the builder the provider fills in.

## Source

- [`crates/cgp-serde/src/providers/record.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/record.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
