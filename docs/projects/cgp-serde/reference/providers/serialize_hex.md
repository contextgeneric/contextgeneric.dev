---
title: 'SerializeHex — encode bytes as hex in cgp-serde'
sidebar_label: 'SerializeHex'
sidebar_position: 17
description: 'The cgp-serde provider that writes bytes as a lowercase hex string and reads a hex string back, asking the context to write and read the String.'
---

# `SerializeHex`

Write bytes as a lowercase hexadecimal string, and read a hex string back.

## Overview

`SerializeHex` is one of the four encodings in `cgp-serde-extra` that an application commonly wants
to choose for itself. It converts bytes to a hex `String` and asks the **context**, the type whose
wiring holds the application's choices, to write that string, and it reads a string through the
context and decodes it. One struct implements both directions, so a context names it in both tables.

## Definition

```rust
pub struct SerializeHex;

#[cgp_impl(SerializeHex)]
#[uses(CanSerializeValue<String>)]
impl<Value> ValueSerializer<Value>
where
    Value: ToHex,
{ ... }

#[cgp_impl(SerializeHex)]
#[uses(CanDeserializeValue<'de, String>)]
impl<'de, Value> ValueDeserializer<'de, Value>
where
    Value: FromHex<Error: Display>,
{ ... }
```

`ToHex` and `FromHex` are the traits of the [`hex`](https://docs.rs/hex) crate, implemented for byte
containers such as `Vec<u8>`, and `FromHex` also for fixed-size byte arrays.

## Usage

Import it from `cgp_serde_extra::providers`, and wire it in both directions beside an entry for
`String`, as the [`basic`](../../examples/basic.md) example does:

```rust
@ValueSerializerComponent.String: SerializeString,
@ValueSerializerComponent.Vec<u8>: SerializeHex,

@ValueDeserializerComponent.[u64, String]: UseSerde,
@ValueDeserializerComponent.Vec<u8>: SerializeHex,
```

## Behavior

Writing turns the bytes into lowercase hex and writes the string through the context, so `b"hi"` is
written as `"6869"`. Reading reads a `String` through the context and decodes it, accepting
uppercase digits too, and reports the `hex` crate's message for bad input: `"6G"` fails with
`Invalid character 'G' at position 1`. The two directions read back what they write.

## Context dependencies

`CanSerializeValue<String>` and `CanDeserializeValue<'de, String>`, usually wired to
[`UseSerde`](./use_serde.md) or [`SerializeString`](./serialize_string.md).

## Pairing

The same provider reads what it writes.

## When to use it

**Reach for `SerializeHex` for bytes in a text format when the output should be easy to read**, such
as keys and digests. [`SerializeBase64`](./serialize_base64.md) is the more compact text encoding.

## Related constructs

- [`SerializeBase64`](./serialize_base64.md), the other encoding of bytes, which `messages` wires in
  the second application.
- [`SerializeBytes`](./serialize_bytes.md), for formats with a native byte type.

## The ideas behind it

- [Component design](../../architecture/component-design.md#one-provider-can-serve-both-directions):
  one provider for both directions.
- [`messages`](../../examples/messages.md): hex in one application and base64 in another.

## Source

- [`crates/cgp-serde-extra/src/providers/hex.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-extra/src/providers/hex.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
