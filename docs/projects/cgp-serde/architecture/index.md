---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'How cgp-serde works, in outline: Serde''s data-type layer replaced by components, the encoded value moved out of Self, and providers that ask the context.'
---

# How cgp-serde works

How can one type be serialized two different ways in two applications, when Rust allows it only one
`Serialize` impl? [cgp-serde](../index.md) answers by rebuilding Serde's `Serialize` and
`Deserialize` as components of [CGP](/docs/), a language extension for Rust with pluggable trait
implementations at compile-time. This page explains the design in outline: which part of Serde it
replaces, what decides a type's encoding, and how that decision reaches every value nested inside
another. Each idea has a page of its own with the details.

## It replaces one layer of Serde

Serde has two sides. Formats such as `serde_json` implement its `Serializer` and `Deserializer`
traits, which know how to write strings, numbers, maps, and sequences. Data types implement its
`Serialize` and `Deserialize` traits, which decide how a value is described in those terms.

cgp-serde replaces only the data-type side. Its implementations call the same format methods a
hand-written `Serialize` impl would, so every Serde format works with it unchanged, and any type
that already implements `Serialize` can keep using that impl. The [bridge to
Serde](./serde-bridge.md) page explains how values cross between the two in each direction.

## The application decides how a type is encoded

In Serde, `Serialize` is implemented by the type being serialized, so the type gets one encoding for
the whole program. cgp-serde's version turns this around. Its serializing component takes the value
as a parameter, and it is implemented by a **context**, a type that stands for an application and
holds that application's choices:

```rust
pub struct AppA;

delegate_components! {
    AppA {
        open {ValueSerializerComponent};

        @ValueSerializerComponent.Vec<u8>: SerializeHex,
        // ...
    }
}
```

The context's wiring table names an implementation, called a **provider**, for each value type. So
`AppA` can write `Vec<u8>` as hex while another context writes it as base64, and neither needs to
own `Vec<u8>`. The compiler resolves each choice while it type-checks the program, so there is no
lookup at run time. The [component design](./component-design.md) page explains the move and why
one provider can serve both directions.

## Providers hand nested values back to the context

A provider for a struct or a collection does not encode the values inside it. It asks the context to
encode each one, and the context's table chooses the provider for that value's type. That is how
`AppA`'s choice for `Vec<u8>` reaches a byte field three structs deep, without any provider on the
way knowing about it. The [re-entrant providers](./reentrant-providers.md) page explains the two
ways a provider does this, and what it asks of the context's table.

## A struct needs only CGP's field derive

A struct becomes serializable by deriving CGP's general-purpose field traits, with
[`CgpData`](/docs/reference/derives/derive_cgp_data), and nothing from Serde. Two generic providers
read the field list those traits expose, one to write a struct and one to read it back. So a crate
can define data types without depending on `serde`, and each application decides how to encode them.
The [derive-free records](./derive-free-records.md) page explains what that gives and what it gives
up.

## A provider can draw on its context

Every provider method receives the context, so a provider can use anything the context offers. A
deserializer that needs somewhere to allocate borrowed values can take an arena from the context,
which Serde's `Deserialize` has no way to receive. The [context services](./context-services.md)
page explains this, and how the allocator itself becomes a wiring choice.

## Each dependency lives in its own crate

The core crate depends only on CGP and Serde. The hex, base64, and date encodings, the JSON helpers,
and the arena allocator each live in a crate of their own, so an application compiles only the
libraries its wiring names. The [crate layout](./crate-layout.md) page lists them.

## What the design costs

The flexibility is paid for in wiring. An application writes out a table naming a provider for every
type it encodes, including types that exist only along the way, such as a `Vec` of structs or the
string a hex encoding produces. A type the table misses is a compile error, and the error can be
long, since the compiler reports the whole path from the top-level value down to the missing entry.

It also sets aside what Serde's derive does per field. The generic struct providers know a field's
name and type and nothing else, so there are no attributes for renaming or skipping a field; a
choice is made per type, for every field of that type.

For a program that encodes each type one way, Serde's derive is simpler and more complete. The
design earns its cost when different applications need different encodings of the same types, when
the types belong to crates that should not depend on `serde`, or when deserializing needs something
from its surroundings. The [comparison with Serde](../serde-comparison.md) sets the two side by
side.

## Where to go next

- [The bridge to Serde](./serde-bridge.md): which part of Serde cgp-serde replaces, and how it
  connects to the rest.
- [`messages`](../examples/messages.md): the design in a running program, two applications and one
  value.
- [Modularity hierarchy](/docs/concepts/modularity-hierarchy): the CGP idea of moving a value out of
  `Self`, with smaller examples.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
