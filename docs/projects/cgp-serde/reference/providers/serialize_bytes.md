---
title: 'SerializeBytes — write bytes with Serde''s byte method in cgp-serde'
sidebar_label: 'SerializeBytes'
sidebar_position: 3
description: 'The cgp-serde provider that writes any AsRef<[u8]> value with Serde''s byte method and reads bytes the input can lend, for formats with a byte type.'
---

# `SerializeBytes`

Write anything that can be viewed as a byte slice with Serde's byte method, and read bytes the input
can lend.

## Overview

`SerializeBytes` is a leaf provider for binary data. It hands the bytes to the format's own byte
method, so the format decides how bytes look, and it reads a value back from a byte slice. It asks
the **context**, the type whose wiring holds an application's choices, for nothing.

## Definition

```rust
pub struct SerializeBytes;

#[cgp_impl(SerializeBytes)]
impl<Value> ValueSerializer<Value>
where
    Value: AsRef<[u8]>,
{ ... }

#[cgp_impl(SerializeBytes)]
impl<'a, Value> ValueDeserializer<'a, Value>
where
    Value: From<&'a [u8]>,
{ ... }
```

The struct also implements Serde's `Visitor` for a `&'a [u8]` borrowed from the input, which both
its reading impl and [`TryDeserializeBytes`](./try_deserialize_bytes.md) use.

## Usage

Import it from `cgp_serde::providers`, and wire it for a byte type in a format with a byte
representation of its own:

```rust
@ValueSerializerComponent.Vec<u8>: SerializeBytes,
@ValueDeserializerComponent.Vec<u8>: SerializeBytes,
```

## Behavior

Writing calls `serialize_bytes`, so the output is the format's byte form. JSON has no byte type, and
`serde_json` writes bytes as an array of numbers, so `vec![1, 2, 3]` becomes `[1,2,3]`.

Reading asks the format for bytes, and accepts them only when the format can lend them straight
from its input. With `serde_json` reading a string, an unescaped JSON string is lent as its raw
bytes, so `"abc"` reads as `[97, 98, 99]`. A JSON array is not a byte string to `serde_json`, so it
is rejected with `invalid type: sequence, expected bytes`, and so is a string the format had to
copy, such as one with an escape in it.

## Context dependencies

None.

## Pairing

The same provider implements both directions. [`TryDeserializeBytes`](./try_deserialize_bytes.md)
reads through a fallible conversion instead.

## When to use it

**Reach for `SerializeBytes` in a format with a native byte type**, where the format's own byte
encoding is what the application wants.

For bytes in a text format such as JSON, reach for [`SerializeHex`](./serialize_hex.md) or
[`SerializeBase64`](./serialize_base64.md) instead. They write the bytes as a string, and read that
string back.

## Related constructs

- [`SerializeString`](./serialize_string.md) is the leaf provider for text.

## The ideas behind it

- [Using a format](../../guides/formats.md#choose-a-format-that-fits-the-output): how the output
  meets each format.

## Source

- [`crates/cgp-serde/src/providers/bytes.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/bytes.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
