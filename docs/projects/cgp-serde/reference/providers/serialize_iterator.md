---
title: 'SerializeIterator — write a collection as a sequence in cgp-serde'
sidebar_label: 'SerializeIterator'
sidebar_position: 10
description: 'The cgp-serde provider that writes any collection iterable by reference as a Serde sequence, asking the context to write each item.'
---

# `SerializeIterator`

Write any collection that can be iterated by reference as a sequence of its items.

## Overview

`SerializeIterator` writes a `Vec`, a slice, a set, or any other collection as a Serde sequence,
and asks the **context**, the type whose wiring holds the application's choices, to write each item.
So the items follow the context's choices: a `Vec<Vec<u8>>` in a context that wires `Vec<u8>` to hex
is a list of hex strings.

## Definition

```rust
#[cgp_impl(new SerializeIterator)]
impl<Value> ValueSerializer<Value>
where
    for<'a> &'a Value: IntoIterator,
    Self: for<'a> CanSerializeValue<<&'a Value as IntoIterator>::Item>,
{ ... }
```

Both bounds hold for every lifetime of the borrow, because the item type depends on it. That is why
the requirement on the context is written in the `where` clause rather than with
[`#[uses]`](/docs/reference/attributes/uses).

## Usage

Import it from `cgp_serde::providers`, and wire each collection type the context writes, beside the
generic reference entry for its items. From the [`messages`](../../examples/messages.md) example:

```rust
@ValueSerializerComponent.<'a, T> &'a T:
    SerializeDeref,

@ValueSerializerComponent.[
    Vec<EncryptedMessage>,
    Vec<MessagesByTopic>,
]:
    SerializeIterator,
```

## Behavior

The provider starts a sequence, then writes each item that iterating `&Value` yields, wrapped with
the context in [`SerializeWithContext`](../types/serialize_with_context.md). With JSON, a `Vec<u64>`
or a `BTreeSet<u64>` becomes an array, and a nested vector whose inner type is also wired to
`SerializeIterator` becomes a nested array. The sequence is started without a declared length.

The items are what iterating by reference yields, which for a `Vec<T>`, a slice, or a set is `&T`.
That is why the context needs an entry for the reference, which the generic `&'a T` entry to
[`SerializeDeref`](./serialize_deref.md) provides. A map yields pairs of references, so a map is
written as a sequence of pairs, each pair written by the context's entry for the tuple type.

## Context dependencies

`CanSerializeValue<I>` for each item type `I` the collection yields by reference, for every lifetime
of the borrow: usually the reference entry, and through it an entry for the item type.

## Pairing

[`DeserializeExtend`](./deserialize_extend.md) reads a sequence back into a collection.

## When to use it

**Reach for `SerializeIterator` for any collection whose items should follow the context's
choices.** For a collection of types the application does not customize, such as a `Vec<u64>`,
[`UseSerde`](./use_serde.md) writes the same array without an entry per item type.

## Related constructs

- [`SerializeFields`](./serialize_fields.md) is the matching provider for a struct.

## The ideas behind it

- [Re-entrant providers](../../architecture/reentrant-providers.md#re-entering-through-an-adapter):
  handing each item to Serde with the context attached.

## Source

- [`crates/cgp-serde/src/providers/iterator.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/iterator.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
