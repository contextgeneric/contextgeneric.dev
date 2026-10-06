---
title: 'UseSerde — use a type''s own Serde impl in cgp-serde'
sidebar_label: 'UseSerde'
sidebar_position: 1
description: 'The cgp-serde provider that serializes or deserializes a value through the type''s own Serde impl, so every existing Serde impl is available to a context.'
---

# `UseSerde`

Serialize or deserialize a value through the value type's own Serde impl.

## Overview

`UseSerde` is how a cgp-serde context reuses Serde. Wired for a type, it calls that type's
`Serialize` or `Deserialize` impl, so every impl in the standard library and in other crates is
available to a **context**, the type whose wiring holds an application's choices, without a
provider being written for it. It is the usual choice for the scalar types at the bottom of a
context's table, such as `u64`, `i64`, and `String`, where the application has no reason to choose
an encoding.

## Definition

```rust
pub struct UseSerde;

#[cgp_impl(UseSerde)]
impl<Value> ValueSerializer<Value>
where
    Value: SerdeSerialize,
{ ... }

#[cgp_impl(UseSerde)]
impl<'a, Value> ValueDeserializer<'a, Value>
where
    Value: SerdeDeserialize<'a>,
{ ... }
```

`SerdeSerialize` and `SerdeDeserialize` are Serde's `Serialize` and `Deserialize`, imported under
those names. One struct implements both directions.

## Usage

Import it from `cgp_serde::providers`, and wire the types that should keep their Serde encoding:

```rust
@ValueSerializerComponent.[u64, String]: UseSerde,
@ValueDeserializerComponent.[u64, String]: UseSerde,
```

## Behavior

The provider ignores the context and calls the Serde impl directly, so the output is exactly what
plain Serde would produce. That also means the context's choices stop at the value. The type's own
impl serializes whatever the value contains, so a struct deriving Serde's `Serialize` with a
`Vec<u8>` field writes that field as Serde does, an array of numbers in JSON, even in a context that
wires `Vec<u8>` to hex.

The deserializing impl passes Serde's `'de` lifetime through, so `UseSerde` can produce a value that
borrows from the input when the Serde impl does. Wired for `<'a> &'a str`, it reads a string
borrowed from the input, when the input can lend one.

## Context dependencies

None. `UseSerde` asks the context for nothing.

## Pairing

The same provider reads what it writes.

## When to use it

**Reach for `UseSerde` for a type whose Serde encoding the application is happy with**, which is
most scalar types, and for any type cgp-serde's generic providers do not cover, such as an enum
with a derived Serde impl.

Reach for [`SerializeRecordFields`](./serialize_record_fields.md) and
[`DeserializeRecordFields`](./deserialize_record_fields.md) instead when a struct's fields should
follow the context's choices, since `UseSerde` hands the whole struct to its own impl.

## Related constructs

- [`SerializeString`](./serialize_string.md) writes any string-like type as a string, where
  `UseSerde` needs the exact type to implement `Serialize`.
- [`SerializeWithContext`](../types/serialize_with_context.md) goes the other way, handing a context
  to a Serde API.

## The ideas behind it

- [The bridge to Serde](../../architecture/serde-bridge.md#going-in-using-a-types-own-serde-impl):
  how values cross between cgp-serde and Serde.
- [`#[cgp_impl]`](/docs/reference/macros/cgp_impl): the macro the provider is written with.

## Source

- [`crates/cgp-serde/src/providers/serde.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/serde.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
