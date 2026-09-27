---
title: 'DeserializeWithContext — read with a context through any Serde format in cgp-serde'
sidebar_label: 'DeserializeWithContext'
sidebar_position: 2
description: 'The cgp-serde adapter that pairs a context with a target type as a Serde DeserializeSeed, so any format reads through the context.'
---

# `DeserializeWithContext`

Pair a context with a target type as a Serde `DeserializeSeed`.

## Overview

`DeserializeWithContext` is the way to read with any Serde format. Serde's `Deserialize` has no
receiver, so there is nowhere to put a **context**, the type whose wiring holds an application's
choices. Serde's `DeserializeSeed` is the trait for deserializing with state, and this adapter is a
seed whose state is the context: driving it with a format's deserializer reads the target type
through the context's wiring.

## Definition

```rust
pub struct DeserializeWithContext<'a, Context, Value> {
    pub context: &'a Context,
    pub phantom: PhantomData<Value>,
}

impl<'a, Context, Value> DeserializeWithContext<'a, Context, Value> {
    pub fn new(context: &'a Context) -> Self { ... }
}

impl<'de, 'a, Context, Value> DeserializeSeed<'de> for DeserializeWithContext<'a, Context, Value>
where
    Context: CanDeserializeValue<'de, Value>,
{ ... }
```

It lives in `cgp_serde::types`.

## Usage

Serde's convenience functions such as `serde_json::from_str` accept only a `Deserialize` type, not a
seed, so build the format's deserializer, drive the seed with it, and check for trailing input:

```rust
let mut deserializer = serde_json::Deserializer::from_str(&text);
let value: Rec = DeserializeWithContext::new(&app).deserialize(&mut deserializer)?;
deserializer.end()?;
```

`deserialize` here is `DeserializeSeed::deserialize`, so `serde::de::DeserializeSeed` must be in
scope.

## Behavior

Driving the seed calls the context's deserializer for `Value` and returns the value. Every nested
value goes back through the same wiring. The format's own error comes back unchanged. The adapter
needs no error wiring on the context.

## Context dependencies

`Context: CanDeserializeValue<'de, Value>`.

## Pairing

[`SerializeWithContext`](./serialize_with_context.md) is the adapter for writing.

## When to use it

**Reach for `DeserializeWithContext` to read with a format that has no cgp-serde helper**, or to
read JSON with no error wiring. For JSON in the context's own error type,
[`DeserializeFromJsonReader`](../providers/deserialize_from_json_reader.md) drives the seed and
checks for trailing input itself.

## Related constructs

- [`DeserializeRecordFields`](../providers/deserialize_record_fields.md) uses it for each field.

## The ideas behind it

-  [The bridge to
  Serde](../../architecture/serde-bridge.md#going-out-handing-a-context-to-a-serde-api): why reading
  needs a seed.
- [Using a format](../../guides/formats.md#read-with-another-format): the seed with RON.

## Source

- [`crates/cgp-serde/src/types/deserialize_with_context.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/types/deserialize_with_context.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
