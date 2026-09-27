---
title: 'SerializeWithContext — hand a context to any Serde format in cgp-serde'
sidebar_label: 'SerializeWithContext'
sidebar_position: 1
description: 'The cgp-serde adapter that pairs a context with a value as a Serde Serialize value, so any format writes through the context.'
---

# `SerializeWithContext`

Pair a context with a value as an ordinary Serde `Serialize` value.

## Overview

`SerializeWithContext` is the way into Serde's formats. Serde's functions, such as
`serde_json::to_string`, accept a value that implements `Serialize` and have no place for a
**context**, the type whose wiring holds an application's choices. The adapter carries both, and its
`Serialize` impl writes the value through the context's wiring. Providers use the same adapter to
hand each nested value back to the context from inside Serde's compound writers.

## Definition

```rust
pub struct SerializeWithContext<'a, Context, T> {
    pub context: &'a Context,
    pub value: &'a T,
}

impl<'a, Context, T> SerializeWithContext<'a, Context, T> {
    pub fn new(context: &'a Context, value: &'a T) -> Self { ... }
}

impl<'a, Context, T> serde::Serialize for SerializeWithContext<'a, Context, T>
where
    Context: CanSerializeValue<T>,
{ ... }
```

It lives in `cgp_serde::types`.

## Usage

Build it with `new` and pass it to any Serde format. From the
[`messages`](../../examples/messages.md) example:

```rust
let serialized =
    serde_json::to_string_pretty(&SerializeWithContext::new(&AppA, &archive)).unwrap();
```

Both fields are public, so a provider can build it with a struct literal, as the library's own
providers do.

## Behavior

Serializing the adapter calls the context's serializer for `T`, so the context's wiring decides the
output, and every nested value goes back through the same wiring. The format's own error comes back
unchanged, since the adapter involves none of CGP's error handling. `T` must be a sized type.

## Context dependencies

`Context: CanSerializeValue<T>`, which needs whatever the provider wired for `T` needs.

## Pairing

[`DeserializeWithContext`](./deserialize_with_context.md) is the adapter for reading.

## When to use it

**Reach for `SerializeWithContext` to write with any Serde format**, and to hand a nested value to
one of Serde's compound writers inside a provider. For a JSON string in the context's own error
type, [`SerializeToJsonString`](../providers/serialize_to_json_string.md) wraps it.

## Related constructs

- [`SerializeIterator`](../providers/serialize_iterator.md) and
  [`SerializeFields`](../providers/serialize_fields.md) use it for each item and field.

## The ideas behind it

- [The bridge to
  Serde](../../architecture/serde-bridge.md#going-out-handing-a-context-to-a-serde-api): how a
  context reaches a Serde API.
- [Re-entrant providers](../../architecture/reentrant-providers.md#re-entering-through-an-adapter):
  the adapter inside a provider.

## Source

- [`crates/cgp-serde/src/types/serialize_with_context.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/types/serialize_with_context.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
