---
sidebar_label: 'Component design'
sidebar_position: 2
description: 'Why cgp-serde moves the encoded value out of Self so the application chooses its encoding, one provider for both directions, and the reading lifetime.'
---

# Component design

Why does moving one type out of `Self` change who chooses how a value is encoded?
[cgp-serde](../index.md) rebuilds Serde's `Serialize` and `Deserialize` as components of
[CGP](/docs/), and its components differ from Serde's traits in exactly that move. This page
explains the move and what it makes possible, how one provider can serve both directions, and how
deserializing carries Serde's lifetime.

## Serde's trait belongs to the value

Serde's `Serialize` is implemented by the type being serialized. Its method takes the value as
`self`:

```rust
// Serde: implemented by the value's type
fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer;
```

So a type has one `Serialize` impl in the whole program, and Rust's coherence rules decide who may
write it: the crate that owns the type, or the crate that owns the trait. `Vec<u8>` is serialized as
a list of numbers because Serde's impl says so, and a program that wants hex has to wrap the type or
annotate every field.

## cgp-serde's belongs to the context

cgp-serde's serializing component, `CanSerializeValue<Value>`, keeps the method and adds one
parameter:

```rust
// cgp-serde: implemented by a context, for each Value
fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer;
```

The value is now an argument, and `self` is a **context**, a type that stands for an application and
holds its choices. The deserializing component, `CanDeserializeValue<'de, Value>`, makes the same
change to `Deserialize`. Each is a CGP [component](/docs/concepts/consumer-and-provider-traits): a
trait that callers use, paired with a trait that providers implement, joined by the context's
wiring.

The consequence is that each context chooses, for each value type, which provider serializes it:

```rust
delegate_components! {
    AppA {
        open {ValueSerializerComponent};

        @ValueSerializerComponent.Vec<u8>:
            SerializeHex,
        // ...
    }
}
```

A second context can wire `Vec<u8>` to `SerializeBase64`, and the two do not conflict, because
each impl is for a different context. Nor does either context need to own `Vec<u8>` or the
component, since owning the context is enough to wire it. [Coherence](/docs/concepts/coherence)
explains why that is allowed.

In the terms of the [modularity hierarchy](/docs/concepts/modularity-hierarchy), the contexts are
environmental, standing for applications rather than for data, and the serialized value is the
component's target. The hierarchy places this shape at the tier for a target whose behavior must
vary between contexts, and serialization is its motivating case: the types being encoded are usually
from other crates, and different applications need them encoded differently.

## One provider can serve both directions

A provider in CGP is a type that names an implementation, so one struct can implement both the
serializing and the deserializing component. cgp-serde does this wherever the two directions are one
decision. [`SerializeHex`](../reference/providers/serialize_hex.md) is one struct with two impls,
one that writes bytes as a hex string and one that reads a hex string back, and a context names it
once in each table:

```rust
@ValueSerializerComponent.Vec<u8>: SerializeHex,
@ValueDeserializerComponent.Vec<u8>: SerializeHex,
```

The names follow from this. A provider named `Serialize…` may implement both directions, while one
named `Deserialize…` implements only reading. Where the two directions work differently, they are
separate providers named for how they work: a struct is written by reading its fields, and read by
building it field by field. This table pairs each provider that writes with the one that reads the
same encoding:

| Writing | Reading |
|---|---|
| [`UseSerde`](../reference/providers/use_serde.md) | the same provider |
| [`SerializeString`](../reference/providers/serialize_string.md) | the same provider, for `String` |
| [`SerializeBytes`](../reference/providers/serialize_bytes.md) | the same provider, or [`TryDeserializeBytes`](../reference/providers/try_deserialize_bytes.md) for a fallible conversion |
| [`SerializeWithDisplay`](../reference/providers/serialize_with_display.md) | [`DeserializeWithFromStr`](../reference/providers/deserialize_with_from_str.md) |
| [`SerializeFrom`](../reference/providers/serialize_from.md) | the same provider |
| [`TrySerializeFrom`](../reference/providers/try_serialize_from.md) | the same provider |
| [`SerializeDeref`](../reference/providers/serialize_deref.md) | [`DeserializeAndAllocate`](../reference/providers/deserialize_and_allocate.md), for a reference |
| [`SerializeIterator`](../reference/providers/serialize_iterator.md) | [`DeserializeExtend`](../reference/providers/deserialize_extend.md) |
| [`SerializeRecordFields`](../reference/providers/serialize_record_fields.md) | [`DeserializeRecordFields`](../reference/providers/deserialize_record_fields.md) |
| [`SerializeVariantFields`](../reference/providers/serialize_variant_fields.md) | [`DeserializeVariantFields`](../reference/providers/deserialize_variant_fields.md) |
| [`SerializeUnit`](../reference/providers/serialize_unit.md) | the same provider, for a `Default` type |
| none | [`DeserializeDefault`](../reference/providers/deserialize_default.md), which wraps another reader |
| [`SerializeHex`](../reference/providers/serialize_hex.md), [`SerializeBase64`](../reference/providers/serialize_base64.md), [`SerializeRfc3339Date`](../reference/providers/serialize_rfc3339_date.md), [`SerializeTimestamp`](../reference/providers/serialize_timestamp.md) | the same providers |

Sharing one struct makes agreement the intent, and a context still wires each direction on its own.
Nothing stops a context from writing with one encoding and reading with another, so a round trip in
a test is what confirms that the two halves match.

## Deserializing carries Serde's lifetime

Serde's `Deserialize<'de>` has a lifetime, so that a value can borrow from its input, such as a
`&'de str` pointing into the JSON text. `CanDeserializeValue<'de, Value>` keeps it as a parameter of
the component. That makes the lifetime part of what a context is checked for: a check on the
deserializing component lists each type paired with the lifetime, written as
[`Life<'de>`](/docs/reference/types/life). This check is from [`basic`](../examples/basic.md):

```rust
check_components! {
    #[check_trait(CanDeserializeApp)]
    <'de> App {
        ValueDeserializerComponent: [
            (Life<'de>, u64),
            (Life<'de>, String),
            (Life<'de>, Vec<u8>),
            (Life<'de>, Payload),
        ]
    }
}
```

The serializing component has no lifetime, since writing a value never borrows from the output.

## What it costs

The move cannot be made to Serde's own traits without breaking every impl of them, so cgp-serde
defines new traits and connects them to Serde's, as [the bridge to Serde](./serde-bridge.md)
explains. Code that expects a `Serialize` value gets one through an adapter, not directly.

The larger cost is that choice has to be exercised. Where Serde finds a type's impl automatically, a
cgp-serde context names a provider for every type it encodes, in each direction it encodes it. The
table is the price of each application being able to choose, and for a program that would make the
same choice everywhere, it is a cost with no return.

## Where to go next

- [Re-entrant providers](./reentrant-providers.md): how a context's choice for a type reaches every
  place the type appears.
- [`basic`](../examples/basic.md): both components wired on one context, with `SerializeHex` in
  both directions.
- [Modularity hierarchy](/docs/concepts/modularity-hierarchy): the tiers of choice CGP offers, and
  when to move a value out of `Self`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
