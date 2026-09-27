---
title: 'SerializeWithDisplay — write a Display value as a string in cgp-serde'
sidebar_label: 'SerializeWithDisplay'
sidebar_position: 5
description: 'The cgp-serde provider that writes any Display value as the string it formats to, asking the context to write the String.'
---

# `SerializeWithDisplay`

Write any `Display` value as the string it formats to.

## Overview

`SerializeWithDisplay` covers every type whose natural encoding is its printed form, such as an
identifier or a unit of measure, with one provider. It formats the value with `Display`, then asks
the **context**, the type whose wiring holds the application's choices, to write the resulting
`String`, so how strings are written stays the context's choice.

## Definition

```rust
#[cgp_impl(new SerializeWithDisplay)]
#[uses(CanSerializeValue<String>)]
impl<Value> ValueSerializer<Value>
where
    Value: Display,
{ ... }
```

[`#[uses]`](/docs/reference/attributes/uses) declares the one thing it asks of the context: writing
a `String`.

## Usage

Import it from `cgp_serde::providers`, and wire it beside an entry for `String`:

```rust
@ValueSerializerComponent.Celsius: SerializeWithDisplay,
@ValueSerializerComponent.String: UseSerde,
```

## Behavior

The provider formats the value with `to_string` and writes the result through the context's entry
for `String`. A `Celsius(21.5)` whose `Display` writes `21.5C` is written to JSON as `"21.5C"`. The
formatted string is allocated on every call.

## Context dependencies

`CanSerializeValue<String>`. Wiring `String` itself to `SerializeWithDisplay` would make the
provider ask for itself, and the context fails to compile; wire `String` to a leaf such as
[`UseSerde`](./use_serde.md) or [`SerializeString`](./serialize_string.md).

## Pairing

[`DeserializeWithFromStr`](./deserialize_with_from_str.md) reads the string back with `FromStr`.

## When to use it

**Reach for `SerializeWithDisplay` when a type's printed form is its encoding**, and the type has a
`FromStr` that parses it back. For a type that implements `AsRef<str>`,
[`SerializeString`](./serialize_string.md) writes it without formatting.

## Related constructs

- [`SerializeFrom`](./serialize_from.md) encodes through any type the value converts into.

## The ideas behind it

- [Re-entrant providers](../../architecture/reentrant-providers.md#re-entering-directly): a provider
  that converts a value and asks the context to encode the result.

## Source

- [`crates/cgp-serde/src/providers/display.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/display.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
