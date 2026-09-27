---
title: 'DeserializeWithFromStr — read a value by parsing a string in cgp-serde'
sidebar_label: 'DeserializeWithFromStr'
sidebar_position: 6
description: 'The cgp-serde provider that reads a string borrowed from the input through the context and parses it with FromStr.'
---

# `DeserializeWithFromStr`

Read any `FromStr` value by parsing a string.

## Overview

`DeserializeWithFromStr` is the reading half of an encoding through text. It asks the **context**,
the type whose wiring holds the application's choices, to read a `&str` borrowed from the input,
then parses it with `FromStr`. It reads back what
[`SerializeWithDisplay`](./serialize_with_display.md) writes, for a type whose `Display` and
`FromStr` agree.

## Definition

```rust
#[cgp_impl(new DeserializeWithFromStr)]
#[uses(CanDeserializeValue<'a, &'a str>)]
impl<'a, Value> ValueDeserializer<'a, Value>
where
    Value: FromStr<Err: Display>,
{ ... }
```

## Usage

Import it from `cgp_serde::providers`, and wire the borrowed string it asks for, usually to
[`UseSerde`](./use_serde.md):

```rust
@ValueDeserializerComponent.Celsius: DeserializeWithFromStr,
@ValueDeserializerComponent.<'a> &'a str: UseSerde,
```

## Behavior

The provider reads a `&str` through the context and parses it, reporting the parse error's
`Display` message through Serde's `Error::custom`. With `&'a str` wired to `UseSerde`, a `u64` wired
to this provider reads from the JSON string `"42"`, and `"x"` fails with
`invalid digit found in string`.

Because it asks for a string borrowed from the input, it reads only strings the input can lend. A
JSON string that `serde_json` has to rebuild in a new buffer, such as one containing an escaped
quote or newline, cannot be lent, and fails with `expected a borrowed string` before any parsing.
Neither can input read from a stream, where every string fails the same way.

## Context dependencies

`CanDeserializeValue<'de, &'de str>`, usually met by wiring `<'a> &'a str` to `UseSerde`.

## Pairing

[`SerializeWithDisplay`](./serialize_with_display.md) writes the string it reads.

## When to use it

**Reach for `DeserializeWithFromStr` for text-encoded values read from a string or a byte slice**,
where the strings are plain, such as numbers, identifiers, and codes. For input read from a stream,
or strings that may contain escapes, write a provider that asks the context for an owned `String`
and parses that; [writing a provider](../../guides/writing-a-provider.md) shows the shape.

## Related constructs

- [`TrySerializeFrom`](./try_serialize_from.md) reads through any type that converts into the
  value.

## The ideas behind it

- [Re-entrant providers](../../architecture/reentrant-providers.md#re-entering-directly): a provider
  that reads another type through the context and converts it.

## Source

- [`crates/cgp-serde/src/providers/display.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/display.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
