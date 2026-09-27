---
title: 'CanAlloc — the allocation component in cgp-serde'
sidebar_label: 'CanAlloc'
sidebar_position: 1
description: 'The cgp-serde component that moves a value into storage living for a lifetime and returns a reference, so a deserializer allocates through a wiring choice.'
---

# `CanAlloc`

Move a value into storage that lives for `'a`, and return a reference to it.

## Overview

`CanAlloc<'a, T>` separates what a deserializer needs, somewhere to put a value, from how that is
provided. [`DeserializeAndAllocate`](../providers/deserialize_and_allocate.md) asks the context for
it, and a **context**, the type whose wiring holds an application's choices, wires it to an
allocation provider such as [`AllocateWithArena`](../providers/allocate_with_arena.md). So swapping
the allocator is one wiring entry, and no deserializer changes.

## Definition

```rust
#[cgp_component(Allocator)]
pub trait CanAlloc<'a, T> {
    fn alloc(&self, value: T) -> &'a mut T;
}
```

[`#[cgp_component]`](/docs/reference/macros/cgp_component) also generates the provider trait
`Allocator` and the wiring key `AllocatorComponent`. All three live in `cgp_serde_alloc::traits`.

## Usage

Wire the component to a provider in the context's table:

```rust
AllocatorComponent:
    AllocateWithArena,
```

A provider of your own implements `Allocator` with [`#[cgp_impl]`](/docs/reference/macros/cgp_impl),
as `AllocateWithArena` does.

## Behavior

The trait says only that the value lives for `'a`, typically a lifetime parameter of the context,
not where it goes. The returned reference is tied to `'a` rather than to the context, so values
allocated through it can outlive the context.

## Context dependencies

None of its own; a context wires `AllocatorComponent` to a provider, which brings its own
requirements.

## Pairing

Not applicable.

## When to use it

**Reach for `CanAlloc` when a provider needs to allocate values that outlive the call.** Depending
on the component, rather than on an arena, keeps the provider usable with any allocator.

## Related constructs

- [`DeserializeAndAllocate`](../providers/deserialize_and_allocate.md), the deserializer that uses
  it.
- [`AllocateWithArena`](../providers/allocate_with_arena.md), the library's provider for it.

## The ideas behind it

- [Context services](../../architecture/context-services.md): the layered design this component is
  the middle of.
- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): what a component is.

## Source

- [`crates/cgp-serde-alloc/src/traits/alloc.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-alloc/src/traits/alloc.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
