---
title: 'SerializeDeref — write a reference as the value behind it in cgp-serde'
sidebar_label: 'SerializeDeref'
sidebar_position: 9
description: 'The cgp-serde provider that writes a reference or smart pointer as the value it points to, and the generic reference entry a collection needs.'
---

# `SerializeDeref`

Write a reference or smart pointer as the value it points to.

## Overview

`SerializeDeref` dereferences a value and asks the **context**, the type whose wiring holds the
application's choices, to write the target. Its main use is one generic entry that forwards every
reference to the context's choice for the referenced type, which a context needs whenever it writes
a collection with [`SerializeIterator`](./serialize_iterator.md), since walking a collection by
reference yields references.

## Definition

```rust
#[cgp_impl(new SerializeDeref)]
#[uses(CanSerializeValue<Value::Target>)]
impl<Value> ValueSerializer<Value>
where
    Value: Deref,
{ ... }
```

## Usage

Import it from `cgp_serde::providers`, and wire it for every reference at once with a generic entry,
as the [`messages`](../../examples/messages.md) example does:

```rust
@ValueSerializerComponent.<'a, T> &'a T:
    SerializeDeref,
```

It also works for a smart pointer wired as its own type, such as `Box<u64>`.

## Behavior

The provider dereferences the value and writes the target through the context, so `&u64` and
`Box<u64>` are written exactly as `u64` is. The target must be a sized type, so `&str` and `&[T]`
are not forwarded this way; wire them to a provider for the reference type itself, such as
[`SerializeString`](./serialize_string.md) for `&'a str`.

## Context dependencies

`CanSerializeValue<Value::Target>`, the type behind the reference.

## Pairing

It writes only. Reading into a reference needs somewhere for the value to live, which is what
[`DeserializeAndAllocate`](./deserialize_and_allocate.md) provides.

## When to use it

**Reach for the generic `&'a T` entry in any context that writes a collection with
`SerializeIterator`.** Without it, the context has no entry for the items the collection yields.

## Related constructs

- [`SerializeFrom`](./serialize_from.md) encodes through a converted value rather than a
  dereferenced one.

## The ideas behind it

- [Re-entrant providers](../../architecture/reentrant-providers.md#what-the-context-must-wire): why
  a collection needs the reference entry.

## Source

- [`crates/cgp-serde/src/providers/deref.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/deref.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
