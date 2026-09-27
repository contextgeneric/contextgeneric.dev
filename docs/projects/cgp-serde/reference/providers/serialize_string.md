---
title: 'SerializeString — write any string-like value in cgp-serde'
sidebar_label: 'SerializeString'
sidebar_position: 2
description: 'The cgp-serde provider that writes any AsRef<str> value as a Serde string and reads an owned String back, asking the context for nothing.'
---

# `SerializeString`

Write anything that can be viewed as a `str` as a Serde string, and read an owned `String`.

## Overview

`SerializeString` is a leaf provider for text. It writes any type that implements `AsRef<str>` with
Serde's string method, so `String`, `&str`, and `Box<str>` all come out as plain strings, and it
reads a `String` back. It asks the **context**, the type whose wiring holds an application's
choices, for nothing, which makes it one of the providers a context's table ends on: the hex,
base64, and RFC 3339 encodings each produce a `String` and ask the context to write it, and
`SerializeString` or [`UseSerde`](./use_serde.md) is what writes it.

## Definition

```rust
pub struct SerializeString;

#[cgp_impl(SerializeString)]
impl<Value> ValueSerializer<Value>
where
    Value: AsRef<str>,
{ ... }

#[cgp_impl(SerializeString)]
impl<'a> ValueDeserializer<'a, String> { ... }
```

The struct also implements Serde's `Visitor`, producing a `String` from a borrowed or an owned
string, which is what its reading impl uses.

## Usage

Import it from `cgp_serde::providers`. The [`basic`](../../examples/basic.md) example writes its
strings with it:

```rust
@ValueSerializerComponent.String: SerializeString,
```

A borrowed string is wired as its own type, since `str` cannot be wired alone:
`@ValueSerializerComponent.<'a> &'a str: SerializeString`.

## Behavior

Writing calls `serialize_str`, so the format escapes the string as it normally would. Reading
produces only `String`, and accepts both a string the input can lend and one the format had to
copy, so an escaped JSON string such as `"a\nb"` reads as a string containing a newline. Input that
is not a string is the format's type error; with JSON, a number fails with
`invalid type: integer ..., expected string`.

## Context dependencies

None.

## Pairing

The same provider reads what it writes, for `String`.

## When to use it

**Reach for `SerializeString` to write a string-like type that does not implement `Serialize`
itself**, or to write several string-like types the same way. For `String` alone,
[`UseSerde`](./use_serde.md) writes the same output.

## Related constructs

- [`SerializeWithDisplay`](./serialize_with_display.md) writes any `Display` type as the string it
  formats to.
- [`SerializeBytes`](./serialize_bytes.md) is the leaf provider for bytes.

## The ideas behind it

- [Re-entrant providers](../../architecture/reentrant-providers.md): why a table needs a leaf for
  the strings its encodings produce.

## Source

- [`crates/cgp-serde/src/providers/string.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/string.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
