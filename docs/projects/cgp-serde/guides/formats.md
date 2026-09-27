---
sidebar_label: 'Using a format'
sidebar_position: 3
description: 'How to use a cgp-serde context with serde_json, RON, and other Serde formats, which formats fit its output, and the entry point a format needs for reading.'
---

# Using a format

A [cgp-serde](../index.md) **context**, the type whose wiring holds an application's choices, works
with any Serde format through the format's ordinary `Serializer` and `Deserializer`, so using one is
a matter of handing the format a value that carries the context. cgp-serde rebuilds Serde's
`Serialize` and `Deserialize` as components of [CGP](/docs/). This guide shows how to write and read
with `serde_json` and another format, and which formats fit the library's output.

## Write with any format

**Wrap the context and the value in
[`SerializeWithContext`](../reference/types/serialize_with_context.md), and pass it wherever the
format expects a `Serialize` value.** The adapter implements Serde's own `Serialize`, so no format
needs changes:

```rust
let compact = serde_json::to_string(&SerializeWithContext::new(&app, &archive))?;
let pretty = serde_json::to_string_pretty(&SerializeWithContext::new(&app, &archive))?;
let ron = ron::to_string(&SerializeWithContext::new(&app, &archive))?;
```

Each call returns the format's own error, since the adapter involves none of CGP's error handling.
A context that wants writing JSON to be an operation that returns the context's own error type uses
[`SerializeToJsonString`](../reference/providers/serialize_to_json_string.md) through
`try_compute` instead, as [`basic`](../examples/basic.md) does; it writes compact JSON.

## Read with `serde_json`

Reading needs the context too, and `serde_json`'s `from_str` has no place for one, so cgp-serde
offers three ways in. The choice depends on where the input comes from and whether the result
borrows from it:

-  **`deserialize_json_string`**, a method on the context from
  [`CanDeserializeJsonString`](../reference/types/can_deserialize_json_string.md). It takes a `&str`
  and needs no `TryComputer` entries, only an error type. Annotate the result's type, as in
  `let payload: Payload = app.deserialize_json_string(&text)?;`, since the method takes no
  turbofish. The result cannot borrow from the string.
-  **[`DeserializeFromJsonReader`](../reference/providers/deserialize_from_json_reader.md)**, a
  provider wired under `@TryComputerComponent.<T> DeserializeJson<T>` and called with `try_compute`.
  It takes any `serde_json` reader: a `StrRead` for a string, a `SliceRead` for bytes, or an
  `IoRead` for a stream. A result can borrow from a string or byte input.
- **The seed directly.** Build a `serde_json::Deserializer`, drive a
  [`DeserializeWithContext`](../reference/types/deserialize_with_context.md) seed with it, and call
  `end` to reject anything after the value. This needs no error wiring at all.

The first two turn `serde_json`'s errors into the context's error type, so the context wires one;
see [wiring a context](./wiring-a-context.md#4-wire-an-error-type-when-the-json-providers-are-used).

## Read with another format

**A format without cgp-serde helpers needs only the few lines the seed takes.** Build the format's
deserializer, drive the seed with it, and check for trailing input if the format supports it. With
RON:

```rust
let mut deserializer = ron::Deserializer::from_str(&text)?;
let value: Rec = DeserializeWithContext::new(&app).deserialize(&mut deserializer)?;
deserializer.end()?;
```

`deserialize` here is Serde's `DeserializeSeed::deserialize`, so `serde::de::DeserializeSeed` must
be in scope. To make reading a format an operation on the context, write a provider on the model of
`DeserializeFromJsonReader`: it takes the format's input, deserializes through the context, and
turns the format's error into the context's error type.

## Choose a format that fits the output

**Self-describing formats work with cgp-serde's output as it is.** Two properties of that output
decide which formats fit:

- **Structs are maps.** The generic struct provider writes a struct as a map from field names to
  values, so JSON is unaffected, and RON writes `{"a":1}` rather than its struct syntax. Reading
  expects a map in return.
- **Lengths are not declared.** Structs and collections are started without a length, so a format
  that must write the length before the elements, such as postcard, rejects them.

For bytes in a text format, wire a text encoding such as
[`SerializeHex`](../reference/providers/serialize_hex.md) or
[`SerializeBase64`](../reference/providers/serialize_base64.md). A byte string written with Serde's
`serialize_bytes`, which [`SerializeBytes`](../reference/providers/serialize_bytes.md) uses, is an
array of numbers in JSON.

## Where to go next

- [The bridge to Serde](../architecture/serde-bridge.md): why the adapters are all a format needs.
- [Wiring a context](./wiring-a-context.md): the error wiring the JSON providers need.
- [Modular error handling](/docs/concepts/modular-error-handling): how a provider raises an error
  into the context's error type.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
