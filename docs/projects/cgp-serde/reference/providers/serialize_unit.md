---
title: 'SerializeUnit — write a variant with no fields in cgp-serde'
sidebar_label: 'SerializeUnit'
sidebar_position: 16
description: 'The cgp-serde provider that writes any value as Serde''s unit and reads a unit back as the type''s default, used for the payload of an enum variant with no fields.'
---

# `SerializeUnit`

Write any value as Serde's unit, and read a unit back as the type's default.

## Overview

`SerializeUnit` is the provider for a type with nothing to write. Its main use is `Nil`, the payload
that [`#[derive(CgpVariant)]`](/docs/reference/derives/derive_cgp_variant) gives an enum variant
with no fields, such as `Empty`, `Empty()`, or `Empty {}`. The
[variant providers](./serialize_variant_fields.md) ask the **context**, the type whose wiring holds
the application's choices, to write and read each payload, so a context with such a variant needs an
entry for `Nil`, and `SerializeUnit` is the usual one.

## Definition

```rust
pub struct SerializeUnit;

#[cgp_impl(SerializeUnit)]
impl<Value> ValueSerializer<Value> { ... }

#[cgp_impl(SerializeUnit)]
impl<'de, Value> ValueDeserializer<'de, Value>
where
    Value: Default,
{ ... }
```

One struct implements both directions. Serializing accepts any value, and deserializing needs a
`Default` to build the value from.

## Usage

Import it from `cgp_serde::providers`, and wire it for `Nil` in both directions:

```rust
@ValueSerializerComponent.Nil: SerializeUnit,
@ValueDeserializerComponent.Nil: SerializeUnit,
```

`Nil` is in `cgp::prelude`, so the entry needs no further import.

## Behavior

The serializer ignores the value and writes Serde's unit, which JSON writes as `null`, RON as `()`,
and postcard as nothing at all. Inside an enum, a variant `Status::Closed` therefore becomes
`{"Closed":null}` in JSON and `Closed(())` in RON.

The deserializer reads a unit and returns `Value::default()`. Anything else is rejected, so with
JSON, `{"Closed":{}}` fails with:

```text
invalid type: map, expected unit at line 1 column 10
```

## Context dependencies

None. `SerializeUnit` asks the context for nothing.

## Pairing

The same provider reads what it writes.

## When to use it

**Reach for `SerializeUnit` for `Nil`, so that an enum with variants that have no fields can be
written and read.** Wire it only for a type with nothing to write: the serializer accepts any value
and writes none of its data, so a type that carries data would lose it.

A context that wants another form for the payload wires `Nil` to its own provider instead. The
entry decides only the payload, so Serde's own forms, such as the bare name `"Closed"` for a unit
variant, are not available this way:
[`SerializeVariantFields`](./serialize_variant_fields.md) always writes the variant around a
payload.

## Related constructs

- [`SerializeVariantFields`](./serialize_variant_fields.md) and
  [`DeserializeVariantFields`](./deserialize_variant_fields.md) ask the context for the `Nil`
  payload this provider handles.
- [`UseSerde`](./use_serde.md) writes `()` the same way, through Serde's own impl for `()`.

## The ideas behind it

- [Extensible variants](/docs/concepts/extensible-variants): an enum seen as a list of named
  variants, each with one payload.

## Source

- [`crates/cgp-serde/src/providers/unit.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/unit.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
