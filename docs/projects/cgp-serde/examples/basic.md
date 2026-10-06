---
sidebar_label: 'basic'
sidebar_position: 1
description: 'A cgp-serde example that writes a struct to JSON and reads it back, with no serialization derive and each field type''s encoding chosen by the context.'
---

# Write a struct to JSON and read it back

This example writes a struct to a JSON string and reads it back, through a **context**, the type
whose wiring holds the application's choices, which decides how each of the struct's field types is
encoded. It is an example from [cgp-serde](../index.md), which rebuilds Serde's `Serialize` and
`Deserialize` traits as components of [CGP](/docs/). The struct derives nothing from Serde, and its
bytes come out as hex because one wiring line says so.

:::tip

### New to CGP?

The [Hello World tutorial](/docs/tutorials/hello) introduces contexts, providers, and wiring with a
smaller program. [Coherence](/docs/concepts/coherence) explains why Rust gives a type only one
`Serialize` impl, which is the limit this library removes. This page can be read without either.

:::

## The problem

The task is ordinary: write a struct to JSON and read it back, with its byte field written as a hex
string rather than as a list of numbers. Two requirements make it more than that. The struct should
derive nothing from Serde, so the crate that defines it need not depend on `serde`. And the byte
encoding should be chosen by the application that serializes the struct, not fixed where the struct
is defined.

### Without CGP

With Serde, the struct derives `Serialize` and `Deserialize`, and the byte field carries a
`#[serde(with = …)]` attribute naming a helper module that writes and reads hex. For one program
that encodes the struct one way, that is short, clear, and the right tool. Its costs are the two the
requirements rule out: the struct's crate depends on `serde`, and the hex encoding is written into
the struct's definition, so an application that wants base64 needs a different struct. This page
moves both decisions out of the struct and into the application's wiring.

## Run it

From the root of the [cgp-serde repository](https://github.com/contextgeneric/cgp-serde):

```sh
cargo run -p cgp-serde-examples --example basic
```

It prints the JSON the struct serializes to, and the value read back from it:

```text
serialized: {"quantity":42,"message":"hello","data":"010203"}
deserialized: Payload { quantity: 42, message: "hello", data: [1, 2, 3] }
```

`cargo test -p cgp-serde-examples --example basic` checks both: the exact JSON, and that reading it
back gives the original value.

## The struct derives nothing from Serde

The data type is an ordinary struct, from
[`basic.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-examples/examples/basic.rs):

```rust
#[derive(Debug, Eq, PartialEq, CgpData)]
pub struct Payload {
    pub quantity: u64,
    pub message: String,
    pub data: Vec<u8>,
}
```

`Debug`, `Eq`, and `PartialEq` are there to print the value and for the example's test. The only
other derive is CGP's [`CgpData`](/docs/reference/derives/derive_cgp_data), which exposes the
struct's field names and types to generic code and lets generic code build the struct one field at a
time. It is not specific to serialization, and nothing in the struct says how any field is encoded.
cgp-serde's record providers read what `CgpData` exposes, as [derive-free
records](../architecture/derive-free-records.md) explains.

## One context holds every choice

The example runs on `App`, a type that stands for this application, which is where its choices live.
It has no fields, because every choice it makes is in its wiring:

```rust
pub struct App;

delegate_components! {
    App {
        open {
            ValueSerializerComponent,
            ValueDeserializerComponent,
            TryComputerComponent,
        };

        ErrorTypeProviderComponent:
            UseAnyhowError,

        ErrorRaiserComponent:
            RaiseAnyhowError,

        @ValueSerializerComponent.u64:
            UseSerde,

        @ValueSerializerComponent.String:
            SerializeString,

        @ValueSerializerComponent.Vec<u8>:
            SerializeHex,

        @ValueSerializerComponent.Payload:
            SerializeRecordFields,

        @ValueDeserializerComponent.[
            u64,
            String,
        ]:
            UseSerde,

        @ValueDeserializerComponent.Payload:
            DeserializeRecordFields,

        @ValueDeserializerComponent.Vec<u8>: SerializeHex,

        @TryComputerComponent.SerializeJson:
            SerializeToJsonString,

        @TryComputerComponent.<T> DeserializeJson<T>:
            DeserializeFromJsonString,
    }
}
```

[`delegate_components!`](/docs/reference/macros/delegate_components) writes the wiring table. The
`open` statement at the top lets the table choose a different provider for each value type, and
each `@` entry makes one choice. `@ValueSerializerComponent.u64: UseSerde` reads "to serialize a
`u64`, use `UseSerde`". `ValueSerializerComponent` and `ValueDeserializerComponent` are cgp-serde's
two components, its versions of `Serialize` and `Deserialize`.

The value being encoded is a parameter of the component, not the type that implements it. That is
the difference from Serde, where `Payload` itself would implement `Serialize`. Here `App` implements
serialization *for* `Payload`, `u64`, and the rest, so a second application could wire the same
types to different providers without either conflicting with the other.

The providers each do one job:

- [`UseSerde`](../reference/providers/use_serde.md) uses the type's own Serde impl, which suits
  `u64`.
- [`SerializeString`](../reference/providers/serialize_string.md) writes anything string-like as a
  string.
- [`SerializeHex`](../reference/providers/serialize_hex.md) writes bytes as a hex string, and reads
  them back.
- [`SerializeRecordFields`](../reference/providers/serialize_record_fields.md) writes a struct as a
  map from field names to values, and
  [`DeserializeRecordFields`](../reference/providers/deserialize_record_fields.md) reads one back.

The two directions are wired separately, so they can name different providers. `String` is written
by `SerializeString` and read by `UseSerde`, which agree on the JSON. `Vec<u8>` names `SerializeHex`
in both directions, because one struct implements both halves of the hex encoding.

## The struct's provider asks the context for its fields

`SerializeRecordFields` knows how to walk a struct, but not how to encode any field. For each field,
it asks the context to serialize the field's type, and the context's wiring answers. That is why
`App` has entries for `u64`, `String`, and `Vec<u8>` as well as for `Payload`: they are the types
the struct's provider hands back.

The same handing back is what lets a choice reach every level of a nested value. Wire `Vec<u8>` to
`SerializeHex`, and every `Vec<u8>` the context serializes is hex, wherever it sits.
[Re-entrant providers](../architecture/reentrant-providers.md) explains the mechanism, and
[`messages`](./messages.md) shows it through three levels of nesting.

## JSON is an operation the context runs

The last two entries make JSON encoding something `App` can do. They are keyed on CGP's
[`TryComputer`](/docs/reference/components/handler/try_computer), an interface for a fallible
computation chosen by a marker type, and the markers `SerializeJson` and `DeserializeJson<T>` pick
the JSON providers. The example calls both through `try_compute`:

```rust
let serialized = context
    .try_compute(PhantomData::<SerializeJson>, &payload())
    .unwrap();

let deserialized: Payload = context
    .try_compute(PhantomData::<DeserializeJson<Payload>>, &serialized)
    .unwrap();
```

The generic `<T> DeserializeJson<T>` entry sends every target type to one provider. The providers
run `serde_json` through the context's serialization wiring, so the JSON follows every choice above.

A JSON error needs an error type to become, and the two error entries supply it. They make the
context's error type `anyhow::Error`, using the providers of the `cgp-error-anyhow` crate, so the
calls return `anyhow::Error` with `serde_json`'s message inside. With the `data` field removed from
the input, reading it back fails with:

```text
missing field: data at line 1 column 33
```

The providers do not name an error type of their own; the context chooses it. [Modular error
handling](/docs/concepts/modular-error-handling) explains how.

## Checking both directions

Wiring is checked when it is used, so the example asserts that every entry resolves with
[`check_components!`](/docs/reference/macros/check_components), once per direction:

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

The deserializing component carries Serde's `'de` lifetime as a parameter, so its entries pair each
type with the lifetime, written as [`Life<'de>`](/docs/reference/types/life). Each table has its
own `#[check_trait]` name, because two checks of one context in one module would otherwise derive
the same name.

## Try a change

Change the encoding of the bytes from hex to base64 by replacing `SerializeHex` with
`SerializeBase64` in both `Vec<u8>` entries, and importing it from `cgp_serde_extra::providers`.
Nothing else changes. Run the example's test again, and its first assertion fails, printing the new
JSON:

```text
assertion `left == right` failed
  left: "{\"quantity\":42,\"message\":\"hello\",\"data\":\"AQID\"}"
 right: "{\"quantity\":42,\"message\":\"hello\",\"data\":\"010203\"}"
```

The bytes `[1, 2, 3]` are now `"AQID"`. Update the expected string to match, and the test passes
again, reading the base64 back into the same `Payload`. The struct and the calls were not touched;
the encoding lived only in the wiring.

## The pattern

This example shows **serialization as a choice made by the context, one type at a time**. The
component takes the encoded value as a parameter, so the context, not the data type, chooses the
provider for each value type, and a provider for a struct asks the context about the struct's
fields. The same move makes the error type the context's choice too. [Modularity
hierarchy](/docs/concepts/modularity-hierarchy#tier-4-one-provider-per-target-type-per-context)
places this shape among CGP's options, as the one for a target type whose behavior must vary between
contexts.

The cost is the table. Every type the context encodes needs an entry in each direction, and the two
directions are wired separately, so nothing but a round trip checks that they agree. For a program
that encodes each type one way, Serde's derive is shorter.

## Where to go next

- [`messages`](./messages.md): the next example, two applications encoding one value differently.
- [Wiring a context](../guides/wiring-a-context.md): how to build and check a table like `App`'s.
- [Coherence](/docs/concepts/coherence): why moving the value out of `Self` lets implementations
  overlap.
- [Announcing cgp-serde](/blog/cgp-serde-release#derive-free-serialization-with-derivecgpdata): the
  post that announced the library, including derive-free serialization. Its code predates the
  current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
