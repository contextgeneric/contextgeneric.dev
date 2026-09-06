---
sidebar_label: 'Life'
sidebar_position: 4
---

# `Life`

A lifetime lifted into a type, so a lifetime parameter can travel through machinery that accepts only
types.

## Overview

`Life<'a>` turns the lifetime `'a` into a type, because CGP's wiring is parameterized by *types* and a CGP
trait may carry a lifetime of its own. The marker that records a provider's dependencies,
[`IsProviderFor`](../traits/wiring/is_provider_for.md), takes a tuple of the trait's generic parameters as
one type argument, so that the compiler can match a provider against the exact instantiation asked for. A
lifetime cannot sit in that tuple, because a tuple is a type and its members must be types. So a lifetime
parameter on the trait must first become a type. `Life<'a>` is that conversion. It packages the lifetime
`'a` as a concrete type that can stand beside the trait's other type parameters.

Without this lift, a trait that borrows could not record its lifetime in the dependency marker, and the
wiring could not tell one lifetime instantiation from another. `Life` lets the lifetime pass through
`IsProviderFor` as `(Life<'a>, T)`, so it stays part of the provider's identity while the marker's
argument stays a plain type.

## Definition

`Life` is a tuple struct wrapping a single phantom marker over a raw pointer to a borrowed unit:

```rust
pub struct Life<'a>(pub PhantomData<*mut &'a ()>);
```

The struct holds nothing at run time. Its only job is to carry the lifetime `'a` in the type system
through [`PhantomData`](phantom_data.md). The choice of `PhantomData<*mut &'a ()>` is deliberate,
because it controls how `Life<'a>` relates to other lifetimes under subtyping. A `*mut T` is
*invariant* in `T`, so wrapping `&'a ()` behind a `*mut` makes `Life<'a>` invariant in `'a`. A
`Life<'long>` is neither a subtype nor a supertype of a `Life<'short>`.

Invariance is correct here because the lifetime is an exact identity in the dependency marker. The
compiler must treat two providers wired for different lifetimes as wired for different things. A variant
`Life` would let the compiler coerce one instantiation into another and pick the wrong provider. The raw
pointer also means that `Life<'a>` does not carry auto-trait obligations tied to a real borrow, because
it neither owns nor references a real value.

## Behavior

`Life` does not define methods and does not implement CGP traits of its own. Its entire behavior is to
occupy a type position. In the generated provider trait for a component with a lifetime, the macro
collects the lifetime into the [`IsProviderFor`](../traits/wiring/is_provider_for.md) argument tuple as
`Life<'a>`, so the provider's dependency obligation reads the same way as for any type parameter. The
provider trait, its blanket forwarding impl, and the impls that satisfy it all agree on the same
`(Life<'a>, T)` shape. That shared shape lets CGP wire and check a borrowing component exactly like a
non-borrowing one.

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

Every impl that wires this component, whether through `UseContext`, a `UseField` getter, or a provider you
write yourself, carries the same `(Life<'a>, T)` tuple, so the resolution machinery preserves the lifetime
end to end.

## When to use it

**You read `Life` in generated code, and you do not write it.** The macros insert it, and you only need to
recognize it.

- **Read `Life<'a>` in an `IsProviderFor` tuple as the component's lifetime.** A dependency marker such
  as `IsProviderFor<..., (Life<'a>, T)>` names a lifetime and a type parameter, in that order.
- **Do not substitute a bare [`PhantomData`](phantom_data.md) for it** when you write a lifetime marker
  yourself. A plain `PhantomData<&'a ()>` is covariant, which is the wrong relationship for a dependency
  marker. `Life` forces the invariance that keeps two lifetime instantiations distinct.

## Common Mistakes

**A lifetime cannot appear directly in the `IsProviderFor` tuple.** The tuple holds types, so a bare
`'a` is invalid there, and the macro lifts it into `Life<'a>`. `Life` in an error means that the component
carries a lifetime.

**`Life<'a>` is invariant in `'a` by design.** It does not behave like `&'a ()`, which is covariant. The
invariance keeps the compiler from confusing providers wired for different lifetimes, so it is intended
rather than an over-restriction.

**A higher-order provider with a lifetime loses its dependency propagation.** When the component carries a
lifetime, the inner-provider bound of such a stack does not get a marker counterpart, because the rewrite
reads the bound's first generic argument as the context and finds a lifetime there. The stack still
compiles and runs. But it loses the propagation that lets `#[check_providers]` localize a broken layer.
[`IsProviderFor`](../traits/wiring/is_provider_for.md) records this limitation.

## Related constructs

- [`IsProviderFor`](../traits/wiring/is_provider_for.md): whose parameter tuple a lifetime is lifted
  into as `Life<'a>`.
- [`#[cgp_component]`](../macros/cgp_component.md): inserts `Life` when a consumer trait carries a
  lifetime.
- [`PhantomData`](phantom_data.md): the marker `Life` is built from, wrapped for invariance.
- [`Index`](index_type.md) and [`Chars`](chars.md): the other lifts that make a non-type
  addressable in trait resolution, a number and a string where `Life` lifts a lifetime.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): the provider-trait
  machinery whose dependency marker carries this lift.

## Source

- The type:
  [`life.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/life.rs)
- The macro logic that wraps a trait's lifetime parameters in `Life` when building the `IsProviderFor`
  argument tuple:
  [`is_provider_params.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/functions/is_provider_params.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
