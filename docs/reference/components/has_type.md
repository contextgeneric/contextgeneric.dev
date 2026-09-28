---
title: 'HasType — a tag-indexed abstract type'
description: 'CGP''s tag-indexed abstract-type component: a context resolves one type per tag through wiring, and generic code names it through TypeOf.'
sidebar_label: 'HasType'
sidebar_position: 4
---

# `HasType`

CGP's tag-indexed abstract-type component: a type per tag that a context resolves through wiring.

## Overview

`HasType<Tag>` lets generic code refer to a type that is chosen per context without committing to a
concrete one. An [abstract type](/docs/reference/glossary#abstract-type) in CGP is a
[trait](https://doc.rust-lang.org/book/ch10-02-traits.html) with a single
[associated type](https://doc.rust-lang.org/reference/items/associated-items.html), and `HasType<Tag>`
is the foundational, tag-indexed instance of that pattern. A **context**, the type that implements the trait, can carry many distinct abstract types, one per `Tag`, and resolve each to a concrete type
through wiring. Generic code names `Self::Type` (or the alias `TypeOf<Context, Tag>`), the concrete type
stays hidden behind the tag, and any context that wires the tag to a type satisfies the bound.

`HasType` is the one abstract-type component in CGP indexed by a tag rather than named. CGP's other
abstract types, such as [`HasErrorType`](./has_error_type.md) and
[`HasRuntimeType`](./has_runtime_type.md), are named components defined with
[`#[cgp_type]`](../macros/cgp_type.md), each with its own provider trait and wiring key. The two kinds share the
[`UseType<T>`](../providers/use_type.md) marker, because `#[cgp_type]` generates a `UseType` impl for
each named component. They also meet at the provider level: `#[cgp_type]` generates a
[`WithProvider`](../providers/with_provider.md) impl that lets a `TypeProvider` back a named
component.

## Definition

`HasType` is defined as:

```rust
#[cgp_component(TypeProvider)]
#[derive_delegate(UseDelegate<Tag>)]
pub trait HasType<Tag> {
    type Type;
}

pub type TypeOf<Context, Tag> = <Context as HasType<Tag>>::Type;
```

Its attributes:

- [`#[cgp_component]`](../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `TypeProvider` that implementations target and the wiring key `TypeProviderComponent`, while `HasType` stays the consumer trait callers use.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates a `UseDelegate` provider that dispatches on the `Tag` type, so a context can route each `Tag` to its own provider; the `open` statement is the modern sugar for the same dispatch.

## Usage

`HasType` and the provider trait `TypeProvider` are in the prelude. The wiring key
`TypeProviderComponent` and the `TypeOf` alias come from `cgp::core::types`. `HasType` carries a
`Tag` parameter, the type-level name that distinguishes one abstract type from another within a
context, and an associated `Type` that the tag resolves to. A context supplies the type in one of
two ways.

The direct way is an ordinary trait impl. The wired way, and the common one, delegates the component's
key `TypeProviderComponent` to [`UseType<T>`](../providers/use_type.md), the zero-sized marker that
reports its own parameter as the abstract type:

```rust
delegate_components! {
    App {
        TypeProviderComponent: UseType<f64>,
    }
}
```

**`UseType<T>` ignores the tag**, so that single entry answers every tag with `f64`. To give different
tags different types, dispatch on the tag with the
[`open` statement](../macros/delegate_components.md#choosing-a-provider-per-type-the-open-statement):

```rust
delegate_components! {
    App {
        open TypeProviderComponent;

        @TypeProviderComponent.ScalarTag: UseType<f64>,
        @TypeProviderComponent.NameTag: UseType<String>,
    }
}
```

The legacy form wires `TypeProviderComponent` to a `UseDelegate` table keyed by tag, or to
[`UseDelegatedType`](../providers/use_delegated_type.md), whose table maps each tag straight to its type.

In most code you would not wire `HasType<Tag>` directly. You would define a named abstract type with
[`#[cgp_type]`](../macros/cgp_type.md), such as `#[cgp_type] trait HasScalarType { type Scalar; }`,
which gives a readable `Self::Scalar` and its own provider trait and marker. The related
[`#[use_type]`](../attributes/use_type.md) attribute, distinct from the `UseType` provider despite
the shared name, rewrites a bare type name and adds the bound inside a definition.

## Examples

A direct use defines no new component and resolves an abstract type by tag through `UseType`:

```rust
use cgp::core::types::{TypeOf, TypeProviderComponent};
use cgp::prelude::*;

pub struct ScalarTag;

pub struct App;

delegate_components! {
    App {
        TypeProviderComponent: UseType<f64>,
    }
}

check_components! {
    App {
        TypeProviderComponent: ScalarTag,
    }
}

fn zero<Context>() -> TypeOf<Context, ScalarTag>
where
    Context: HasType<ScalarTag>,
    TypeOf<Context, ScalarTag>: Default,
{
    Default::default()
}
```

`App` wires `TypeProviderComponent` to `UseType<f64>`, so the `UseType` impl makes `App` implement
`HasType<ScalarTag>` with `Type = f64`. `App` is an **[environmental context](/docs/reference/glossary#environmental-context)** here, a type standing for
the application that carries its type choices. `zero::<App>()` returns `0.0`. In real code you would
usually define a named `#[cgp_type]` component instead.

## When to use it

**You rarely name `HasType` directly. Reach for [`#[cgp_type]`](../macros/cgp_type.md) instead**, which
defines a named abstract type with its own readable trait, and let it build on this substrate. The one
time `HasType<Tag>` is useful on its own is when a program wants a family of abstract types keyed by a
tag it computes rather than a fixed set of named traits.

Recognizing `HasType` matters more than writing it. Its `TypeProvider` trait appears in the
`WithProvider` impl every `#[cgp_type]` component carries, so an expansion or a diagnostic about a
named abstract type can mention it.

## Related constructs

- [`#[cgp_type]`](../macros/cgp_type.md) — defines named abstract-type components, whose generated
  `WithProvider` impl accepts a `TypeProvider`.
- [`UseType`](../providers/use_type.md) — the zero-sized marker that supplies a concrete type to the
  abstract one.
- [`#[use_type]`](../attributes/use_type.md) — the differently-named attribute that imports an abstract
  type into a definition; do not confuse the two.
- [`UseDelegatedType`](../providers/use_delegated_type.md) — resolves an abstract type through a table
  rather than fixing it.
- [`WithProvider`](../providers/with_provider.md) — the adapter that lets a `TypeProvider` stand in as a
  named component's provider.
- [`HasErrorType`](./has_error_type.md) — a concrete abstract-type component built on this one.

The ideas behind it:

- [Abstract types](/docs/concepts/abstract-types) — the associated types a context chooses for itself,
  and why a determined type propagates nowhere while a parameter propagates everywhere.

## Source

- The trait, the `TypeProvider` provider trait, and the `TypeOf` alias:
  [`has_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-type/src/traits/has_type.rs)
- The `UseType` provider and its `TypeProvider` impl:
  [`use_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-type/src/impls/use_type.rs)
- The `#[cgp_type]` macro that defines named abstract-type components:
  [`cgp_type/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_type/)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
