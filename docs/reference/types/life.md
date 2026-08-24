---
sidebar_label: 'Life'
sidebar_position: 5
---

# `Life`

A lifetime lifted into a type, so a lifetime parameter can travel through machinery that only accepts
types.

:::info

### Generated machinery

**You are not expected to write `Life` by hand.** The component-defining macros, principally
[`#[cgp_component]`](../macros/cgp_component.md), insert it when a trait carries a lifetime. You meet it
in a generated provider trait and in a provider-resolution error, and this page explains why it is there.

:::

## Overview

`Life<'a>` exists because CGP's wiring is parameterized by *types*, not lifetimes, yet a CGP trait may
carry a lifetime of its own. The marker that surfaces a provider's dependencies,
[`IsProviderFor`](../traits/wiring/is_provider_for.md), takes a tuple of the trait's generic parameters as
one type argument, so the compiler can match a provider against the exact instantiation it is asked for.
A lifetime cannot sit in that tuple, because a tuple is a type and its members must be types. So a
lifetime parameter on the trait must first become a type, and `Life<'a>` is that conversion: it packages
the lifetime `'a` as a concrete type that can stand alongside the trait's other type parameters.

Without this lift, a trait that borrows could not record its lifetime in the dependency marker, and the
wiring could not tell one lifetime instantiation from another. Here a **context** is the type the
capability runs against. `Life` lets the lifetime ride through `IsProviderFor` as `(Life<'a>, T)`,
keeping it part of the provider's identity while the marker's argument stays a plain type.

## Definition

`Life` is a tuple struct wrapping a single phantom marker over a raw pointer to a borrowed unit:

```rust
pub struct Life<'a>(pub PhantomData<*mut &'a ()>);
```

The struct holds no runtime data. Its only job is to carry the lifetime `'a` in the type system through
[`PhantomData`](phantom_data.md). The choice of `PhantomData<*mut &'a ()>` is deliberate and controls how
`Life<'a>` relates to other lifetimes under subtyping. A `*mut T` is *invariant* in `T`, so wrapping
`&'a ()` behind a `*mut` makes `Life<'a>` invariant in `'a`: a `Life<'long>` is neither a subtype nor a
supertype of a `Life<'short>`. Invariance is correct here because the lifetime is an exact identity in
the dependency marker. Two providers wired for different lifetimes must be treated as wired for genuinely
different things, and a variant `Life` would let the compiler coerce one instantiation into another and
pick the wrong provider. The raw pointer also keeps `Life<'a>` from carrying auto-trait obligations tied
to a real borrow, since it neither owns nor references a real value.

## Behavior

`Life` has no methods and implements no CGP traits of its own; its entire behavior is to occupy a type
position. In a generated provider trait for a component with a lifetime, the lifetime is collected into
the [`IsProviderFor`](../traits/wiring/is_provider_for.md) argument tuple as `Life<'a>`, so the provider's
dependency obligation reads the same way it would for any type parameter. The provider trait, its blanket
forwarding impl, and the impls that satisfy it all agree on the same `(Life<'a>, T)` shape, which is what
lets a borrowing component be wired and checked exactly like a non-borrowing one.

## Examples

`Life` appears in the wiring generated for a component whose consumer trait carries a lifetime. Given a
borrowing getter component:

```rust
use cgp::prelude::*;

#[cgp_component(ReferenceGetter)]
pub trait HasReference<'a, T: 'a + ?Sized> {
    fn get_reference(&self) -> &'a T;
}
```

the generated provider trait records the lifetime in its dependency marker through `Life`, so its
`IsProviderFor` bound names the lifetime as the type `Life<'a>` rather than as a bare `'a`:

```rust
// generated, in readable form:
// pub trait ReferenceGetter<'a, __Context__, T: 'a + ?Sized>:
//     IsProviderFor<ReferenceGetterComponent, __Context__, (Life<'a>, T)>
// {
//     fn get_reference(__context__: &__Context__) -> &'a T;
// }
```

Every impl that wires this component, whether through `UseContext`, a `UseField` getter, or a
hand-written provider, carries the same `(Life<'a>, T)` tuple, so the lifetime is preserved end to end
through the resolution machinery.

## When to use it

**You read `Life` in generated code; you do not write it.** The macros insert it, and recognizing it is
all that is asked.

- **Read `Life<'a>` in an `IsProviderFor` tuple as the component's lifetime.** A dependency marker such
  as `IsProviderFor<..., (Life<'a>, T)>` is naming a lifetime and a type parameter, in that order.
- **Do not substitute a bare [`PhantomData`](phantom_data.md) for it** when you write a lifetime marker
  by hand. A plain `PhantomData<&'a ()>` is covariant, which is the wrong relationship for a dependency
  marker; `Life` forces the invariance that keeps two lifetime instantiations distinct.

## Common Mistakes

**A lifetime cannot appear directly in the `IsProviderFor` tuple.** The tuple holds types, so a bare
`'a` is invalid there and is lifted into `Life<'a>`. Meeting `Life` in an error is the sign a component
carries a lifetime.

**`Life<'a>` is invariant in `'a`, on purpose.** It does not behave like `&'a ()`, which is covariant.
The invariance is what keeps providers wired for different lifetimes from being confused, so it is a
feature rather than an over-restriction.

**A higher-order provider with a lifetime loses its dependency propagation.** The inner-provider bound of
such a stack gets no marker counterpart when the component carries a lifetime, because the rewrite reads
the bound's first generic argument as the context and finds a lifetime there. The stack still compiles
and runs; what is lost is the propagation that lets `#[check_providers]` localize a broken layer. This is
a recorded limitation, noted on [`IsProviderFor`](../traits/wiring/is_provider_for.md).

## Related constructs

- [`IsProviderFor`](../traits/wiring/is_provider_for.md) — whose parameter tuple a lifetime is lifted
  into as `Life<'a>`.
- [`#[cgp_component]`](../macros/cgp_component.md) — inserts `Life` when a consumer trait carries a
  lifetime.
- [`PhantomData`](phantom_data.md) — the marker `Life` is built from, wrapped for invariance.
- [`Index`](index_type.md) and [`Chars`](spines/chars.md) — the other lifts that make a non-type
  addressable in trait resolution, a number and a string where `Life` lifts a lifetime.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the provider-trait
  machinery whose dependency marker carries this lift.

## Source

- The type:
  [`life.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/life.rs)
- The macro logic that wraps a trait's lifetime parameters in `Life` when building the `IsProviderFor`
  argument tuple:
  [`is_provider_params.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/functions/is_provider_params.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
