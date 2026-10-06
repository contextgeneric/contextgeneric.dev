---
title: 'AllocateWithArena — allocate into an arena held by the context in cgp-serde'
sidebar_label: 'AllocateWithArena'
sidebar_position: 25
description: 'The cgp-serde provider that implements the allocation component by moving values into a typed_arena::Arena the context supplies through an arena getter.'
---

# `AllocateWithArena`

Allocate a value into the arena the context supplies.

## Overview

`AllocateWithArena` implements the allocation component, [`CanAlloc`](../components/can_alloc.md),
over the [`typed-arena`](https://docs.rs/typed-arena) crate. It gets a `typed_arena::Arena` from the
**context**, the type whose wiring holds the application's choices, through the arena getter
`HasArena`, and moves each value into it. Every value in an arena is freed together when the arena
is dropped.

## Definition

```rust
#[cgp_impl(new AllocateWithArena)]
#[uses(HasArena<'a, Value>)]
impl<'a, Value: 'a> Allocator<'a, Value> { ... }
```

`Allocator` is the provider trait of `CanAlloc`. `HasArena<'a, T>`, from
`cgp_serde_typed_arena::traits`, is a getter component whose method returns the context's
`&'a Arena<T>`, and its wiring key is `ArenaGetterComponent`.

## Usage

Import it from `cgp_serde_typed_arena::providers`. Wire it as the context's allocator, and point the
arena getter at the field that holds the arena with
[`UseField`](/docs/reference/providers/use_field). From the repository's arena example:

```rust
#[derive(HasField)]
pub struct App<'a> {
    pub arena: &'a Arena<Coord>,
}

delegate_components! {
    <'a> App<'a> {
        // ...
        ArenaGetterComponent:
            UseField<Symbol!("arena")>,
        AllocatorComponent:
            AllocateWithArena,
        // ...
    }
}
```

A context with arenas for several types opens `ArenaGetterComponent` and keys the getter on the
type, as in `@ArenaGetterComponent.Coord: UseField<Symbol!("coords")>`.

## Behavior

The provider calls `self.arena().alloc(value)`, so the value moves into the arena and lives as long
as the arena does. The context holds the arena by reference, borrowed from its caller, which is what
lets allocated values outlive the context.

## Context dependencies

`HasArena<'a, Value>`: an arena getter for each type allocated, wired to the field that holds it.

## Pairing

Not applicable; it implements allocation, not serialization.

## When to use it

**Reach for `AllocateWithArena` when a context deserializes many borrowed values of one type** and
they can all be freed together. A different allocator implements `Allocator` for itself, and the
deserializer is unchanged.

## Related constructs

- [`DeserializeAndAllocate`](./deserialize_and_allocate.md), the deserializer that calls it.

## The ideas behind it

- [Context services](../../architecture/context-services.md): the layers that make the allocator a
  wiring choice, and the lifetimes involved.

## Source

- [`crates/cgp-serde-typed-arena/src/providers/alloc.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-typed-arena/src/providers/alloc.rs)
- [`crates/cgp-serde-typed-arena/src/traits/has_arena.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-typed-arena/src/traits/has_arena.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
