---
sidebar_label: 'messages'
sidebar_position: 2
description: 'A cgp-serde test in which two application contexts encode one nested value differently, hex and RFC 3339 against base64 and Unix time.'
---

# Encode one value two ways in two applications

This test serializes one nested archive of messages twice, under two applications that disagree
about how bytes and dates should look. It is an example from [cgp-serde](../index.md), which
rebuilds Serde's `Serialize` and `Deserialize` traits as components of [CGP](/docs/). The data types
name no encoding, and the two applications differ in three wiring lines, which is the whole
difference between the two JSON documents they produce.

:::tip

### New to CGP?

[`basic`](./basic.md) introduces the wiring table this page builds on. For CGP itself, the [Hello
World tutorial](/docs/tutorials/hello) shows contexts and wiring with a smaller program, and
[Coherence](/docs/concepts/coherence) explains why plain Rust allows only one encoding per type,
which is what two applications here get around. This page can be read without them.

:::

## Run it

From the root of the [cgp-serde repository](https://github.com/contextgeneric/cgp-serde), run the
test and show what it prints:

```sh
cargo test -p cgp-serde-tests messages -- --nocapture
```

It prints the same archive twice. From the first application, `AppA`, bytes are hex and dates are
RFC 3339 strings:

```text
serialized with A: {
  "decryption_key": "746f702d736563726574",
  "messages_by_topics": [
    {
      "encrypted_topic": "416c6c2061626f757420434750",
      "messages": [
        {
          "message_id": 1,
          "author_id": 2,
          "date": "2025-11-03T14:15:00+00:00",
          "encrypted_data": "48656c6c6f2066726f6d20527573744c616221"
        },
        {
          "message_id": 4,
          "author_id": 8,
          "date": "2025-12-19T23:45:00+00:00",
          "encrypted_data": "4f6e65207965617220616e6e697665727361727921"
        }
      ]
    }
  ]
}
```

From the second, `AppB`, bytes are base64 and dates are Unix timestamps:

```text
serialized with B: {
  "decryption_key": "dG9wLXNlY3JldA==",
  "messages_by_topics": [
    {
      "encrypted_topic": "QWxsIGFib3V0IENHUA==",
      "messages": [
        {
          "message_id": 1,
          "author_id": 2,
          "date": 1762179300,
          "encrypted_data": "SGVsbG8gZnJvbSBSdXN0TGFiIQ=="
        },
        {
          "message_id": 4,
          "author_id": 8,
          "date": 1766187900,
          "encrypted_data": "T25lIHllYXIgYW5uaXZlcnNhcnkh"
        }
      ]
    }
  ]
}
```

The test prints the documents rather than asserting them, and passes when both serialize.

## The data names no encoding

The archive is three nested structs, from
[`messages.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-tests/src/tests/messages.rs):

```rust
#[derive(CgpData)]
pub struct EncryptedMessage {
    pub message_id: u64,
    pub author_id: u64,
    pub date: DateTime<Utc>,
    pub encrypted_data: Vec<u8>,
}

#[derive(CgpData)]
pub struct MessagesByTopic {
    pub encrypted_topic: Vec<u8>,
    pub messages: Vec<EncryptedMessage>,
}

#[derive(CgpData)]
pub struct MessagesArchive {
    pub decryption_key: Vec<u8>,
    pub messages_by_topics: Vec<MessagesByTopic>,
}
```

Byte fields appear at every level, and dates in the innermost struct. Nothing in the structs says
how either is written. [`CgpData`](/docs/reference/derives/derive_cgp_data) only exposes the fields
to generic code, and `Vec<u8>` and `DateTime<Utc>` belong to other crates, so in Serde their
encoding would be whatever those crates' `Serialize` impls chose.

## Each application is a context with its own table

`AppA` is a type that stands for the first application, which is where its choices live. It has no
fields, and its wiring table names a provider for each type the archive contains:

```rust
pub struct AppA;

delegate_components! {
    AppA {
        open {ValueSerializerComponent};

        @ValueSerializerComponent.<'a, T> &'a T:
            SerializeDeref,

        @ValueSerializerComponent.[
            u64,
            String,
        ]:
            UseSerde,

        @ValueSerializerComponent.Vec<u8>:
            SerializeHex,

        @ValueSerializerComponent.DateTime<Utc>:
            SerializeRfc3339Date,

        @ValueSerializerComponent.[
            Vec<EncryptedMessage>,
            Vec<MessagesByTopic>,
        ]:
            SerializeIterator,

        @ValueSerializerComponent.[
            MessagesArchive,
            MessagesByTopic,
            EncryptedMessage,
        ]:
            SerializeFields,
    }
}
```

The `open` statement lets the table choose by the type being serialized, and each `@` entry makes
one choice, as in [`basic`](./basic.md). A bracketed list gives several types the same provider.

`AppB` is a second context with the same table, except for three entries:

```rust
@ValueSerializerComponent.[
    i64,
    u64,
    String,
]:
    UseSerde,

@ValueSerializerComponent.Vec<u8>:
    SerializeBase64,

@ValueSerializerComponent.DateTime<Utc>:
    SerializeTimestamp,
```

[`SerializeHex`](../reference/providers/serialize_hex.md) and
[`SerializeBase64`](../reference/providers/serialize_base64.md) are two providers for the same type,
`Vec<u8>`, and so are [`SerializeRfc3339Date`](../reference/providers/serialize_rfc3339_date.md)
and [`SerializeTimestamp`](../reference/providers/serialize_timestamp.md) for `DateTime<Utc>`. As
Serde impls they would conflict, since a type can have only one. As entries in two contexts' tables
they do not, because each choice holds only within the context that makes it.
[Coherence](/docs/concepts/coherence) explains why that is allowed.

## A choice reaches every level of nesting

Neither table says anything about where a `Vec<u8>` appears, and yet every byte field in the
archive, at all three levels, follows its context's choice. That is because no provider encodes the
values inside the one it is given. [`SerializeFields`](../reference/providers/serialize_fields.md)
walks a struct and asks the context to serialize each field, and
[`SerializeIterator`](../reference/providers/serialize_iterator.md) walks a collection and asks the
context to serialize each item. Each of those requests goes back through the same table, so the
context's choice for a type applies wherever the type turns up. [Re-entrant
providers](../architecture/reentrant-providers.md) explains the mechanism.

## Entries the traversal needs

Because every nested value goes back through the table, the table needs an entry for every type the
traversal reaches, not only for the fields in the data. Four entries exist for that reason:

- **Each vector of structs** gets `SerializeIterator`. A `Vec<EncryptedMessage>` is a value the
  traversal reaches, so it needs an entry of its own besides the one for `EncryptedMessage`.
-  **The generic reference entry**, `<'a, T> &'a T`, sends every reference to
  [`SerializeDeref`](../reference/providers/serialize_deref.md), which serializes the value behind
  it. Walking a borrowed `Vec<EncryptedMessage>` yields `&EncryptedMessage`, and this one entry
  covers every such reference.
- **`String`** is wired, although no field is a `String`, because the hex, base64, and RFC 3339
  providers each produce a string and ask the context to serialize it.
-  **`i64`** is wired in `AppB`, although no field is an `i64`, because `SerializeTimestamp` turns
  the date into an `i64` and asks the context to serialize that.

The last point is the easiest to miss, and the change at the end of this page shows what happens
without it.

## Serializing through an ordinary Serde API

Each context's wiring is asserted by a
[`check_components!`](/docs/reference/macros/check_components) table listing the seven types it
serializes. Then the test hands each context and the archive to `serde_json`:

```rust
let serialized =
    serde_json::to_string_pretty(&SerializeWithContext::new(&AppA, &archive)).unwrap();
println!("serialized with A: {serialized}");
```

[`SerializeWithContext`](../reference/types/serialize_with_context.md) pairs a context with a value
and implements Serde's own `Serialize` trait, so any Serde format accepts it unchanged. Any error is
`serde_json`'s own, which is why neither context wires an error type.

## Try a change

Remove `i64` from `AppB`'s `UseSerde` entry, leaving `[u64, String]`, and run
[`cargo cgp check`](/docs/cargo-cgp/check). The check on `AppB` fails, and the tool names the
missing entry as the root cause:

```text
error[E0277]: [CGP-E001] the consumer traits `CanSerializeValue<DateTime<Utc>>`, `CanSerializeValue<EncryptedMessage>`, `CanSerializeValue<MessagesByTopic>`, and `CanSerializeValue<MessagesArchive>` are not implemented for context `AppB`
    = note: root cause: [CGP-E107] context `AppB` does not contain any delegate entry for `@ValueSerializerComponent.i64`
```

Every checked type that contains a date fails, and all for one reason. Below the root cause, the
tool prints the chain that leads to it: from `MessagesArchive` through `SerializeFields`, the
vectors, the references, and `EncryptedMessage`, down to `SerializeTimestamp`, which asks `AppB` for
an `i64` it has no entry for. The fix is to put `i64` back. `cargo cgp check` leads with the root
cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every
class.

## The pattern

This test shows **two applications choosing different implementations for the same types, without
a conflict**. Each application is a context, the serialized value is a parameter of the component,
and a context's table decides the provider for each value type, including types from other crates.
Because every provider hands nested values back to the context, one entry decides a type's encoding
everywhere in the value. The idea is CGP's local answer to Rust's coherence rule, explained in
[Coherence](/docs/concepts/coherence), and [Modularity
hierarchy](/docs/concepts/modularity-hierarchy#tier-4-one-provider-per-target-type-per-context)
places it among the alternatives.

The cost is the table each application writes. `AppA` and `AppB` share all but three entries and
still list every one, and both carry entries, such as the reference entry and `AppB`'s `i64`, that
serve the traversal rather than the data. The shared entries can move into a namespace the two
contexts join, as the last step of [wiring a
context](../guides/wiring-a-context.md#6-share-entries-between-contexts-with-a-namespace) shows, but
the traversal's entries are still there to write. A program that needs one encoding per type gets it
from Serde's derive with none of this.

## Where to go next

- [Architecture](../architecture/index.md): the design this example rests on, one idea per page.
- [Wiring a context](../guides/wiring-a-context.md): how to find every type a traversal reaches, and
  how to check the table.
- [Coherence](/docs/concepts/coherence): the Rust rule that gives a type one impl, and how CGP
  restores choice one context at a time.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
