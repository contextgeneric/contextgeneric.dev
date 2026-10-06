---
sidebar_label: 'Compared with Serde'
sidebar_position: 5
description: 'cgp-serde and Serde compared: what they share, what cgp-serde adds, how Serde''s idioms map onto wiring, and when plain Serde is better.'
---

# cgp-serde compared with Serde

[cgp-serde](./index.md) is built on [Serde](https://serde.rs/) and depends on it, so this is not a
comparison between rival libraries. It compares two ways of implementing one layer of Serde: Serde's
own `Serialize` and `Deserialize` traits with their derive, and cgp-serde's
[CGP](/docs/) components with their providers. Serde is the mature, widely adopted library of
the two, and the foundation cgp-serde stands on. This page sets out what the two share, what
cgp-serde adds, how Serde's idioms map onto it, and when plain Serde is the better choice.

## What they share

Everything below the data-type layer is shared rather than compared. cgp-serde uses Serde's data
model, its `Serializer` and `Deserializer` traits, and every format built on them, and it reuses any
existing `Serialize` or `Deserialize` impl through [`UseSerde`](./reference/providers/use_serde.md).
A program can therefore use cgp-serde for some types and Serde's derive for the rest, in the same
value and the same format. [The bridge to Serde](./architecture/serde-bridge.md) explains how the
two meet.

## What cgp-serde adds

cgp-serde adds four things that Serde's trait design cannot express, each a consequence of moving
the encoded value out of `Self` and letting a **context**, a type that stands for an application,
choose its providers:

- **Encodings chosen per application.** Two contexts can encode the same type differently, such as
  `Vec<u8>` as hex in one application and base64 in another, with the choice reaching every nested
  occurrence. With Serde, a type has one `Serialize` impl in the program.
- **Implementations that overlap, and implementations for types a crate does not own.** Providers
  such as "any `Display` type, as a string" and "any `AsRef<[u8]>` type, as bytes" coexist, and a
  crate can provide an encoding for a type from another crate. Rust's coherence rules allow Serde
  neither.
- **Data types with no serialization derive.** A struct that derives CGP's general field traits is
  serialized by generic providers, so a crate of data types needs no `serde` dependency; see
  [derive-free records](./architecture/derive-free-records.md).
- **Services while deserializing.** A provider can take something from the context as it works, such
  as an arena to allocate into; see [context services](./architecture/context-services.md). Serde
  supports deserializing with state through hand-written `DeserializeSeed` impls, which its derive
  does not produce.

## How Serde's idioms map

Many of Serde's features exist to work within the one-impl-per-type rule, and cgp-serde meets the
same needs with wiring instead. The mapping is not one to one, because Serde's attributes are per
field while a cgp-serde context chooses per type:

| With Serde | With cgp-serde |
|---|---|
| `#[serde(with = "hex")]` on a field | wire the field's type to `SerializeHex` in the context; every field of that type follows |
| `serialize_with` and `deserialize_with` | write a provider, and wire the type to it |
| a newtype to change a type's encoding | wire a different provider in a different context |
| `#[serde(remote = "…")]` for a type from another crate | wire the type to a provider directly |
| `#[derive(Serialize, Deserialize)]` on a struct | derive `CgpData`, and wire the struct to `SerializeRecordFields` and `DeserializeRecordFields` |
| `#[derive(Serialize, Deserialize)]` on an enum whose variants hold one value or none | derive `CgpVariant`, wire the enum to `SerializeVariantFields` and `DeserializeVariantFields`, and wire `Nil` to `SerializeUnit` for the variants with no fields |
| a hand-written `DeserializeSeed` for state | a provider that takes the state from the context |
| a borrowed `&'de str` field | wire `&'a str` to `UseSerde`, and read from input that can lend it |

## What Serde's derive does that the generic providers do not

Serde's derive is the product of years of use, and it handles cases that cgp-serde's generic
providers leave to it:

- **Field and container attributes**, such as `rename`, `skip`, `flatten`, `default`, and
  `deny_unknown_fields`. cgp-serde's struct providers write every field under its Rust name and
  expect every field to be present.
- **Enum representations, tuple structs, and tuples.** Serde derives all of them, in several
  representations for enums. cgp-serde's providers handle structs with named fields, and enums
  whose variants each hold one value or none, in Serde's default externally tagged form. A variant
  with no fields is written as `{"Variant":null}`, where Serde writes a bare `"Variant"`, an empty
  sequence, or an empty map, depending on how the variant is declared.
- **Recursive types**, such as a tree whose nodes contain nodes. Serde derives them without
  difficulty; cgp-serde's generic providers cannot express the recursion, and such a type needs a
  provider written for it.
- **Binary formats that need lengths.** Serde's derived impls declare lengths, which formats such as
  postcard rely on. cgp-serde's generic providers do not, so their output suits self-describing
  formats such as JSON and RON.

Any of these types can still take part in a cgp-serde value: it keeps Serde's derive, and the
context wires it to `UseSerde`.

## What each approach costs

Serde's cost is the rule it lives under. A type has one encoding for the whole program, chosen by
the crate that owns the type, and a program that needs another has to wrap the type, annotate every
field that holds it, or ask the owner to change it. For most programs that cost is never felt,
because one encoding per type is all they want.

cgp-serde's cost is the wiring and CGP itself. Each application writes out a table naming a provider
for every type it encodes, including in-between types such as a `Vec` of structs, and a mistake in
the table is a compile error that can be long. Writing a provider means learning CGP's components
and wiring. The library is also a proof of concept, lightly tested and with none of Serde's history
of use. Provider selection is resolved at compile time, so wiring adds no lookup at run time, but no
benchmark compares the two.

## When plain Serde is the better choice

**Plain Serde is the right tool whenever the one-impl-per-type rule does not bind.** A program that
encodes each type one way needs no choice per context, and Serde's derive gives it attributes,
enums, binary formats, and a mature implementation with no wiring to write. It is also the right
choice for a type that needs the attributes or representations above, for binary formats, and for a
team with no other use for CGP, since cgp-serde's wiring and its errors carry CGP's learning cost.

cgp-serde earns its place where the rule does bind: the same types encoded differently by different
applications, encodings for types a crate does not own, data crates that should not depend on
`serde`, and deserializers that need something Serde's traits have no room for. Because the two
work together through `UseSerde` and the adapters, a program can use each for the types it suits.

## Where to go next

- [`messages`](./examples/messages.md): the per-application encoding in a running program.
- [Limitations](./limitations.md): where cgp-serde's design stops.
- [Coherence](/docs/concepts/coherence): the Rust rule behind Serde's one impl per type.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
