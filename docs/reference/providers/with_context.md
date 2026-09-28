---
title: 'WithContext — the context''s own entry'
description: 'The alias WithProvider<UseContext>: a named type or getter component answered by the context''s own HasType or HasField entry, keyed by the component.'
sidebar_label: 'WithContext'
sidebar_position: 6.1
---

# `WithContext`

The `WithProvider`-adapted spelling of `UseContext`: answer a named type or getter component from the
context's own generic `HasType` or `HasField` entry.

## Overview

`WithContext` is the alias `WithProvider<UseContext>`. It pairs the [`WithProvider`](with_provider.md)
adapter with [`UseContext`](use_context.md), which implements the two foundational traits by asking
the [**context**](/docs/reference/glossary#context), the type the implementation runs against:
[`TypeProvider`](../components/has_type.md) through the context's own `HasType<Tag>`, and
[`FieldGetter`](../traits/field-access/field_getter.md) through its own `HasField<Tag>`.
`WithProvider` passes the component's marker as that tag, so wiring a component to `WithContext`
answers it from the context's `HasType` or `HasField` entry keyed by the component itself. Like every
CGP provider, it carries no runtime value.

`WithContext` is distinct from the bare [`UseContext`](use_context.md), and the two are not
interchangeable. `UseContext` implements a component's provider trait *directly*, by calling the
context's consumer trait for that same component, which is how it serves as a
[higher-order provider](/docs/reference/glossary#higher-order-provider)'s default inner provider and
why wiring a component to it loops. `WithContext` goes through the foundational traits instead, so it
reaches a *different* entry: the generic `HasType` or `HasField` table the context already has. It is
the least-used member of the `With…` family.

## Usage

`WithContext` is in the prelude, so `use cgp::prelude::*;` is enough. It takes no type parameter and
appears as the value of a type component's or single-method getter's wiring entry:

```rust
delegate_components! {
    App {
        TypeProviderComponent: UseType<String>,
        NameTypeProviderComponent: WithContext,
    }
}
```

What the entry needs from the context depends on the component's kind:

- **A [`#[cgp_type]`](../macros/cgp_type.md) component**, such as `HasNameType`, resolves its type
  through the context's `HasType<NameTypeProviderComponent>`. The built-in `TypeProviderComponent`,
  imported from `cgp::core::types`, is the usual way to supply it; wired to `UseType<String>`, it
  answers every tag with `String`.
- **A single-method [`#[cgp_getter]`](../macros/cgp_getter.md) component**, such as `HasName`, reads
  the context's `HasField<NameGetterComponent>` entry. No derive generates a `HasField` impl keyed by a
  component marker, so the context implements it by hand.

## Examples

A context supplies one type for every tag through the built-in `HasType`, and routes a named type
component to that answer with `WithContext`:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::core::types::TypeProviderComponent;

#[cgp_type]
pub trait HasNameType {
    type Name;
}

pub struct App;

delegate_components! {
    App {
        TypeProviderComponent: UseType<String>,
        NameTypeProviderComponent: WithContext,
    }
}

check_components! {
    App {
        NameTypeProviderComponent,
    }
}

pub fn name_type(name: PhantomData<<App as HasNameType>::Name>) -> PhantomData<String> {
    name
}
```

`App` resolves `HasNameType::Name` by asking its own `HasType<NameTypeProviderComponent>`, which the
`TypeProviderComponent: UseType<String>` entry answers with `String`, so `name_type` compiles only
because `Name` is `String`. `App` is an
[environmental context](/docs/reference/glossary#environmental-context) with no fields.

## When to use it

**Reach for `WithContext` when a context already answers a generic `HasType` or `HasField` lookup and
a named component should take that answer.** That is a narrow arrangement, so the adjacent constructs
usually fit better:

- [`UseType<T>`](use_type.md) on the named type component, to fix its type directly;
- [`UseFields`](use_fields.md) or [`UseField`](use_field.md) on a getter component, to read a named
  field;
- an [`#[implicit]`](../attributes/implicit.md) argument, where a provider needs a value from its own
  context;
- the bare [`UseContext`](use_context.md), as a higher-order provider's default inner provider.

## Under the hood

`WithContext` is a type alias:

```rust
pub type WithContext = WithProvider<UseContext>;
```

The [`WithProvider`](with_provider.md) impl that `#[cgp_type]` generates bounds its inner provider by
`TypeProvider<__Context__, NameTypeProviderComponent>`, and the one `#[cgp_getter]` generates bounds it
by `FieldGetter<__Context__, NameGetterComponent>`. `UseContext` implements both by asking the
context, the first through the `UseContext` impl `#[cgp_component]` generates for `HasType` and the
second through a hand-written impl beside `FieldGetter`:

```rust
impl<Context, Tag, Field> FieldGetter<Context, Tag> for UseContext
where
    Context: HasField<Tag, Value = Field>,
{
    type Value = Field;

    fn get_field(context: &Context, _tag: PhantomData<Tag>) -> &Self::Value {
        context.get_field(PhantomData)
    }
}
```

With the component's marker as `Tag`, the lookups are `Context: HasType<NameTypeProviderComponent>`
and `Context: HasField<NameGetterComponent>`.

## Related constructs

- [`WithProvider`](with_provider.md) — the adapter this alias specializes.
- [`UseContext`](use_context.md) — the inner provider it wraps, whose direct form serves as a
  higher-order provider's inner default.
- [`HasType` / `TypeProvider`](../components/has_type.md) — the generic type lookup a type component
  reaches through it.
- [`FieldGetter`](../traits/field-access/field_getter.md) and
  [`HasField`](../traits/field-access/has_field.md) — the field lookup a getter component reaches
  through it.
- [`WithField`](with_field.md), [`WithType`](with_type.md) — the sibling aliases for a specific field or
  concrete type.

The ideas behind it:

- [Abstract types](/docs/concepts/abstract-types) — the tag-indexed `HasType` component this routes a
  named type component through.

## Source

- The alias:
  [`use_context.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/use_context.rs)
- `UseContext`'s `FieldGetter` impl:
  [`has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
