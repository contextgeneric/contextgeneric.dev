---
sidebar_label: 'WithDelegatedType'
sidebar_position: 6.5
---

# `WithDelegatedType`

Resolve one or more abstract-type components from a shared table, through the `WithProvider` adapter.

## Overview

`WithDelegatedType<Components>` is the alias `WithProvider<UseDelegatedType<Components>>`. It answers a
context's abstract-type components by looking each type tag up in the `Components` table, on a
**context**, the type a capability runs against, by adapting the foundational
[`UseDelegatedType`](use_delegated_type.md) provider through the [`WithProvider`](with_provider.md)
layer. Like every CGP provider, it carries no runtime value.

Unlike [`WithType`](with_type.md), this alias is not an alternative to a directly-wireable provider: the
bare [`UseDelegatedType`](use_delegated_type.md) supplies only the foundational
[`TypeProvider`](../components/has_type.md), so `WithDelegatedType` is the form you wire.

## Usage

`WithDelegatedType` is not in the prelude. Import it from `cgp::core::types`. It takes one type
parameter, the lookup table, and appears as the value of one or more abstract-type components' wiring
entries:

```rust
use cgp::core::types::WithDelegatedType;

delegate_components! {
    App {
        [
            ScalarTypeProviderComponent,
            IndexTypeProviderComponent,
        ]: WithDelegatedType<AppTypes>,
    }
}
```

The `Components` parameter is a type that maps each type tag to a concrete type, built as an ordinary
[`delegate_components!`](../macros/delegate_components.md) table. Any tag the table has no entry for is
unresolved, and the context does not implement `HasType` for it.

## Examples

A lookup table maps type tags to concrete types, and a context wires several type components through it:

```rust
use cgp::core::types::WithDelegatedType;
use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

#[cgp_type]
pub trait HasIndexType {
    type Index;
}

pub struct App;
pub struct AppTypes;

delegate_components! {
    AppTypes {
        ScalarTypeProviderComponent: f64,
        IndexTypeProviderComponent: usize,
    }
}

delegate_components! {
    App {
        [
            ScalarTypeProviderComponent,
            IndexTypeProviderComponent,
        ]: WithDelegatedType<AppTypes>,
    }
}
```

`App` routes both its scalar and index type components through `WithDelegatedType<AppTypes>`. For `App`'s
`Scalar` the provider finds `f64` in `AppTypes`, and for `Index` it finds `usize`. One provider entry on
`App` answers two abstract types, with the concrete choices held in one place where they can be reused,
swapped, or supplied by a namespace.

## When to use it

**Reach for `WithDelegatedType` when several abstract types should be answered from one shared table**,
so a context points its type components at the table rather than fixing each one at the wiring site.

Prefer [`WithType<T>`](with_type.md) or the plain [`UseType<T>`](use_type.md) for the common case of
fixing one abstract type to one concrete type, which needs no table.

## Under the hood

`WithDelegatedType<Components>` is a type alias:

```rust
pub type WithDelegatedType<Components> = WithProvider<UseDelegatedType<Components>>;
```

The [`WithProvider`](with_provider.md) adapter forwards a type component's provider-trait method to the
inner provider's foundational method, and [`UseDelegatedType<Components>`](use_delegated_type.md) is the
foundational [`TypeProvider`](../components/has_type.md) that resolves each tag through a
[`DelegateComponent`](../traits/delegate_component.md) lookup. The `UseDelegatedType` mechanism is
documented on the [`UseDelegatedType`](use_delegated_type.md) page, and the generated `WithProvider` impl
on the [`WithProvider`](with_provider.md) page.

## Related constructs

- [`WithProvider`](with_provider.md) — the adapter this alias specializes.
- [`UseDelegatedType`](use_delegated_type.md) — the foundational provider it wraps, where the table
  lookup is documented.
- [`WithType`](with_type.md) — the sibling alias for a single fixed concrete type.
- [`DelegateComponent`](../traits/delegate_component.md) — the table the lookup reads.
- [`HasType` / `TypeProvider`](../components/has_type.md) — the abstract-type component it answers for.
- [`#[cgp_type]`](../macros/cgp_type.md) — defines the abstract-type components it resolves.

The ideas behind it:

- [Abstract types](/docs/concepts/abstract-types) — the associated types a context chooses for itself,
  here from a shared table.

## Source

- The alias:
  [`use_delegated_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-type/src/impls/use_delegated_type.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
