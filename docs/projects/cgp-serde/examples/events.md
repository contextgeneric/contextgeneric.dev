---
sidebar_label: 'events'
sidebar_position: 3
description: 'A cgp-serde example in which two applications write and read back a batch of chat events, an enum of records, each with its own encodings, and misread each other''s JSON.'
---

# Write and read back an enum of records in two applications

This example writes a batch of chat events to JSON and reads it back, under two applications that
encode bytes and dates differently, and then has each application read the other's JSON. It is an
example from [cgp-serde](../index.md), which rebuilds Serde's `Serialize` and `Deserialize` traits
as components of [CGP](/docs/). The events are an enum whose variants hold structs, and none of the
types derives anything from Serde.

:::tip

### New to CGP?

[`messages`](./messages.md) introduces two applications with their own wiring, which this page
extends to an enum and to reading. For CGP itself, the [Hello World tutorial](/docs/tutorials/hello)
shows contexts and wiring with a smaller program. This page can be read without them.

:::

## The problem

A chat client syncs a batch of events with a server. Each event is one of several kinds, a message
posted, edited, or reacted to, or the history cleared, and each kind carries its own data. The
server stores batches compactly, with bytes as base64 and dates as Unix timestamps. A debugging
inspector writes the same batches for people to read, with bytes as hex and dates as RFC 3339
strings. Both read batches back, and both use the same Rust types.

### Without CGP

With Serde, the event enum and its structs each have one `Serialize` and one `Deserialize` impl in
the whole program, so the encoding of their byte and date fields is fixed by `#[serde(with = …)]`
attributes on the types. Two applications that want different encodings need two copies of the
types, or wrappers and conversions around them. This page keeps one set of types and gives each
application its own choices, in both directions.

## Run it

From the root of the [cgp-serde repository](https://github.com/contextgeneric/cgp-serde), run the
example:

```sh
cargo run -p cgp-serde-examples --example events
```

It prints the batch as each application writes it, confirms that each reads its own JSON back into
the same batch, and prints the error each gives reading the other's. The first event, from the
server and then from the inspector, condensed onto one line each:

```json
{"Posted": {"message_id": 1, "author_id": 2, "date": 1762179300, "encrypted_data": "SGVsbG8="}}
{"Posted": {"message_id": 1, "author_id": 2, "date": "2025-11-03T14:15:00+00:00", "encrypted_data": "48656c6c6f"}}
```

`cargo test -p cgp-serde-examples --example events` checks both documents exactly, both round trips,
and both errors.

## An enum whose variants hold structs

The event is an enum deriving CGP's
[`CgpVariant`](/docs/reference/derives/derive_cgp_variant), and each variant holds a struct deriving
[`CgpData`](/docs/reference/derives/derive_cgp_data), from
[`events.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-examples/examples/events.rs):

```rust
#[derive(Debug, PartialEq, CgpData)]
pub struct Posted {
    pub message_id: u64,
    pub author_id: u64,
    pub date: DateTime<Utc>,
    pub encrypted_data: Vec<u8>,
}

#[derive(Debug, PartialEq, CgpVariant)]
pub enum ChatEvent {
    Posted(Posted),
    Edited(Edited),
    Reacted(Reacted),
    HistoryCleared,
}

#[derive(Debug, PartialEq, CgpData)]
pub struct SyncBatch {
    pub device_key: Vec<u8>,
    pub events: Vec<ChatEvent>,
}
```

`Edited` holds a message id, a date, and new encrypted data, and `Reacted` holds a message id, an
author id, and an emoji as a `String`, since many emoji, such as 👍🏽, are more than one `char`.
`HistoryCleared` carries no data, so it has no fields, and `CgpVariant` gives it the payload `Nil`,
CGP's empty value. The batch is a struct holding a `Vec` of events, so structs sit inside the enum
and the enum sits inside a struct.

## Each application wires both directions

Each application is a **context**, a type whose wiring holds the application's choices. The server's
table chooses a provider for every type the batch contains, once for writing and once for reading:

```rust
delegate_components! {
    ServerApp {
        open {
            ValueSerializerComponent,
            ValueDeserializerComponent,
        };

        @ValueSerializerComponent.<'a, T> &'a T: SerializeDeref,
        @ValueSerializerComponent.[i64, u64, String]: UseSerde,
        @ValueSerializerComponent.Nil: SerializeUnit,
        @ValueSerializerComponent.Vec<u8>: SerializeBase64,
        @ValueSerializerComponent.DateTime<Utc>: SerializeTimestamp,
        @ValueSerializerComponent.[Posted, Edited, Reacted, SyncBatch]: SerializeRecordFields,
        @ValueSerializerComponent.ChatEvent: SerializeVariantFields,
        @ValueSerializerComponent.Vec<ChatEvent>: SerializeIterator,

        @ValueDeserializerComponent.[i64, u64, String]: UseSerde,
        @ValueDeserializerComponent.Nil: SerializeUnit,
        @ValueDeserializerComponent.Vec<u8>: SerializeBase64,
        @ValueDeserializerComponent.DateTime<Utc>: SerializeTimestamp,
        @ValueDeserializerComponent.[Posted, Edited, Reacted, SyncBatch]: DeserializeRecordFields,
        @ValueDeserializerComponent.ChatEvent: DeserializeVariantFields,
        @ValueDeserializerComponent.Vec<ChatEvent>: DeserializeExtend,
    }
}
```

The source spreads each entry over two lines. The structs go to
[`SerializeRecordFields`](../reference/providers/serialize_record_fields.md) and
[`DeserializeRecordFields`](../reference/providers/deserialize_record_fields.md), and the enum to
[`SerializeVariantFields`](../reference/providers/serialize_variant_fields.md) and
[`DeserializeVariantFields`](../reference/providers/deserialize_variant_fields.md), which write
each event as `{"Variant": value}` and read it back. Each of them asks the context for the types
inside, so the encodings of `Vec<u8>` and `DateTime<Utc>` reach through the enum into every struct.
The `Nil` entry serves `HistoryCleared`:
[`SerializeUnit`](../reference/providers/serialize_unit.md) writes it as
`{"HistoryCleared": null}`, where Serde's derive would write the bare name `"HistoryCleared"`.

The inspector's table is the same except for three entries:
[`SerializeHex`](../reference/providers/serialize_hex.md) for `Vec<u8>`,
[`SerializeRfc3339Date`](../reference/providers/serialize_rfc3339_date.md) for `DateTime<Utc>`, and
no `i64` entry, which only the server's timestamp encoding needs. Each context checks every type
it reaches with [`check_components!`](/docs/reference/macros/check_components), in both directions.

## Reading the other application's JSON

Each application reads its own JSON back into a batch equal to the original. The dates in the batch
are whole seconds, since a Unix timestamp drops anything finer. **Reading the other application's
JSON fails, and the two directions fail differently.** The inspector reading the server's JSON fails
at the first field, because base64 is not valid hex:

```text
Invalid character 'Z' at position 0 at line 2 column 30
```

The server reading the inspector's JSON gets further. The inspector's hex device key,
`"6465766963652d37"`, uses only characters that base64 also allows, so the server decodes it as
base64, into the wrong bytes, without an error. It stops only at the first date, a string where it
expects a number:

```text
invalid type: string "2025-11-03T14:15:00+00:00", expected i64 at line 8 column 43
```

The JSON does not record which encoding wrote it, and neither do the types. Two programs exchanging
data therefore have to wire the same choices, and when they do not, a value can be misread without
any error.

## Try a change

Remove the server's `@ValueDeserializerComponent.Nil: SerializeUnit` entry, remove
`(Life<'de>, Nil)` from its check, and run [`cargo cgp check`](/docs/cargo-cgp/check). The check
fails on the three types that contain an event, and the tool names the missing entry as the root
cause:

```text
error[E0277]: [CGP-E001] the consumer traits `CanDeserializeValue<ChatEvent>`, `CanDeserializeValue<Vec<ChatEvent>>`, and `CanDeserializeValue<SyncBatch>` are not implemented for context `ServerApp`
    = note: root cause: [CGP-E107] context `ServerApp` does not contain any delegate entry for `@ValueDeserializerComponent.Nil`
```

`HistoryCleared` carries `Nil`, so reading an event asks the context to read `Nil`, even though no
struct has a `Nil` field. Put the entry back to fix it.

## The pattern

This example shows **the encoding of a whole data model, enums included, as a choice the context
makes in both directions**. The struct and enum providers walk the types through CGP's field and
variant traits, and hand every value inside back to the context, so one entry decides how a type is
written and read wherever it appears. The idea behind the enum providers is
[Extensible variants](/docs/concepts/extensible-variants), and
[Coherence](/docs/concepts/coherence) explains why plain Rust allows only one encoding per type.

The cost is the same as in [`messages`](./messages.md): each application lists every type its data
reaches, in each direction. The second half of this example shows a cost that belongs to any choice
of encoding rather than to CGP: the choice is not visible in the data, so two programs must agree on
it. With Serde's derive the agreement comes from sharing the types; here it comes from sharing the
wiring.

## Where to go next

- [`SerializeVariantFields`](../reference/providers/serialize_variant_fields.md) and
  [`DeserializeVariantFields`](../reference/providers/deserialize_variant_fields.md): the enum
  providers, with the limits on which enums they accept.
- [Wiring a context](../guides/wiring-a-context.md): how to find every type a traversal reaches, and
  how to check the table.
- [Derive-free records](../architecture/derive-free-records.md): serializing data through CGP's
  traits instead of a serialization derive.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
