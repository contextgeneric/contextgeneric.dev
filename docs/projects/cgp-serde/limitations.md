---
sidebar_label: 'Limitations'
sidebar_position: 10
description: 'The limits of cgp-serde: a proof of concept, the part of Serde it replaces, the types and formats it covers, and when Serde fits better.'
---

# Limitations

cgp-serde is a proof of concept built to show what CGP's wiring does for serialization, and its
design has limits worth knowing before relying on it. [cgp-serde](./index.md) rebuilds Serde's
`Serialize` and `Deserialize` as components of [CGP](/docs/). This page describes those limits,
and the cases where plain Serde is the better choice.

## It is a demonstration, not a library to depend on

cgp-serde is lightly tested and not meant for production use. Its value is as a worked example: the
clearest demonstration on this site of CGP letting different applications choose different
implementations for the same types. Read it to learn the pattern, and expect rough edges if you
build on it.

## It replaces one layer of Serde, not Serde

cgp-serde replaces the traits data types implement, `Serialize` and `Deserialize`, and keeps
everything else. Its output is whatever Serde's data model can express, written by an ordinary Serde
format. It cannot make a format do something the format does not do, and a program still depends on
`serde` and on the format crates it uses. See [the bridge to Serde](./architecture/serde-bridge.md).

## The generic providers cover structs and simple enums

The providers that walk a type generically handle structs with named fields, enums whose variants
each hold exactly one value, the standard collections, references, and types that convert to or
from another type. Enums are written only in Serde's default form, `{"Variant": value}`, and only
an enum without a lifetime can be written. The providers do not handle other enums, tuple structs,
or tuples. Those types, and any other type that already implements Serde's traits,
are encoded through their own Serde impl, with `UseSerde`, and the choices of the **context**, the
type whose wiring holds an application's choices, do not reach inside them.

A recursive type, such as a tree whose nodes contain nodes, cannot be encoded by the generic
providers at all, because the compiler cannot resolve the cycle in the wiring. It needs a provider
written for it. See [re-entrant providers](./architecture/reentrant-providers.md).

## Choices are per type, not per field

A context chooses one provider for each type, and every value of that type follows it, so two fields
of the same type in one struct are encoded the same way. There are no per-field attributes: a field
is written under its Rust name, and cannot be renamed, skipped, flattened, or defaulted when
missing. A field that needs different treatment needs a type of its own. See [derive-free
records](./architecture/derive-free-records.md).

## The output suits self-describing formats

The generic providers write structs as maps and start maps and sequences without declaring their
length. Self-describing formats such as JSON and RON accept that. Length-prefixed binary formats,
such as postcard, do not. See [using a format](./guides/formats.md).

## Every type needs an entry

A context names a provider for every type it encodes, in each direction, including types that
appear only along the way, such as a `Vec` of structs, the references a collection yields, and the
string an encoding produces. Nothing is found automatically. The table is checked completely at
compile time, but it is longer than the data it serves, and it grows with every type the data
reaches. See [wiring a context](./guides/wiring-a-context.md).

## Mistakes produce long compile errors

A missing entry is a compile error, and the compiler's raw message lists every step from the
top-level type down to the entry it could not find. Checking the context with `check_components!`
and reading the error with [`cargo cgp check`](/docs/cargo-cgp/check) gets to the cause faster:
`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet
reshape every class. The [debugging
guide](./guides/debugging-wiring.md) shows the common mistakes.

## When plain Serde is the better choice

Most programs are better served by Serde's derive:

- **A program that encodes each type one way** needs no choice per application, and Serde's derive
  gives it that encoding with no wiring to write.
- **A type that needs Serde's attributes or enum representations** is better with Serde's derive,
  and can still sit inside a cgp-serde value through `UseSerde`.
- **A binary format**, or any format that needs lengths declared, needs Serde's derived impls.

cgp-serde's design earns its cost where the same types must be encoded differently by different
applications, where a crate should not depend on `serde`, or where deserializing needs something
from its surroundings. The [comparison with Serde](./serde-comparison.md) sets out the trade in
full.

## Where to go next

- [How cgp-serde works](./architecture/index.md): the design these limits follow from.
- [Modularity hierarchy](/docs/concepts/modularity-hierarchy): the CGP idea behind cgp-serde, and
  when a simpler tier is enough.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
