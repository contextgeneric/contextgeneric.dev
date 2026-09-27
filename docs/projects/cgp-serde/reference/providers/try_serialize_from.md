---
title: 'TrySerializeFrom — encode through a fallible conversion in cgp-serde'
sidebar_label: 'TrySerializeFrom'
sidebar_position: 8
description: 'The cgp-serde provider that encodes a value through another type with TryInto in either direction, reporting a failed conversion as a Serde error.'
---

# `TrySerializeFrom`

Encode a value through another type, with a conversion that can fail.

## Overview

`TrySerializeFrom<Target>` is the fallible form of [`SerializeFrom`](./serialize_from.md). It
converts with `TryInto` rather than `Into`, and reports a failed conversion as an error, so it can
encode through a type the value does not always fit, such as a narrower integer. Like
`SerializeFrom`, it asks the **context**, the type whose wiring holds an application's choices, to
encode the other type.

## Definition

```rust
pub struct TrySerializeFrom<Target>(pub PhantomData<Target>);

#[cgp_impl(TrySerializeFrom<Target>)]
#[uses(CanSerializeValue<Target>)]
impl<Value, Target> ValueSerializer<Value>
where
    Value: Clone + TryInto<Target, Error: Display>,
{ ... }

#[cgp_impl(TrySerializeFrom<Source>)]
#[uses(CanDeserializeValue<'a, Source>)]
impl<'a, Value, Source> ValueDeserializer<'a, Value>
where
    Source: TryInto<Value, Error: Display>,
{ ... }
```

## Usage

Import it from `cgp_serde::providers`, and name the in-between type as its parameter:

```rust
@ValueSerializerComponent.u16: TrySerializeFrom<u8>,
@ValueDeserializerComponent.i8: TrySerializeFrom<u64>,
@ValueSerializerComponent.u8: UseSerde,
@ValueDeserializerComponent.u64: UseSerde,
```

## Behavior

The directions mirror `SerializeFrom`, with the conversion error's `Display` message reported
through the serializer's or deserializer's `Error::custom`. A `u16` written as
`TrySerializeFrom<u8>` writes `7` for `7`, and for `300` fails with
`out of range integral type conversion attempted`. An `i8` read as `TrySerializeFrom<u64>` reads
`5`, and fails on `500` with the same message. Writing clones the value, so the value type must
implement `Clone`.

## Context dependencies

`CanSerializeValue<Target>` when writing, and `CanDeserializeValue<'de, Target>` when reading.

## Pairing

The same provider implements both directions.

## When to use it

**Reach for `TrySerializeFrom` when the conversion can fail**, and a failure should be an error
rather than a panic or a silent truncation. When the conversion always succeeds,
[`SerializeFrom`](./serialize_from.md) does the same without the error path.

## Related constructs

- [`TryDeserializeBytes`](./try_deserialize_bytes.md) converts from a byte slice with `TryFrom`.

## The ideas behind it

- [Writing a provider](../../guides/writing-a-provider.md#report-errors-through-serde): reporting a
  failure through the format's error.

## Source

- [`crates/cgp-serde/src/providers/try_from.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/try_from.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
