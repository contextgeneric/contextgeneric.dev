---
title: 'SerializeFrom — encode a value through a converted type in cgp-serde'
sidebar_label: 'SerializeFrom'
sidebar_position: 7
description: 'The cgp-serde provider that encodes a value through another type it converts to or from with Into, asking the context for that type.'
---

# `SerializeFrom`

Encode a value through another type it converts to or from with `Into`.

## Overview

`SerializeFrom<Target>` lets a type borrow another type's encoding. Writing converts the value into
`Target` and asks the **context**, the type whose wiring holds the application's choices, to write
the `Target`; reading asks the context for the other type and converts from it. So a newtype or a
narrow integer can be encoded exactly as a wider type the context already knows how to write.

## Definition

```rust
pub struct SerializeFrom<Target>(pub PhantomData<Target>);

#[cgp_impl(SerializeFrom<Target>)]
#[uses(CanSerializeValue<Target>)]
impl<Value, Target> ValueSerializer<Value>
where
    Value: Clone + Into<Target>,
{ ... }

#[cgp_impl(SerializeFrom<Source>)]
#[uses(CanDeserializeValue<'a, Source>)]
impl<'a, Value, Source> ValueDeserializer<'a, Value>
where
    Source: Into<Value>,
{ ... }
```

The type parameter plays opposite roles in the two directions: it is the type the value converts
into when writing, and the type it converts from when reading.

## Usage

Import it from `cgp_serde::providers`, and name the in-between type as its parameter:

```rust
@ValueSerializerComponent.u32: SerializeFrom<u64>,
@ValueDeserializerComponent.u32: SerializeFrom<u8>,
@ValueSerializerComponent.u64: UseSerde,
@ValueDeserializerComponent.u8: UseSerde,
```

Here a `u32` is written as a `u64`, and read as a `u8` that is widened. A context that wires one
type in both directions usually names two different parameters, since `Into` rarely holds both ways.

## Behavior

Writing clones the value, converts the clone with `Into`, and writes the result through the context.
The clone is needed because `Into` consumes its input and the provider holds only a reference, so
the value type must implement `Clone`. Reading reads the other type through the context and converts
it with `Into`.

## Context dependencies

`CanSerializeValue<Target>` when writing, and `CanDeserializeValue<'de, Target>` when reading.

## Pairing

The same provider implements both directions. [`TrySerializeFrom`](./try_serialize_from.md) is the
fallible form.

## When to use it

**Reach for `SerializeFrom` when a type should be encoded as another type it converts to without
loss**, such as a newtype around a number. For a conversion that can fail, such as narrowing an
integer, reach for [`TrySerializeFrom`](./try_serialize_from.md).

## Related constructs

- [`SerializeWithDisplay`](./serialize_with_display.md) encodes through a string.
- [`SerializeDeref`](./serialize_deref.md) encodes through the value a reference points to.

## The ideas behind it

- [Re-entrant providers](../../architecture/reentrant-providers.md#re-entering-directly): a provider
  that converts and asks the context to encode the result.

## Source

- [`crates/cgp-serde/src/providers/from.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/from.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
