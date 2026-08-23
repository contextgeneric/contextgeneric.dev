---
sidebar_label: 'UseContext'
sidebar_position: 1
---

# `UseContext`

Satisfy a provider trait by routing back through the context's own consumer-trait implementation.

## Overview

`UseContext` turns a context's existing consumer-trait implementation into a provider that other
providers can call. The **context** is the type a capability runs against, and it normally *uses* a
provider through its consumer trait. Sometimes the implementation another provider wants for a
capability is exactly the one the context already supplies that way. `UseContext` is that bridge: it is
a provider whose method bodies call the consumer method on the context, so handing a component
`UseContext` means "use whatever this context already does for this trait."

This makes `UseContext` the exact dual of the consumer-trait blanket implementation that
[`#[cgp_component]`](../macros/cgp_component.md) generates. That blanket impl runs in the
consumer-to-provider direction: a context implements `CanEncode` by delegating to whichever provider
implements `Encoder` for it. `UseContext` runs the other way: it implements the provider trait
`Encoder` by delegating to whatever `CanEncode` implementation the context has. One forwards the
consumer trait to a provider; the other forwards a provider trait back to the consumer trait.

`UseContext` matters most with [higher-order providers](/docs/concepts/higher-order-providers), which
take another provider as a type parameter. Such a provider can default its inner-provider parameter to
`UseContext`, so when no inner provider is named, the inner step falls back to whatever the context
already wires for it. That inner step may be a different capability, or the same capability at a
different type, as the example below shows by encoding a `Vec<T>` through the context's own encoder for
`T`. Like every CGP provider, `UseContext` carries no runtime value: it is a unit marker whose `self`
position is never read.

## Usage

`UseContext` is in the prelude, so `use cgp::prelude::*;` is enough. It takes no type parameter and
appears in two places. Most often it is a higher-order provider's default inner provider, written in
the provider's struct definition:

```rust
pub struct EncodeVec<Inner = UseContext>(pub PhantomData<Inner>);
```

It also appears directly in a wiring entry to route a component through the context's own
implementation, which is how the [dispatch combinators](dispatch/index.md) default their
per-variant provider.

`UseContext` stands in for a provider trait only where that trait resolves to a *different*
implementation than the one being wired: a wrapper over another capability, or a per-type dispatch entry
for a different type.

## Examples

The idiomatic use is as the default inner provider of a higher-order provider. `EncodeVec` encodes a
`Vec` by encoding each element through an inner provider, and defaults that inner provider to
`UseContext`, so the elements are encoded by whatever the context already wires for their type:

```rust
use cgp::prelude::*;

#[cgp_component(Encoder)]
pub trait CanEncode<Value> {
    fn encode(&self, value: &Value) -> Vec<u8>;
}

#[cgp_impl(new EncodeAsText)]
impl<Value> Encoder<Value>
where
    Value: core::fmt::Display,
{
    fn encode(&self, value: &Value) -> Vec<u8> {
        value.to_string().into_bytes()
    }
}

pub struct EncodeVec<Inner = UseContext>(pub PhantomData<Inner>);

#[cgp_impl(EncodeVec<Inner>)]
#[use_provider(Inner: Encoder<Item>)]
impl<Item, Inner> Encoder<Vec<Item>> {
    fn encode(&self, values: &Vec<Item>) -> Vec<u8> {
        values
            .iter()
            .flat_map(|item| Inner::encode(self, item))
            .collect()
    }
}

pub struct App;

delegate_components! {
    App {
        open EncoderComponent;

        @EncoderComponent.u32: EncodeAsText,
        @EncoderComponent.Vec<u32>: EncodeVec,
    }
}
```

`App` uses the [`open` statement](../macros/delegate_components.md) to wire the `Encoder` component per
value type, giving `Vec<u32>` a bare `EncodeVec` whose `Inner` defaults to `UseContext`. Encoding a
`Vec<u32>` runs `EncodeVec`, which calls `Inner::encode(self, item)` on each element; `Inner` resolves
through `UseContext` to `App`'s own `CanEncode<u32>`, wired to `EncodeAsText`. So encoding
`vec![1u32, 2, 3]` produces the bytes of `"123"`. There is no cycle, because the element lookup is for a
*different* type (`u32`) than the wired one (`Vec<u32>`). Naming an explicit inner provider in place of
the `UseContext` default binds the element encoder directly and bypasses the context's `CanEncode<u32>`
wiring.

`UseContext` acts as a default only when the provider's struct definition gives it as the default
generic parameter, as `EncodeVec` does. A provider without such a default has no inner provider to fall
back to, and the inner provider must always be named.

## When to use it

**Reach for `UseContext` as the default inner provider of a higher-order provider**, so the wrapper
reuses the context's own wiring for the inner step when no inner provider is named. The inner step may
be a different capability, or the same capability at a different type, as `EncodeVec` delegates
`Encoder<Vec<T>>` to the context's `Encoder<T>`.

The one shape `UseContext` cannot serve is a component whose only implementation on the context is that
same delegation, which loops; [Common Mistakes](#common-mistakes) explains it. When you want a wrapper
over the *same* capability and the *same* type, name the inner provider explicitly rather than
defaulting to `UseContext`.

## Under the hood

[`#[cgp_component]`](../macros/cgp_component.md) emits a `UseContext` implementation of the provider
trait for every component it defines, alongside the consumer blanket impl, the provider blanket impl,
the component marker, and the [`RedirectLookup`](redirect_lookup.md) impl. The generated impl bounds the
context on the consumer trait and forwards each method to it. For a component such as

```rust
#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}
```

the macro generates this `UseContext` impl (shown with the macro's real placeholder identifiers):

```rust
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    fn greet(__context__: &__Context__) {
        __Context__::greet(__context__)
    }
}
```

The provider method takes the context explicitly and calls the context's own `CanGreet::greet`. Each
`UseContext` impl is paired with a matching [`IsProviderFor`](../traits/is_provider_for.md) impl
carrying the same `where` clause, so delegation propagates the dependency and a check reports a missing
consumer-trait implementation precisely. Any supertrait bound on the consumer trait is reproduced in the
`where` clause, so a context must satisfy it before `UseContext` can stand in as a provider.

## Common Mistakes

**Wiring a component to `UseContext` on the context whose only implementation is that same delegation
creates a cycle.** If `App` wired `@EncoderComponent.u32: UseContext`, then `App`'s `CanEncode<u32>`
would be implemented by delegating to `UseContext`, whose `Encoder<u32>` impl in turn calls `App`'s
`CanEncode<u32>`. The trait solver chases this in a loop and reports it as an overflow or an unsatisfied
bound. `UseContext` belongs where the provider trait resolves to a *different* implementation, such as a
wrapper over another capability or a per-type dispatch entry for a different type. The `EncodeVec`
example above is safe for exactly that reason: it is wired for `Vec<u32>`, but its inner `UseContext`
resolves `Encoder<u32>`, a different entry.

## Related constructs

- [`#[cgp_component]`](../macros/cgp_component.md) — generates the `UseContext` impl for every
  component, and the consumer blanket impl `UseContext` is the dual of.
- [Higher-order providers](/docs/concepts/higher-order-providers) — where `UseContext` is the default
  inner provider.
- [`WithContext`](with_context.md) — the `WithProvider`-adapted alias of `UseContext`, for serving a
  component from the context's own foundational implementation.
- [`WithProvider`](with_provider.md) — its `WithContext` alias is `WithProvider<UseContext>`, adapting a
  context's own implementation into a component.
- [`RedirectLookup`](redirect_lookup.md) — the other `#[cgp_component]`-generated provider, routing
  through a separate table rather than back to the context.
- [Dispatch combinators](dispatch/index.md) — default their per-variant provider to `UseContext`.
- [`delegate_components!`](../macros/delegate_components.md) — wires it, and its `open` statement
  dispatches a component per type as the example does.
- [`IsProviderFor`](../traits/is_provider_for.md) — carries the dependency `UseContext` propagates.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the two-way split
  `UseContext` bridges.
- [Higher-order providers](/docs/concepts/higher-order-providers) — the pattern that makes it useful.

## Source

- Struct and the `WithContext` alias:
  [`use_context.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/use_context.rs)
- The generated `UseContext` impl:
  [`cgp_component/evaluated/to_use_context_impl.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_component/evaluated/to_use_context_impl.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
