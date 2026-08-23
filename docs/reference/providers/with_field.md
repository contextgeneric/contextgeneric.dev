---
sidebar_label: 'WithField'
sidebar_position: 6.3
---

# `WithField`

Wire a getter component to a named context field, through the `WithProvider` adapter.

## Overview

`WithField<Tag>` is the alias `WithProvider<UseField<Tag>>`. It implements a getter component by reading
the context field named by `Tag`, on a **context**, the type a capability runs against, by adapting the
foundational [`UseField<Tag>`](use_field.md) getter through the [`WithProvider`](with_provider.md) layer.
It reads the same field the plain [`UseField`](use_field.md) provider does, and both are interchangeable
in wiring. Like every CGP provider, it carries no runtime value.

## Usage

`WithField` is not in the prelude. Import it from `cgp::core::field::impls`. It takes one type parameter,
the field tag, and appears as the value of a getter component's wiring entry:

```rust
use cgp::core::field::impls::WithField;

delegate_components! {
    Person {
        NameGetterComponent: WithField<Symbol!("first_name")>,
    }
}
```

The tag is a [`Symbol!`](../macros/symbol.md) for a named field or an `Index<N>` for a tuple field, and
the context must have a [`HasField`](../traits/field-access/has_field.md) implementation for it. This reads the
`first_name` field exactly as wiring [`UseField<Symbol!("first_name")>`](use_field.md) would.

## Examples

A getter whose method name differs from the field it reads, wired through the alias:

```rust
use cgp::core::field::impls::WithField;
use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct Person {
    pub first_name: String,
}

delegate_components! {
    Person {
        NameGetterComponent: WithField<Symbol!("first_name")>,
    }
}
```

`WithField<Symbol!("first_name")>` expands to `WithProvider<UseField<Symbol!("first_name")>>`, so
`person.name()` reads the `first_name` field. The field name lives in the wiring, not in the trait.

## When to use it

**Prefer the plain [`UseField<Tag>`](use_field.md) form.** It reads the same field and is the value a
[`#[cgp_getter]`](../macros/cgp_getter.md) component is normally wired to. `WithField` exists for the
case where a component is reached only through the [`WithProvider`](with_provider.md) adapter, and it
reads as a single wiring choice where spelling out `WithProvider<UseField<Tag>>` would not.

For the common case of reading a field, an [`#[implicit]`](../attributes/implicit.md) argument is
simpler than any getter provider. For a stored type that borrows to the getter's return type through
`AsRef`, use [`WithFieldRef`](with_field_ref.md).

## Under the hood

`WithField<Tag>` is a type alias:

```rust
pub type WithField<Tag> = WithProvider<UseField<Tag>>;
```

The [`WithProvider`](with_provider.md) adapter forwards a getter's provider-trait method to the inner
provider's foundational method, and [`UseField<Tag>`](use_field.md) is the foundational
[`FieldGetter`](../traits/field-access/field_getter.md) that reads the field at `Tag`. The generated `WithProvider`
impl that makes this work is shown on the [`WithProvider`](with_provider.md) page.

## Related constructs

- [`WithProvider`](with_provider.md) — the adapter this alias specializes.
- [`UseField`](use_field.md) — the inner provider it wraps, and the plain form to prefer.
- [`WithFieldRef`](with_field_ref.md) — the sibling alias for a field borrowed through `AsRef`.
- [`#[cgp_getter]`](../macros/cgp_getter.md) — defines the getter component and generates the
  `WithProvider` impl.
- [`HasField`](../traits/field-access/has_field.md) and [`#[derive(HasField)]`](../derives/derive_has_field.md) — the
  field access it reads, keyed by [`Symbol!`](../macros/symbol.md) or `Index<N>`.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — value injection through a
  provider's own requirements, of which reading a field is the most common form.

## Source

- The alias:
  [`use_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/use_field.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
