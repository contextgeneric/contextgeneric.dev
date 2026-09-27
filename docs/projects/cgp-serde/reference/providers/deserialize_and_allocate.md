---
title: 'DeserializeAndAllocate — read a reference into an allocator in cgp-serde'
sidebar_label: 'DeserializeAndAllocate'
sidebar_position: 22
description: 'The cgp-serde provider that reads a borrowed &T by reading an owned T through the context and allocating it through the context''s allocation component.'
---

# `DeserializeAndAllocate`

Read a borrowed `&'a Value` by reading an owned `Value` and allocating it through the context.

## Overview

`DeserializeAndAllocate` lets a deserialized value hold references to other deserialized values,
such as a `Vec<&'a Coord>`, without the references pointing into the input. It reads each value as
an owned `Value`, then asks the **context**, the type whose wiring holds the application's choices,
to allocate it with [`CanAlloc`](../components/can_alloc.md), and returns the reference. It knows
nothing about where the value is stored, which is the context's choice.

## Definition

```rust
#[cgp_impl(new DeserializeAndAllocate)]
#[uses(CanAlloc<'a, Value>, CanDeserializeValue<'de, Value>)]
impl<'de, 'a, Value> ValueDeserializer<'de, &'a Value> { ... }
```

## Usage

Import it from `cgp_serde_alloc::providers`, and wire it for the reference type, beside an entry for
the owned type and an allocator. From the repository's arena test:

```rust
AllocatorComponent:
    AllocateWithArena,

@ValueDeserializerComponent.[
    Coord,
    <'b> Payload<'b>,
]:
    DeserializeRecordFields,

@ValueDeserializerComponent.<'b> &'b Coord:
    DeserializeAndAllocate,
```

## Behavior

The provider reads the owned value through the context, passes it to `alloc`, and returns the
reference the allocator gives back. The reference's lifetime `'a` is separate from the input's
`'de`, so the value outlives the input and borrows from the allocator instead.

## Context dependencies

`CanDeserializeValue<'de, Value>` for the owned type, and `CanAlloc<'a, Value>`. Without an
allocator entry, the context fails to compile with a missing `AllocatorComponent` entry, as [the
debugging guide](../../guides/debugging-wiring.md#an-entry-for-a-type-no-field-has) shows.

## Pairing

It reads only. A reference is written with [`SerializeDeref`](./serialize_deref.md).

## When to use it

**Reach for `DeserializeAndAllocate` when deserialized values should share storage**, such as many
small values in one arena rather than a `Box` each.

## Related constructs

- [`AllocateWithArena`](./allocate_with_arena.md), an allocator for it to call.
- [`DeserializeExtend`](./deserialize_extend.md), which reads a collection of the references.

## The ideas behind it

- [Context services](../../architecture/context-services.md): a deserializer taking a service from
  its context, in layers.

## Source

- [`crates/cgp-serde-alloc/src/providers/alloc_deserialize.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-alloc/src/providers/alloc_deserialize.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
