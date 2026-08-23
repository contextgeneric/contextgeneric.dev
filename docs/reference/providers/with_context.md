---
sidebar_label: 'WithContext'
sidebar_position: 6.1
---

# `WithContext`

The `WithProvider`-adapted spelling of `UseContext`, defined for completeness of the `With…` family.

## Overview

`WithContext` is the alias `WithProvider<UseContext>`. It pairs the [`WithProvider`](with_provider.md)
adapter with [`UseContext`](use_context.md) on a **context**, the type a capability runs against.
[`UseContext`](use_context.md) carries a foundational [`FieldGetter`](../traits/field_getter.md), and
[`WithProvider`](with_provider.md) adapts a foundational provider into a named component's provider, so
`WithContext` is the two composed. Like every CGP provider, it carries no runtime value.

`WithContext` is distinct from the bare [`UseContext`](use_context.md), and the two play different
roles. `UseContext` implements a component's provider trait *directly* by routing back to the context's
consumer trait, which is how it serves as a higher-order provider's default inner provider. `WithContext`
instead wraps `UseContext` in the [`WithProvider`](with_provider.md) adapter. It is the least-used member
of the `With…` family: the library defines it for completeness, and the common ways to serve a component
from the context's own state are the constructs named below rather than this alias.

## Usage

`WithContext` is in the prelude, so `use cgp::prelude::*;` is enough. It takes no type parameter. It is
the alias `WithProvider<UseContext>`, so it appears wherever `WithProvider<UseContext>` would, adapting
`UseContext`'s foundational implementation into a named component's provider.

For the everyday needs it is adjacent to, reach for one of these instead, all of which the codebase
exercises directly:

- an [`#[implicit]`](../attributes/implicit.md) argument, where a provider simply needs a value from its
  own context;
- [`UseFields`](use_fields.md) on a getter component, to read same-named fields;
- the bare [`UseContext`](use_context.md), as a higher-order provider's default inner provider.

## When to use it

**Reach for the constructs above rather than `WithContext` in practice.** The value it wraps is reached
more directly through [`UseContext`](use_context.md), [`UseFields`](use_fields.md), or an
[`#[implicit]`](../attributes/implicit.md) argument.

## Under the hood

`WithContext` is a type alias:

```rust
pub type WithContext = WithProvider<UseContext>;
```

The [`WithProvider`](with_provider.md) adapter forwards a component's provider-trait method to the inner
provider's foundational method, and [`UseContext`](use_context.md) supplies that foundational method,
its [`FieldGetter`](../traits/field_getter.md) reading the context's `HasField`. The generated
`WithProvider` impl that makes this work is shown on the [`WithProvider`](with_provider.md) page.

## Related constructs

- [`WithProvider`](with_provider.md) — the adapter this alias specializes.
- [`UseContext`](use_context.md) — the inner provider it wraps, whose direct form serves as a
  higher-order provider's inner default.
- [`WithField`](with_field.md), [`WithType`](with_type.md) — the sibling aliases for a specific field or
  concrete type.
- [`UseFields`](use_fields.md) — reads same-named fields on a getter component, one of the direct forms
  to prefer.
- [`FieldGetter`](../traits/field_getter.md) — the foundational getter `UseContext` supplies here.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the two-way split
  `UseContext` bridges.

## Source

- The alias:
  [`use_context.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/use_context.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
