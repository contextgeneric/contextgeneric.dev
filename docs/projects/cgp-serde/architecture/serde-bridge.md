---
sidebar_label: 'The bridge to Serde'
sidebar_position: 1
description: 'Which layer of Serde cgp-serde replaces and which it keeps, how values cross between cgp-serde and ordinary Serde code, and where errors are reported.'
---

# The bridge to Serde

How can a library replace `Serialize` and still work with `serde_json`, and with every type that
already implements `Serialize`? [cgp-serde](../index.md) rebuilds Serde's data-type traits as
components of [CGP](/docs/), and keeps the rest of Serde as it is. This page explains which layer
it replaces, how values cross between cgp-serde and ordinary Serde code in each direction, and where
errors are reported.

## Serde has two sides, and cgp-serde replaces one

Serde separates what a value is from how a format writes it. On one side, formats such as
`serde_json` and RON implement the `Serializer` and `Deserializer` traits, which can write and read
a small set of shapes: strings, numbers, byte arrays, sequences, maps, and a few more. On the other
side, data types implement `Serialize` and `Deserialize`, which describe a value in those shapes.
The shapes are the contract between the two sides, and Serde calls them its data model.

cgp-serde replaces the data-type side only. Its two components take the place of `Serialize` and
`Deserialize`, and its providers receive an ordinary Serde `Serializer` or `Deserializer` and call
the same methods a hand-written impl would, such as `serialize_str` or `serialize_map`. So a format
needs no changes to work with cgp-serde, and cgp-serde has no code for any particular format.

## Going in: using a type's own Serde impl

The bridge carries values in two directions. Going in, the provider
[`UseSerde`](../reference/providers/use_serde.md) lets a **context**, the type whose wiring holds an
application's choices, use a type's existing Serde impl. Its serializing half is one line of work:

```rust
#[cgp_impl(UseSerde)]
impl<Value> ValueSerializer<Value>
where
    Value: SerdeSerialize,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        value.serialize(serializer)
    }
}
```

`SerdeSerialize` is Serde's `Serialize`, imported under another name. Every `Serialize` impl, in the
standard library or in any crate, is therefore available to a context without a cgp-serde provider
being written for it; the context wires the type to `UseSerde`. This is also how a context handles a
type that cgp-serde's generic providers do not cover, such as an enum with a derived impl.

The trade is that the context's choices stop at that value. The type's own impl serializes its
contents, so a `Vec<u8>` inside it is written the way that impl writes it, whatever the context
wires for `Vec<u8>`.

## Going out: handing a context to a Serde API

Going out, two adapter types let a context and a value enter any Serde API.
[`SerializeWithContext`](../reference/types/serialize_with_context.md) pairs a context with a value
and implements Serde's `Serialize`, so it can be passed to `serde_json::to_string` or any other
format's entry point:

```rust
let serialized =
    serde_json::to_string_pretty(&SerializeWithContext::new(&AppA, &archive)).unwrap();
```

[`DeserializeWithContext`](../reference/types/deserialize_with_context.md) does the same for
reading. It implements Serde's `DeserializeSeed`, the trait for deserializing with state, where the
state is the context. Serde's convenience functions such as `serde_json::from_str` accept only a
type that implements `Deserialize`, not a seed, so a program builds the format's deserializer itself
and drives the seed with it. The [JSON
providers](../reference/providers/deserialize_from_json_reader.md) do this for `serde_json`, and
[using a format](../guides/formats.md) shows the few lines another format needs.

The same two adapters work inside cgp-serde. When a provider hands a nested value to one of Serde's
compound writers or readers, it wraps the value in an adapter, which is how the context's choices
reach nested values; see [re-entrant providers](./reentrant-providers.md).

## Where errors are reported

Errors are reported at two levels. Inside a serialization, a provider reports a failure through the
format's own error type, with Serde's `Error::custom`, exactly as a hand-written Serde impl does. An
invalid hex digit or a missing struct field is reported this way, and the format adds its position
to the message.

At the edge, the JSON providers turn the finished `serde_json::Error` into the context's own error
type, using CGP's [modular error handling](/docs/concepts/modular-error-handling), so a caller gets
whatever error type the context wires, such as `anyhow::Error`. A program that calls `serde_json`
through the adapters directly skips this level and gets `serde_json`'s error unchanged.

## What the output looks like to a format

Because cgp-serde writes values through Serde's ordinary methods, a format sees the shapes its
providers choose, and two of those choices affect which formats accept the output:

- **Structs are written as maps.** The generic struct provider writes a map from field names to
  values, where Serde's derive writes a struct. JSON writes both the same way. A format with its own
  struct syntax, such as RON, writes a map.
- **Lengths are not declared.** The generic struct and collection providers start a map or sequence
  without saying how long it is. Self-describing formats such as JSON and RON do not need to know,
  but a length-prefixed binary format such as postcard does, and rejects the output.

## What it costs

Keeping Serde's formats means keeping Serde's interface for them, and that interface was designed
for stateless types. Serializing needs only the adapter, but deserializing through a context needs a
seed, which Serde's one-line helpers do not accept, so every format gets a few lines of entry-point
code, written once. And a value handed to `UseSerde` is outside the context's reach, so the boundary
between the two layers is a decision a context makes type by type.

The benefit is that adoption can be partial. A program can use cgp-serde for the types whose
encoding it wants to choose, and Serde's derive for the rest, in the same value and the same format.

## Where to go next

- [Component design](./component-design.md): the two components that replace `Serialize` and
  `Deserialize`.
- [Using a format](../guides/formats.md): serializing and deserializing with `serde_json` and other
  formats.
- [`messages`](../examples/messages.md): the adapter handing two contexts to `serde_json`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
