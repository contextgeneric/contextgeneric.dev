---
sidebar_label: 'Wiring a context'
sidebar_position: 1
description: 'How to build a cgp-serde context''s wiring table: opening the components, finding every type a traversal reaches, writing keys, and checking the result.'
---

# Wiring a context

A cgp-serde context is a type that stands for an application, and its wiring table names a provider
for every type the application encodes. [cgp-serde](../index.md) rebuilds Serde's `Serialize` and
`Deserialize` as components of [CGP](/docs/). This guide gives the steps for building the table,
the key syntax some types need, and the checks that prove it complete.

## 1. Open the components and choose per type

Open each serialization component with the `open` statement, then write one entry per value type.
The statement comes first in the block:

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
            SerializeRecordFields,
    }
}
```

This is the table of `AppA` from the [`messages`](../examples/messages.md) example. Each `@` entry
reads "to serialize this type, use this provider", and a bracketed list gives several types one
provider. [`delegate_components!`](/docs/reference/macros/delegate_components) documents the syntax
in full.

Open both components when the context reads as well as writes, as in
`open { ValueSerializerComponent, ValueDeserializerComponent };`, and write the reading entries
under `@ValueDeserializerComponent`. A context that runs the JSON providers through `try_compute`
opens `TryComputerComponent` too, as [`basic`](../examples/basic.md) does.

## 2. Wire every type the traversal reaches

List the types a traversal touches, not only the types the data names. Start from each top-level
type, and follow what each provider asks the context for, which each provider's
[reference page](../reference/index.md) states:

- **A struct** wired to [`SerializeRecordFields`](../reference/providers/serialize_record_fields.md)
  or [`DeserializeRecordFields`](../reference/providers/deserialize_record_fields.md) needs an entry
  for each field's type.
- **A collection** needs an entry of its own, besides its item type's. Writing one with
  [`SerializeIterator`](../reference/providers/serialize_iterator.md) also asks for the item as a
  reference, which one generic entry covers:
  `@ValueSerializerComponent.<'a, T> &'a T: SerializeDeref`.
- **An encoding** asks for the type it produces. The hex, base64, and RFC 3339 providers ask for
  `String`, and [`SerializeTimestamp`](../reference/providers/serialize_timestamp.md) asks for
  `i64`, so a context wires those types even when no field has them.
- **A leaf**, a type the application does not want to customize, goes to
  [`UseSerde`](../reference/providers/use_serde.md), which uses the type's own Serde impl and asks
  for nothing.

A type the table misses is a compile error rather than a runtime failure, and [debugging the
wiring](./debugging-wiring.md) shows what each kind of omission reports.

## 3. Write keys for references, lifetimes, and arrays

Three kinds of type need care in a key:

- **References and borrowed types** declare their lifetime in the entry:
  `@ValueSerializerComponent.<'a, T> &'a T`, `@ValueDeserializerComponent.<'a> &'a str`, or
  `@ValueDeserializerComponent.<'b> Vec<&'b Coord>`. Inside a list, each type declares its own, as
  in `[Coord, <'b> Payload<'b>]`. A key with a fixed lifetime such as `&'static str` also works, but
  matches only that lifetime.
- **A context with a lifetime** declares it on the table, as in `<'a> App<'a> { … }`, and the
  entries use lifetime names of their own.
- **A fixed-size array** cannot be written as a key, because square brackets are the syntax for a
  list of keys, so `@ValueDeserializerComponent.[u8; 32]` fails to parse. Define a type alias, such
  as `type Digest = [u8; 32];`, and key on the alias.

## 4. Wire an error type when the JSON providers are used

The JSON providers and the `deserialize_json_string` method turn `serde_json`'s errors into the
context's error type, so a context that uses them wires one. With the `anyhow` backend from the
[`cgp-error-anyhow`](https://docs.rs/cgp-error-anyhow) crate, that is two plain entries:

```rust
ErrorTypeProviderComponent:
    UseAnyhowError,

ErrorRaiserComponent:
    RaiseAnyhowError,
```

The components are imported from `cgp::core::error`, and the providers from `cgp_error_anyhow`.
[Modular error handling](/docs/concepts/modular-error-handling) explains the two components. A
context that only serializes through
[`SerializeWithContext`](../reference/types/serialize_with_context.md) needs neither, since the
format's own error comes back.

## 5. Check every value type

Assert the wiring with a [`check_components!`](/docs/reference/macros/check_components) table that
lists every type the context encodes. CGP checks wiring when it is used, so without a check a
missing entry surfaces only at the first call, with a longer error. List the reading side with the
component's lifetime, written as [`Life<'de>`](/docs/reference/types/life):

```rust
check_components! {
    #[check_trait(CanUseAppSerializer)]
    App {
        ValueSerializerComponent: [
            u64,
            String,
            Vec<u8>,
            Payload,
        ],
    }
}

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

Give each table its own `#[check_trait]` name when one module checks the same context more than
once, since both would otherwise take the same default name. Prefer a separate `check_components!`
to [`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components), which
checks only plain entries and leaves every `@` entry under an `open` statement unchecked.

## 6. Share entries between contexts with a namespace

Two applications that make most of the same choices can put the shared entries in a
[namespace](/docs/concepts/namespaces), a reusable wiring table that contexts join, and keep only
their differences in their own tables. This is `AppA` and `AppB` from `messages`, reduced to two of
their structs, with the shared entries moved into a namespace:

```rust
cgp_namespace! {
    new MessagesNamespace {
        @ValueSerializerComponent.<'a, T> &'a T:
            SerializeDeref,
        @ValueSerializerComponent.[i64, u64, String]:
            UseSerde,
        @ValueSerializerComponent.Vec<EncryptedMessage>:
            SerializeIterator,
        @ValueSerializerComponent.[MessagesByTopic, EncryptedMessage]:
            SerializeRecordFields,
    }
}

pub struct AppA;

delegate_components! {
    AppA {
        namespace MessagesNamespace;
        open ValueSerializerComponent;

        @ValueSerializerComponent.Vec<u8>: SerializeHex,
        @ValueSerializerComponent.DateTime<Utc>: SerializeRfc3339Date,
    }
}

pub struct AppB;

delegate_components! {
    AppB {
        namespace MessagesNamespace;
        open ValueSerializerComponent;

        @ValueSerializerComponent.Vec<u8>: SerializeBase64,
        @ValueSerializerComponent.DateTime<Utc>: SerializeTimestamp,
    }
}
```

Each context still opens the component, and a context's own entries only fill what the namespace
leaves out: a context cannot replace an entry the namespace makes. So put in the namespace only the
choices every joining context shares, and leave each point of difference, here the bytes and the
dates, to the contexts. [`cgp_namespace!`](/docs/reference/macros/cgp_namespace) documents the
syntax.

## Where to go next

- [Debugging the wiring](./debugging-wiring.md): what each kind of mistake reports.
- [Re-entrant providers](../architecture/reentrant-providers.md): why the traversal asks for the
  types it does.
- [Checking your wiring](/docs/concepts/check-traits): why CGP checks wiring lazily, and what a
  check forces.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
