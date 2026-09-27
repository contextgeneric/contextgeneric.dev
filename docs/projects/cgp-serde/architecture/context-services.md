---
sidebar_label: 'Context services'
sidebar_position: 5
description: 'How a cgp-serde deserializer takes a service such as an arena from its context, in layers that make the allocator a wiring choice.'
---

# Context services

How does a deserializer get hold of something it needs while it works, such as an arena to allocate
into, when Serde's `Deserialize` receives nothing but the input? [cgp-serde](../index.md) rebuilds
Serde's `Serialize` and `Deserialize` as components of [CGP](/docs/), and every provider in it
receives the context. This page explains how a provider uses that to take a service from the
context, the layers the library's arena support is built in, and the lifetimes that let deserialized
values outlive the context.

## A provider can ask the context for anything

Serde's `Deserialize` has no receiver, and `serde_json::from_str` has no argument for extra state,
so a deserializer that needs an allocator, a lookup table, or a setting has nowhere to receive it. A
cgp-serde provider receives the **context**, the type that holds an application's choices, and it
can require any trait the context implements, not only the serialization components. It declares the
requirement with [`#[uses]`](/docs/reference/attributes/uses) and calls it on `self`, as it would
any method. This is CGP's [impl-side dependencies](/docs/concepts/impl-side-dependencies), applied
to a problem Serde's traits have no room for.

The library's worked case is deserializing borrowed values. A `Payload<'a>` holding a
`Vec<&'a Coord>` points at `Coord` values that must live somewhere, and allocating them all into one
arena is cheaper than giving each a `Box`. The deserializer needs the arena while it runs, and it
takes it from the context.

## Three layers, so the allocator is a choice

The work is split into three layers, each a separate piece of wiring, so that the deserializer does
not decide how memory is allocated.

**The deserializer** reads an owned value and asks the context to allocate it.
[`DeserializeAndAllocate`](../reference/providers/deserialize_and_allocate.md) is the whole of it:

```rust
#[cgp_impl(new DeserializeAndAllocate)]
#[uses(CanAlloc<'a, Value>, CanDeserializeValue<'de, Value>)]
impl<'de, 'a, Value> ValueDeserializer<'de, &'a Value> {
    fn deserialize<D>(&self, deserializer: D) -> Result<&'a Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = self.deserialize(deserializer)?;
        let value = self.alloc(value);

        Ok(value)
    }
}
```

It deserializes a `&'a Value` by asking the context to deserialize a `Value`, then to allocate it.
It knows nothing about arenas.

**The allocation component** is what it asks for.
[`CanAlloc`](../reference/components/can_alloc.md) moves a value into storage that lives for `'a`
and returns a reference to it:

```rust
#[cgp_component(Allocator)]
pub trait CanAlloc<'a, T> {
    fn alloc(&self, value: T) -> &'a mut T;
}
```

**The implementation** answers it.
[`AllocateWithArena`](../reference/providers/allocate_with_arena.md) allocates into a
`typed_arena::Arena` that it gets from the context through an arena getter, a component whose
provider reads the arena from a field:

```rust
#[cgp_impl(new AllocateWithArena)]
#[uses(HasArena<'a, Value>)]
impl<'a, Value: 'a> Allocator<'a, Value> {
    fn alloc(&self, value: Value) -> &'a mut Value {
        self.arena().alloc(value)
    }
}
```

A context wires all three. These are the relevant entries from the repository's arena test, where
the context holds the arena in a field named `arena`:

```rust
#[derive(HasField)]
pub struct App<'a> {
    pub arena: &'a Arena<Coord>,
}

delegate_components! {
    <'a> App<'a> {
        open {
            ValueDeserializerComponent,
            // ...
        };

        ArenaGetterComponent:
            UseField<Symbol!("arena")>,
        AllocatorComponent:
            AllocateWithArena,

        @ValueDeserializerComponent.<'b> &'b Coord:
            DeserializeAndAllocate,

        @ValueDeserializerComponent.<'b> Vec<&'b Coord>:
            DeserializeExtend,
        // ...
    }
}
```

[`UseField`](/docs/reference/providers/use_field) points the arena getter at the `arena` field.
Swapping the allocator means wiring `AllocatorComponent` to a different provider, and neither the
deserializer nor any data type changes. The allocation component and the arena implementation are
in separate crates for the same reason, so that a program with its own allocator depends on the
component alone.

## The values outlive the context

A deserialized `Payload<'a>` stays valid after the context is gone, because the context only borrows
the arena. The caller creates the arena first and hands the context a reference to it, as
`&'a Arena<Coord>`. `CanAlloc<'a, T>` returns references tied to that `'a`, not to the context, so
the values live as long as the arena does.

The input's lifetime, `'de`, is separate from `'a`. The input can be dropped as soon as
deserializing finishes, because the values borrow from the arena rather than from the input.

## What it costs

Every service a provider draws from the context is one more thing the context must supply and wire,
and the layers add entries of their own: the arena version needs a getter entry and an allocator
entry besides the per-type entries. A single provider that reached into a field directly would be
shorter to read, at the price of fixing the allocator inside it.

The pay-off is that a deserializer can depend on its surroundings without its caller threading the
dependency through every call. The context carries the arena, and every provider that needs it finds
it there.

## Where to go next

- [Crate layout](./crate-layout.md): why allocation is two crates, and what each crate depends on.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): the CGP idea of a provider
  declaring what it needs from its context.
- [Rust language proposals](/docs/comparisons/rust-language-proposals): the proposal for passing
  context through Rust code, which this pattern resembles at the library level.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
