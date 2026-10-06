---
title: 'SerializeBase64 — encode bytes as base64 in cgp-serde'
sidebar_label: 'SerializeBase64'
sidebar_position: 18
description: 'The cgp-serde provider that writes bytes as a standard padded base64 string and reads one back into a Vec<u8>.'
---

# `SerializeBase64`

Write bytes as a standard, padded base64 string, and read one back into a `Vec<u8>`.

## Overview

`SerializeBase64` is the compact text encoding of bytes in `cgp-serde-extra`. It converts bytes to a
base64 `String` and asks the **context**, the type whose wiring holds the application's choices, to
write that string, and it reads a string through the context and decodes it. One struct implements
both directions.

## Definition

```rust
pub struct SerializeBase64;

#[cgp_impl(SerializeBase64)]
#[uses(CanSerializeValue<String>)]
impl<Value> ValueSerializer<Value>
where
    Value: AsRef<[u8]>,
{ ... }

#[cgp_impl(SerializeBase64)]
#[uses(CanDeserializeValue<'de, String>)]
impl<'de> ValueDeserializer<'de, Vec<u8>> { ... }
```

Writing accepts any `AsRef<[u8]>`; reading produces a `Vec<u8>`.

## Usage

Import it from `cgp_serde_extra::providers`. The second application in the
[`messages`](../../examples/messages.md) example writes its bytes with it:

```rust
@ValueSerializerComponent.Vec<u8>:
    SerializeBase64,
```

## Behavior

Both directions use the standard base64 alphabet with padding, from the
[`base64`](https://docs.rs/base64) crate. Writing turns `b"hi"` into `"aGk="`, and reading turns
`"aGk="` back into the bytes. Input without its padding, such as `"aGk"`, fails with
`Invalid padding`.

## Context dependencies

`CanSerializeValue<String>` and `CanDeserializeValue<'de, String>`.

## Pairing

The same provider reads what it writes, for `Vec<u8>`.

## When to use it

**Reach for `SerializeBase64` for bytes in a text format when size matters more than readability.**
The encoding is the standard padded one; a format that expects URL-safe or unpadded base64 needs a
provider of its own, which [writing a provider](../../guides/writing-a-provider.md) shows how to
write.

## Related constructs

- [`SerializeHex`](./serialize_hex.md), the readable encoding of bytes.

## The ideas behind it

- [`messages`](../../examples/messages.md): base64 in one application and hex in another.

## Source

- [`crates/cgp-serde-extra/src/providers/base64.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-extra/src/providers/base64.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
