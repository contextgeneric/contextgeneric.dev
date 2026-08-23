---
sidebar_label: 'UseType (provider)'
---

# `UseType` (provider)

Supply a concrete type as the value of an abstract-type component, binding it purely through wiring.

:::info

### Not the same as `#[use_type]`

This page documents the `UseType` **provider**, the struct `UseType<Type>`. It is a different construct
from the [`#[use_type]` attribute](../attributes/use_type.md), which rewrites bare type names inside a
definition and adds the owning trait as a bound. The attribute is about *referring to* an abstract type
ergonomically; the provider here is about *choosing the concrete type* an abstract type resolves to.
They share a name because both concern abstract types, but they live in different places and do
different jobs.

:::

## Overview

`UseType<Type>` removes the need to hand-write a provider every time a **context** wants to fix an
abstract type to a concrete one. The context is the type a capability runs against, and it decides what
each abstract type resolves to. An abstract type in CGP is a trait with a single associated type,
defined with [`#[cgp_type]`](../macros/cgp_type.md), such as `trait HasScalarType { type Scalar; }`.
Generic code refers to the type without committing to any particular one, and a concrete context
decides what it actually is. Without `UseType`, making that decision would mean writing a small
provider whose only content is `type Scalar = f64;`, repeated for every abstract type and every
concrete choice.

`UseType<T>` captures that trivial shape once. It is a [`TypeProvider`](../components/has_type.md) that
reports its type parameter `T` as the abstract type, so wiring a context's type component to
`UseType<f64>` sets the abstract type to `f64` with no custom impl. It is the type-level counterpart of
[`UseField`](use_field.md), which lets a getter read a field named by a parameter: a general provider
parameterized by exactly the thing the context wants to supply. Like every CGP provider, it carries no
runtime value.

## Usage

`UseType` is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the
concrete type to supply, and appears as the value of an abstract-type component's wiring entry:

```rust
delegate_components! {
    App {
        ScalarTypeProviderComponent: UseType<f64>,
    }
}
```

A `#[cgp_type]` trait such as `HasScalarType` generates the provider trait `ScalarTypeProvider` and the
component marker `ScalarTypeProviderComponent`; wiring that marker to `UseType<f64>` sets the context's
`Scalar` to `f64`. Any bound on the associated type, such as `type Scalar: Copy`, is enforced against
the concrete type at the wiring site.

`UseType<Type>` has an alias, `WithType<Type>`, which is
[`WithProvider<UseType<Type>>`](with_provider.md); it is imported from `cgp::core::types`. Both bind
the same type. Prefer the plain `UseType<Type>` form.

## Examples

A complete use defines an abstract type, wires a concrete type with `UseType`, and reads it back:

```rust
use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar: Copy;
}

pub struct App;

delegate_components! {
    App {
        ScalarTypeProviderComponent: UseType<f64>,
    }
}
```

`App` wires `ScalarTypeProviderComponent` to `UseType<f64>`, so `App` implements `HasScalarType` with
`Scalar = f64`. The `Copy` bound on the associated type is checked against `f64` where the wiring is
written.

The same binding can be written with the `WithType` alias, which routes through
[`WithProvider`](with_provider.md):

```rust
use cgp::core::types::WithType; // not in the prelude

delegate_components! {
    App {
        ScalarTypeProviderComponent: WithType<f64>,
    }
}
```

Both make `App::Scalar` resolve to `f64`.

## When to reach for it, and when not

**Reach for `UseType<T>` to bind an abstract type to a concrete one, which is the common case.** It is
the idiomatic value for a [`#[cgp_type]`](../macros/cgp_type.md) component's wiring entry, and it saves
writing a one-line provider by hand.

Reach for [`UseDelegatedType`](use_delegated_type.md) when the concrete type should itself come from a
lookup table rather than being fixed at the wiring site, which lets one entry answer several type
components at once. And do not confuse this provider with the
[`#[use_type]` attribute](../attributes/use_type.md), which imports an abstract type into a definition
and is unrelated to choosing the concrete type.

## Under the hood

`UseType<Type>` implements the built-in provider trait [`TypeProvider`](../components/has_type.md) for
every context and tag, setting its associated `Type` to the struct's own type parameter:

```rust
#[cgp_provider(TypeProviderComponent)]
impl<Context, Tag, Type> TypeProvider<Context, Tag> for UseType<Type> {
    type Type = Type;
}
```

The implementation is unconditional in `Context` and `Tag`: `UseType<f64>` is a `TypeProvider` whose
`Type` is `f64` regardless of which context or type tag asks. `HasType<Tag>` is the consumer trait that
reads this, so once a context's type component is wired to `UseType<f64>`, the context implements
`HasType<Tag>` with `Type = f64`.

The same provider is what [`#[cgp_type]`](../macros/cgp_type.md) targets. For
`#[cgp_type] trait HasScalarType { type Scalar; }`, the macro generates a `UseType` implementation for
the component's own provider trait:

```rust
impl<Scalar, __Context__> ScalarTypeProvider<__Context__> for UseType<Scalar> {
    type Scalar = Scalar;
}
```

so wiring `ScalarTypeProviderComponent` to `UseType<f64>` sets `Scalar = f64`. The built-in
`TypeProvider` impl and the per-component impl are the two faces of the same `UseType<Type>` struct: the
first makes it a provider for the built-in `HasType` component, the second for a user-defined
abstract-type component. A bound on the associated type is copied into the generated impl's `where`
clause, so the concrete type must satisfy it at the wiring site.

## Related constructs

- [`#[cgp_type]`](../macros/cgp_type.md) — defines the abstract type and generates the `UseType` impl
  that `UseType<T>` supplies the concrete type for.
- [`#[use_type]` attribute](../attributes/use_type.md) — the differently-named construct that imports an
  abstract type into a definition; do not confuse the two.
- [`HasType` / `TypeProvider`](../components/has_type.md) — the built-in abstract-type component this
  implements.
- [`UseDelegatedType`](use_delegated_type.md) — resolves an abstract type through a table instead of
  fixing it.
- [`WithProvider`](with_provider.md) — the adapter behind the `WithType` alias.
- [`UseField`](use_field.md) — the field-level analogue for getter components.

The ideas behind it:

- [Abstract types](/docs/concepts/abstract-types) — the associated types a context chooses for itself,
  which this provider supplies.

## Source

- Struct, `WithType` alias, and the built-in `TypeProvider` impl:
  [`use_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-type/src/impls/use_type.rs)
- The `HasType`/`TypeProvider` traits:
  [`has_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-type/src/traits/has_type.rs)
- The `#[cgp_type]`-generated `UseType` impl:
  [`cgp_type/item.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_type/item.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
