---
sidebar_label: 'WithProvider'
sidebar_position: 6
---

# `WithProvider`

Adapt a foundational provider, one implementing `TypeProvider` or `FieldGetter`, into a provider of a
specific named component.

## Overview

`WithProvider<Provider>` bridges CGP's two layers of provider trait. Foundational traits like
[`TypeProvider`](../components/has_type.md) and [`FieldGetter`](../traits/field_getter.md) are generic,
component-agnostic mechanisms: a `TypeProvider` supplies *some* abstract type for *some* tag, and a
`FieldGetter` reads *some* field for *some* output tag, without either knowing which named component it
serves. A named component, by contrast, has a specific provider trait, such as `NameTypeProvider` or
`NameGetter`, that a **context** wires to, where the context is the type the capability runs against.
`WithProvider<Provider>` is the adapter that lets a foundational provider stand in as the provider for
one of those named components: it implements the component's provider trait by forwarding to the
foundational provider's method.

This adapter is what lets the foundational layer be wired without each foundational provider
implementing every component trait by hand. A field getter written once as a `FieldGetter` can serve any
number of getter components through `WithProvider`, and an abstract-type implementation written once as
a `TypeProvider` can serve any type component the same way.

`WithProvider` is rarely written in full, because its common uses are packaged as aliases. The family
`WithContext`, `WithType`, `WithField`, `WithFieldRef`, and `WithDelegatedType` are each
`WithProvider<...>` specialized to a particular inner provider, and those aliases are what appear in
everyday wiring. Understanding `WithProvider` is what explains why the aliases work. Like every CGP
provider, it carries no runtime value.

## Usage

`WithProvider` is in the prelude, so `use cgp::prelude::*;` is enough. It takes one type parameter, the
foundational provider to adapt, and appears as the value of a named component's wiring entry. In
practice you write one of its aliases instead:

| Alias | Expands to | Import from |
|---|---|---|
| `WithContext` | `WithProvider<UseContext>` | prelude |
| `WithType<Type>` | `WithProvider<UseType<Type>>` | `cgp::core::types` |
| `WithField<Tag>` | `WithProvider<UseField<Tag>>` | `cgp::core::field::impls` |
| `WithFieldRef<Tag, Value>` | `WithProvider<UseFieldRef<Tag, Value>>` | `cgp::core::field::impls` |
| `WithDelegatedType<Components>` | `WithProvider<UseDelegatedType<Components>>` | `cgp::core::types` |

Each alias wires the inner provider it names as a specific component's provider. A component becomes
adaptable this way only when its macro generates the `WithProvider` impl:
[`#[cgp_type]`](../macros/cgp_type.md) does so for every abstract-type component, and
[`#[cgp_getter]`](../macros/cgp_getter.md) does so for a getter with exactly one method.

## Examples

The everyday way to use `WithProvider` is through one of its aliases, which reads as a single wiring
choice. Adapting the context's own field getter into a getter component uses `WithField`:

```rust
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
`Person`'s `NameGetter` provider is the adapter wrapping the foundational
[`UseField`](use_field.md) getter for the `first_name` field. The generated `WithProvider` impl forwards
`name()` to `UseField`'s `FieldGetter::get_field`, which reads `first_name`. The same shape recurs for
abstract types: wiring a type component to `WithType<String>` adapts [`UseType<String>`](use_type.md),
and to `WithDelegatedType<SomeTable>` adapts a [`UseDelegatedType`](use_delegated_type.md) that resolves
the type through a table.

## When to reach for it, and when not

**Reach for one of the aliases, not `WithProvider` directly.** `WithField`, `WithType`, `WithContext`,
`WithFieldRef`, and `WithDelegatedType` are the readable forms, and they are interchangeable with the
plain providers they wrap: wiring a getter to `WithField<Tag>` and to
[`UseField<Tag>`](use_field.md) both read the same field. Prefer the plain provider
([`UseField`](use_field.md), [`UseType`](use_type.md)) where it applies, and the `With...` alias where a
component is reached only through the `WithProvider` adapter.

Write `WithProvider<Provider>` in full only when adapting a foundational provider that has no ready
alias.

## Under the hood

[`#[cgp_type]`](../macros/cgp_type.md) and [`#[cgp_getter]`](../macros/cgp_getter.md) generate a
`WithProvider` impl that forwards a component's provider-trait method to the inner provider's
foundational method. For a type component such as

```rust
#[cgp_type]
pub trait HasNameType {
    type Name;
}
```

`#[cgp_type]` emits a `WithProvider` impl that defines the component's associated type from the inner
`TypeProvider` (shown with the macro's real placeholder identifiers):

```rust
impl<__Provider__, Name, __Context__> NameTypeProvider<__Context__> for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, NameTypeProviderComponent, Type = Name>,
{
    type Name = Name;
}
```

For a single-method getter, `#[cgp_getter]` emits an analogous impl that reads the value through the
inner `FieldGetter`:

```rust
impl<__Context__, __Provider__> NameGetter<__Context__> for WithProvider<__Provider__>
where
    __Provider__: FieldGetter<__Context__, NameGetterComponent, Value = String>,
{
    fn name(__context__: &__Context__) -> &str {
        __Provider__::get_field(__context__, PhantomData::<NameGetterComponent>).as_str()
    }
}
```

In both cases the bound names the foundational trait, keyed by the component marker, and the method or
associated type forwards to it. `#[cgp_getter]` generates the `WithProvider` impl only when the getter
has exactly one method, since a single foundational getter cannot serve several methods at once. Each
impl is paired with a matching [`IsProviderFor`](../traits/is_provider_for.md) impl.

The aliases specialize `WithProvider` to a fixed inner provider so the common cases need no
`WithProvider<...>` spelled out. `WithContext = WithProvider<UseContext>` adapts the context's own
consumer-trait implementation; `WithType<Type>` and `WithField<Tag>` adapt the foundational type and
field providers; `WithFieldRef<Tag, Value>` adapts a getter that borrows through `AsRef`; and
`WithDelegatedType<Components>` adapts a type provider that looks its type up in a table. Each alias
lives beside the inner provider it wraps.

## Related constructs

- [`#[cgp_type]`](../macros/cgp_type.md) and [`#[cgp_getter]`](../macros/cgp_getter.md) — generate the
  `WithProvider` impls that make a component adaptable.
- [`TypeProvider`](../components/has_type.md) and [`FieldGetter`](../traits/field_getter.md) — the
  foundational traits it adapts.
- [`UseContext`](use_context.md), [`UseType`](use_type.md), [`UseField`](use_field.md),
  [`UseFieldRef`](use_field_ref.md), and [`UseDelegatedType`](use_delegated_type.md) — the inner
  providers its aliases wrap.
- [`delegate_components!`](../macros/delegate_components.md) — wires the aliases, and
  [`check_components!`](../macros/check_components.md) verifies them.

The ideas behind it:

- [Abstract types](/docs/concepts/abstract-types) — where the `TypeProvider` layer this adapts is used.
- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the provider-trait layer
  the adapter bridges.

## Source

- Struct:
  [`with_provider.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/with_provider.rs);
  the `WithContext` alias in
  [`use_context.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/use_context.rs)
- The `WithType`/`WithDelegatedType` aliases in
  [`cgp-type/src/impls/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-type/src/impls/),
  and `WithField`/`WithFieldRef` in
  [`cgp-field/src/impls/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-field/src/impls/)
- The generated `WithProvider` impls:
  [`cgp_type/item.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_type/item.rs)
  and
  [`cgp_getter/with_provider.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_getter/with_provider.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
