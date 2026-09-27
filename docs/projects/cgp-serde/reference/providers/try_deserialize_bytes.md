---
title: 'TryDeserializeBytes — read a value from bytes by TryFrom in cgp-serde'
sidebar_label: 'TryDeserializeBytes'
sidebar_position: 4
description: 'The cgp-serde provider that reads a byte slice and converts it with TryFrom, such as into a fixed-size array.'
---

# `TryDeserializeBytes`

Read a value from a byte slice through a fallible `TryFrom` conversion.

## Overview

`TryDeserializeBytes` reads bytes the way [`SerializeBytes`](./serialize_bytes.md) does, then
converts them into the target type with `TryFrom`, reporting a failed conversion as an error. A
fixed-size array is the typical target, since a slice converts to one only when the lengths match.
It asks the **context**, the type whose wiring holds an application's choices, for nothing.

## Definition

```rust
#[cgp_impl(new TryDeserializeBytes)]
impl<'a, Value> ValueDeserializer<'a, Value>
where
    Value: TryFrom<&'a [u8], Error: Display>,
{ ... }
```

## Usage

Import it from `cgp_serde::providers`. A fixed-size array cannot be written as a key, because square
brackets are the syntax for a list of keys, so key it on a type alias:

```rust
type Digest = [u8; 32];

// in the context's table:
@ValueDeserializerComponent.Digest: TryDeserializeBytes,
```

## Behavior

The provider reads a byte slice the input can lend, as `SerializeBytes` does, and converts it,
reporting the conversion error's `Display` message through Serde's `Error::custom`. A `[u8; 3]`
reads from the JSON string `"abc"`, and from `"ab"` fails with `could not convert slice to array`.

## Context dependencies

None.

## Pairing

It reads only. A value that is `AsRef<[u8]>` is written with
[`SerializeBytes`](./serialize_bytes.md).

## When to use it

**Reach for `TryDeserializeBytes` when the target cannot be built from every byte slice**, such as a
fixed-size array. For a type that converts from any slice, such as `Vec<u8>`, `SerializeBytes` reads
it without the conversion.

## Related constructs

- [`TrySerializeFrom`](./try_serialize_from.md) is the general fallible conversion, through any
  type the context can read.

## The ideas behind it

- [Wiring a
  context](../../guides/wiring-a-context.md#3-write-keys-for-references-lifetimes-and-arrays): why
  an array is keyed on an alias.

## Source

- [`crates/cgp-serde/src/providers/bytes.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/bytes.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
