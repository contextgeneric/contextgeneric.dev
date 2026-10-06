---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'Every cgp-serde provider, adapter type, and component, with what each handles, what it asks the context for, and the crate to import it from.'
---

# cgp-serde reference

This reference documents the public items of [cgp-serde](../index.md), one page per item. cgp-serde
rebuilds Serde's `Serialize` and `Deserialize` as components of [CGP](/docs/), so most of its items
are **providers**: implementations a **context**, the type whose wiring holds an application's
choices, names in its wiring table to say how a type is encoded. The tables below list every
provider by what it handles and what it asks the context for, which is what a context must also
wire.

:::tip

### New to CGP?

A provider page assumes the wiring that [`basic`](../examples/basic.md) walks through. For CGP
itself, the [Hello World tutorial](/docs/tutorials/hello) introduces contexts, providers, and
wiring, and the site's [reference](/docs/reference/) documents the CGP constructs these pages link
to.

:::

## Serialization providers

Every provider below implements the serializing component, the deserializing one, or both, and is
imported from its crate's `providers` module, such as `cgp_serde::providers::SerializeRecordFields`. The
last column is what the provider asks the context to encode in turn; see [re-entrant
providers](../architecture/reentrant-providers.md).

| Provider | Crate | Direction | Handles | Asks the context for |
|---|---|---|---|---|
| [`UseSerde`](./providers/use_serde.md) | `cgp-serde` | both | any type with Serde's traits | nothing |
| [`SerializeString`](./providers/serialize_string.md) | `cgp-serde` | both | writes any `AsRef<str>`; reads `String` | nothing |
| [`SerializeBytes`](./providers/serialize_bytes.md) | `cgp-serde` | both | writes any `AsRef<[u8]>`; reads any `From<&[u8]>` | nothing |
| [`TryDeserializeBytes`](./providers/try_deserialize_bytes.md) | `cgp-serde` | reads | any `TryFrom<&[u8]>` | nothing |
| [`SerializeWithDisplay`](./providers/serialize_with_display.md) | `cgp-serde` | writes | any `Display` type | `String` |
| [`DeserializeWithFromStr`](./providers/deserialize_with_from_str.md) | `cgp-serde` | reads | any `FromStr` type | `&str` |
| [`SerializeFrom`](./providers/serialize_from.md) | `cgp-serde` | both | a type converted with `Into` | the converted type |
| [`TrySerializeFrom`](./providers/try_serialize_from.md) | `cgp-serde` | both | a type converted with `TryInto` | the converted type |
| [`SerializeDeref`](./providers/serialize_deref.md) | `cgp-serde` | writes | a reference or smart pointer | the value it points to |
| [`SerializeIterator`](./providers/serialize_iterator.md) | `cgp-serde` | writes | a collection iterable by reference | each item it yields |
| [`DeserializeExtend`](./providers/deserialize_extend.md) | `cgp-serde` | reads | a collection that can be extended | each item |
| [`SerializeRecordFields`](./providers/serialize_record_fields.md) | `cgp-serde` | writes | a struct with named fields | each field's type |
| [`DeserializeRecordFields`](./providers/deserialize_record_fields.md) | `cgp-serde` | reads | a struct with named fields | each field's type |
| [`DeserializeDefault`](./providers/deserialize_default.md) | `cgp-serde` | reads | a `Default` type that may be null | nothing; wraps another provider |
| [`SerializeHex`](./providers/serialize_hex.md) | `cgp-serde-extra` | both | bytes, as hex | `String` |
| [`SerializeBase64`](./providers/serialize_base64.md) | `cgp-serde-extra` | both | bytes, as base64 | `String` |
| [`SerializeRfc3339Date`](./providers/serialize_rfc3339_date.md) | `cgp-serde-extra` | both | `DateTime<Utc>`, as an RFC 3339 string | `String` |
| [`SerializeTimestamp`](./providers/serialize_timestamp.md) | `cgp-serde-extra` | both | `DateTime<Utc>`, as a Unix timestamp | `i64` |
| [`DeserializeAndAllocate`](./providers/deserialize_and_allocate.md) | `cgp-serde-alloc` | reads | a reference, allocated through the context | the owned value, and an allocator |

## JSON and allocation providers

Four more providers implement other components. The JSON providers make writing and reading JSON an
operation the context runs, and the arena provider implements allocation:

| Provider | Crate | What it does | Asks the context for |
|---|---|---|---|
| [`SerializeToJsonString`](./providers/serialize_to_json_string.md) | `cgp-serde-json` | writes a value as a compact JSON string | the value's serializer, and an error type |
| [`DeserializeFromJsonReader`](./providers/deserialize_from_json_reader.md) | `cgp-serde-json` | reads a value from any `serde_json` reader | the value's deserializer, and an error type |
| [`DeserializeFromJsonString`](./providers/deserialize_from_json_string.md) | `cgp-serde-json` | reads a value from a string | what its inner provider asks for |
| [`AllocateWithArena`](./providers/allocate_with_arena.md) | `cgp-serde-typed-arena` | allocates into an arena held by the context | the arena |

## Types and components

- [`SerializeWithContext`](./types/serialize_with_context.md) — pairs a context with a value as a
  Serde `Serialize` value, the way into any Serde format.
- [`DeserializeWithContext`](./types/deserialize_with_context.md) — pairs a context with a target
  type as a Serde `DeserializeSeed`.
- [`CanDeserializeJsonString`](./types/can_deserialize_json_string.md) — gives a context a
  `deserialize_json_string` method.
- [`CanAlloc`](./components/can_alloc.md) — the component a deserializer allocates through.

The pages for the two serialization components, `CanSerializeValue<Value>` and
`CanDeserializeValue<'de, Value>`, and for the arena getter `HasArena`, are still being written. The
[component design](../architecture/component-design.md) page explains the two components, and
[`AllocateWithArena`](./providers/allocate_with_arena.md) shows the arena getter in use.

## Looking for a name you don't see?

Some names belong to a construct documented under another name:

| Name | Where it is documented |
|---|---|
| `ValueSerializer`, `ValueSerializerComponent` | the provider trait and wiring key of `CanSerializeValue`; see [component design](../architecture/component-design.md) |
| `ValueDeserializer`, `ValueDeserializerComponent` | the provider trait and wiring key of `CanDeserializeValue`; see [component design](../architecture/component-design.md) |
| `SerializeJson`, `DeserializeJson` | the marker types that select the JSON operations; see [`SerializeToJsonString`](./providers/serialize_to_json_string.md) and [`DeserializeFromJsonReader`](./providers/deserialize_from_json_reader.md) |
| `Allocator`, `AllocatorComponent` | the provider trait and wiring key of [`CanAlloc`](./components/can_alloc.md) |
| `ArenaGetter`, `ArenaGetterComponent` | the provider trait and wiring key of `HasArena`; see [`AllocateWithArena`](./providers/allocate_with_arena.md) |
| `deserialize_json_string` | the method of [`CanDeserializeJsonString`](./types/can_deserialize_json_string.md) |

## Where to go next

- [Wiring a context](../guides/wiring-a-context.md): choosing providers for a context's table.
- [Writing a provider](../guides/writing-a-provider.md): adding a provider of your own.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
