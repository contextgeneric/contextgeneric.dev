---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'cgp-serde rebuilds Serde''s Serialize and Deserialize as CGP components, so each application chooses how a type is encoded, with no conflicting impls.'
---

# cgp-serde

cgp-serde rebuilds [Serde](https://serde.rs/)'s `Serialize` and `Deserialize` traits as components
of [CGP](/docs/), a language extension for Rust with pluggable trait implementations at
compile-time. With Serde, a type has one encoding for the whole program, chosen by whoever owns the
type. With cgp-serde, the encoding of each type is chosen by the application instead, so two
applications can write the same value differently, and neither needs to own the type or wrap it.

cgp-serde is a proof of concept. It replaces Serde's per-type implementations and keeps everything
else: Serde's data model, its formats such as `serde_json`, and any `Serialize` impl a type already
has. Its generic providers cover structs with named fields and the standard collections, and
scalars, enums, and every other type are encoded through their own Serde impls. These pages show
what the design makes possible and where it stops.

## One value, two encodings

The library's own demonstration encodes one nested archive of messages under two applications,
`AppA` and `AppB`. The data types derive nothing from Serde. Each application is a **context**, a
type that stands for that application and holds its choices, and its wiring table names an
implementation, or **provider**, for each type it encodes. The two tables are the same except for
three lines. `AppA` has these:

```rust
@ValueSerializerComponent.[u64, String]: UseSerde,
@ValueSerializerComponent.Vec<u8>: SerializeHex,
@ValueSerializerComponent.DateTime<Utc>: SerializeRfc3339Date,
```

and `AppB` has these:

```rust
@ValueSerializerComponent.[i64, u64, String]: UseSerde,
@ValueSerializerComponent.Vec<u8>: SerializeBase64,
@ValueSerializerComponent.DateTime<Utc>: SerializeTimestamp,
```

Serialized through `serde_json`, the first message of the archive comes out like this from `AppA`:

```json
{
  "message_id": 1,
  "author_id": 2,
  "date": "2025-11-03T14:15:00+00:00",
  "encrypted_data": "48656c6c6f2066726f6d20527573744c616221"
}
```

and like this from `AppB`:

```json
{
  "message_id": 1,
  "author_id": 2,
  "date": 1762179300,
  "encrypted_data": "SGVsbG8gZnJvbSBSdXN0TGFiIQ=="
}
```

Bytes are hex in one and base64 in the other, and dates are RFC 3339 strings in one and Unix
timestamps in the other, at every level of nesting. In Serde, `Vec<u8>` and `DateTime<Utc>` each
have one `Serialize` impl in the whole program, and a struct's derive fixes how its fields are
written, so producing both documents from the same structs needs a second set of wrapper types. Here
each choice is one wiring line in the application that makes it. The
[`messages`](./examples/messages.md) example walks through the whole program.

## What the design shows

cgp-serde exists to show what CGP's per-context wiring does to a trait every Rust developer knows,
and it shows three things.

**The application chooses the encoding.** The serialized value becomes a parameter of the
component, and the context becomes the type that implements it, so each application makes its own
choice for each value type, including types it does not own. The
[component design](./architecture/component-design.md) page explains the move, and
[Coherence](/docs/concepts/coherence) explains why it lets choices overlap without conflicting.

**A data type needs no serialization derive.** A struct that derives CGP's general-purpose
[`CgpData`](/docs/reference/derives/derive_cgp_data) is serialized by generic providers that read
its fields, so a crate can define serializable types without depending on `serde` at all. See
[derive-free records](./architecture/derive-free-records.md).

**A deserializer can draw on its context.** Every provider receives the context, so a deserializer
can take a service from it while it works, such as an arena to allocate borrowed values into.
Serde's traits have no place to pass one. See [context
services](./architecture/context-services.md).

A user who only wires a context meets little of CGP beyond one wiring table. Writing a provider is
where CGP's traits become the subject, and that is where the learning cost falls.

## Status and costs

cgp-serde is a demonstration of the design, and these are the costs a reader evaluating it should
know first:

- **It is a proof of concept.** The library is lightly tested and not meant for production use. The
  [limitations page](./limitations.md) describes the limits of its design.
- **A context names every type it encodes.** The wiring table has an entry for each type a value
  contains, including in-between types such as a `Vec` of structs, so it is longer than the data
  definitions it serves. A type with no entry is a compile error, not a runtime surprise.
- **Mistakes produce long compile errors.** A missing entry deep inside a nested type fails to
  compile, and the compiler's raw message lists every step it took to find it. Checking the context
  with `check_components!` and reading the error with [`cargo cgp check`](/docs/cargo-cgp/check)
  names the missing entry: `cargo cgp check` leads with the root cause for the classes it
  recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class. The [debugging
  guide](./guides/debugging-wiring.md) shows the common mistakes.
- **Its output suits self-describing formats.** JSON and RON work with it as they are.
  Length-prefixed binary formats, such as postcard, do not accept the maps and sequences its generic
  providers write. See [using a format](./guides/formats.md).
- **It builds on stable Rust.** The repository pins its toolchain, and nothing needs nightly.

## The examples

The examples are the repository's tests, walked through as short tutorials in the order they teach:

- [`basic`](./examples/basic.md) — one struct written to JSON and read back, with its bytes as hex
  and its errors in the context's error type.
- [`messages`](./examples/messages.md) — one nested archive encoded two ways by two contexts that
  differ in three wiring lines.

Two more tests deserialize borrowed values into an arena that the context supplies. Their pages are
still being written, and [context services](./architecture/context-services.md) explains the design
they show.

## The rest of the section

The [architecture](./architecture/index.md) pages explain the design one idea at a time. The guides
cover [wiring a context](./guides/wiring-a-context.md), [writing a
provider](./guides/writing-a-provider.md), [using a format](./guides/formats.md), and [debugging the
wiring](./guides/debugging-wiring.md). The [reference](./reference/index.md) has a page for each
provider. The [comparison with Serde](./serde-comparison.md) sets out what cgp-serde keeps, adds,
and lacks, and when plain Serde is the better choice, and the [limitations page](./limitations.md)
describes where the design stops.

## Running it

Clone the [repository](https://github.com/contextgeneric/cgp-serde) and run the examples from its
root with `cargo test -p cgp-serde-tests`. The repository's toolchain file selects the Rust version,
and no example needs a network or any program outside the build.

To use cgp-serde from your own crate, depend on its crates from the repository by git: `cgp-serde`
for the components and core providers, and `cgp-serde-extra`, `cgp-serde-json`, `cgp-serde-alloc`,
or `cgp-serde-typed-arena` for the providers your wiring names. The crates published on crates.io
are an earlier release, built on an older version of CGP, and the code on these pages does not
compile against them.

## Where to go next

- [`basic`](./examples/basic.md): the first example, and the one to read first.
- [Modularity hierarchy](/docs/concepts/modularity-hierarchy): where cgp-serde's shape sits among
  the ways CGP lets code choose an implementation.
- [Hello World](/docs/tutorials/hello): a first CGP program, for a reader new to CGP.
- [Announcing cgp-serde](/blog/cgp-serde-release): the post that announced the library, with its
  motivation. Its code predates the current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
