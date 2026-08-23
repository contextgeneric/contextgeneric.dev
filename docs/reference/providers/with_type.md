---
sidebar_label: 'WithType'
sidebar_position: 6.2
---

# `WithType`

Bind an abstract-type component to a concrete type, through the `WithProvider` adapter.

## Overview

`WithType<Type>` is the alias `WithProvider<UseType<Type>>`. It binds an abstract-type component to the
concrete `Type` on a **context**, the type a capability runs against, by adapting the foundational
[`UseType<Type>`](use_type.md) provider through the [`WithProvider`](with_provider.md) layer. It sets
the same abstract type the plain [`UseType`](use_type.md) provider does, and both are interchangeable in
wiring. Like every CGP provider, it carries no runtime value.

## Usage

`WithType` is not in the prelude. Import it from `cgp::core::types`. It takes one type parameter, the
concrete type to supply, and appears as the value of an abstract-type component's wiring entry:

```rust
use cgp::core::types::WithType;

delegate_components! {
    App {
        ScalarTypeProviderComponent: WithType<f64>,
    }
}
```

This sets the context's `Scalar` to `f64`, exactly as wiring [`UseType<f64>`](use_type.md) would.

## Examples

A complete use defines an abstract type and wires a concrete type through the alias:

```rust
use cgp::core::types::WithType;
use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar: Copy;
}

pub struct App;

delegate_components! {
    App {
        ScalarTypeProviderComponent: WithType<f64>,
    }
}
```

`App` wires `ScalarTypeProviderComponent` to `WithType<f64>`, so `App` implements `HasScalarType` with
`Scalar = f64`. The `Copy` bound on the associated type is checked against `f64` where the wiring is
written.

## When to use it

**Prefer the plain [`UseType<T>`](use_type.md) form.** It binds the same type and is the idiomatic
value for a [`#[cgp_type]`](../macros/cgp_type.md) component's wiring entry. `WithType` exists for the
case where a component is reached only through the [`WithProvider`](with_provider.md) adapter, and it
reads as a single wiring choice where spelling out `WithProvider<UseType<f64>>` would not.

For an abstract type resolved through a table rather than fixed to one concrete type, use
[`WithDelegatedType`](with_delegated_type.md).

## Under the hood

`WithType<Type>` is a type alias:

```rust
pub type WithType<Type> = WithProvider<UseType<Type>>;
```

The [`WithProvider`](with_provider.md) adapter forwards a component's provider-trait method to the
inner provider's foundational method, and [`UseType<Type>`](use_type.md) is the foundational
[`TypeProvider`](../components/has_type.md) that reports `Type` as the abstract type. The generated
`WithProvider` impl that makes this work is shown on the [`WithProvider`](with_provider.md) page.

## Related constructs

- [`WithProvider`](with_provider.md) — the adapter this alias specializes.
- [`UseType`](use_type.md) — the inner provider it wraps, and the plain form to prefer.
- [`WithDelegatedType`](with_delegated_type.md) — the sibling alias for a table-resolved type.
- [`#[cgp_type]`](../macros/cgp_type.md) — defines the abstract-type component and generates the
  `WithProvider` impl.
- [`HasType` / `TypeProvider`](../components/has_type.md) — the built-in abstract-type component this
  binds.

The ideas behind it:

- [Abstract types](/docs/concepts/abstract-types) — the associated types a context chooses for itself,
  which this alias supplies.

## Source

- The alias:
  [`use_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-type/src/impls/use_type.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
