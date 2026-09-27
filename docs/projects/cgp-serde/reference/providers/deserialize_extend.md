---
title: 'DeserializeExtend — read a sequence into a collection in cgp-serde'
sidebar_label: 'DeserializeExtend'
sidebar_position: 11
description: 'The cgp-serde provider that reads a Serde sequence into any collection with a default that can be extended, asking the context to read each item.'
---

# `DeserializeExtend`

Read a sequence into any collection that starts empty and can be extended item by item.

## Overview

`DeserializeExtend` reads a Serde sequence into a `Vec`, a set, or any collection that implements
`Default` and `Extend`, and asks the **context**, the type whose wiring holds the application's
choices, to read each item. So the items follow the context's choices, including items that borrow
from an arena through [`DeserializeAndAllocate`](./deserialize_and_allocate.md).

## Definition

```rust
pub struct DeserializeExtend;

#[cgp_impl(DeserializeExtend)]
#[uses(CanDeserializeValue<'de, Item>)]
impl<'de, Value, Item> ValueDeserializer<'de, Value>
where
    Value: Default + IntoIterator<Item = Item> + Extend<Item>,
{ ... }
```

The `IntoIterator` bound only names the collection's item type; the provider never iterates.

## Usage

Import it from `cgp_serde::providers`, and wire each collection type the context reads, beside its
item type. The repository's arena test reads a vector of references into an arena:

```rust
@ValueDeserializerComponent.<'b> Vec<&'b Coord>:
    DeserializeExtend,
```

## Behavior

The provider asks the format for a sequence, starts from `Value::default()`, and adds each item as
it is read, through the context. Any collection meeting the bounds works: a `BTreeSet<u64>` reads
`[3,1,3]` as `{1, 3}`, because the set's own `Extend` drops the repeat. Input that is not a sequence
is the format's type error; with JSON, an object fails with `invalid type: map, expected sequence`.

A map's items are key-value pairs, so a map is read from a sequence of pairs, the shape
[`SerializeIterator`](./serialize_iterator.md) writes, rather than from a JSON object.

## Context dependencies

`CanDeserializeValue<'de, Item>` for the collection's item type.

## Pairing

[`SerializeIterator`](./serialize_iterator.md) writes the sequence it reads.

## When to use it

**Reach for `DeserializeExtend` for any collection whose items should be read through the
context.** For a collection of plain types, [`UseSerde`](./use_serde.md) reads it without an entry
per item type.

## Related constructs

- [`DeserializeRecordFields`](./deserialize_record_fields.md) is the matching provider for a
  struct.

## The ideas behind it

- [Re-entrant providers](../../architecture/reentrant-providers.md): how each item is read through
  the context.

## Source

- [`crates/cgp-serde/src/providers/extend.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/extend.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
