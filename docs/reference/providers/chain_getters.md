---
sidebar_label: 'ChainGetters'
---

# `ChainGetters`

Compose a list of field getters into one, reaching a field several hops inside a nested context.

## Overview

`ChainGetters<Getters>` reaches a field that does not live directly on the context but several levels
inside it. The **context** is the type a capability runs against, and a single
[`UseField`](use_field.md) reads one field of one context. But CGP contexts often nest: a context holds
a config, the config holds a connection, the connection holds a timeout, and a getter may need the
innermost value. Writing one provider that walks the whole path by hand is tedious and couples the
getter to the nesting. `ChainGetters<Getters>` takes a list of getters, applies them in order, and
threads the reference from each step into the next, so the chain reads like the path it traverses:
outer getter, then the next, ending at the target field.

`ChainGetters` is a foundational [`FieldGetter`](../traits/field_getter.md), so it is wired to a getter
component through the [`WithProvider`](with_provider.md) adapter rather than named on its own. The list
is a type-level [`Cons`](../types/type_level_spines.md) spine whose elements are each a field getter for
the value the previous step produced, written with the [`Product!`](../macros/product.md) macro.
`ChainGetters` recurses down the list. Like every CGP provider, it carries no runtime value: it is a
marker named in wiring.

## Usage

`ChainGetters` is not in the prelude. Import it from `cgp::core::field::impls`:

```rust
use cgp::core::field::impls::ChainGetters;
```

It takes one type parameter, a [`Product!`](../macros/product.md) list of getters, and appears wrapped
in [`WithProvider`](with_provider.md) as the value of a getter component's wiring entry:

```rust
delegate_components! {
    App {
        NameGetterComponent: WithProvider<
            ChainGetters<Product![
                UseField<Symbol!("config")>,
                UseField<Symbol!("name")>,
            ]>,
        >,
    }
}
```

Each element of the list is a field getter, usually [`UseField`](use_field.md), for the value the
previous step produced. The value the last step reads must match what the getter returns, the same way
any getter provider must. Every step is asked under the same tag, so the chain navigates *contexts*
rather than different field names per step, and each getter in the list decides for itself which field
of its input it reads. `ChainGetters` has no dedicated alias; wrap it in `WithProvider` directly.

## Examples

A typical use reaches a field on a nested inner context by chaining the getter that produces the inner
context with the getter that reads the field. Here an `App` holds a `Config`, and the target is the
`Config`'s `name`:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::ChainGetters; // not in the prelude

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct Config {
    pub name: String,
}

#[derive(HasField)]
pub struct App {
    pub config: Config,
}

delegate_components! {
    App {
        NameGetterComponent: WithProvider<
            ChainGetters<Product![
                UseField<Symbol!("config")>,
                UseField<Symbol!("name")>,
            ]>,
        >,
    }
}
```

`App` wires `NameGetterComponent` to `WithProvider<ChainGetters<...>>` over a two-element list. The
first getter, `UseField<Symbol!("config")>`, reads `App`'s `config` field to produce a `&Config`; the
second, `UseField<Symbol!("name")>`, reads that `Config`'s `name` field to produce the `&String` the
`&str` getter borrows from. `ChainGetters` threads the reference from the first step into the second, so
`app.name()` returns the name nested two levels in, with no hand-written walking code.

## When to reach for it, and when not

**Reach for `ChainGetters` when a getter's value lives on a nested inner context** rather than on the
context directly, so the getter has to walk through one or more intermediate values to reach it.

For a field on the context itself, prefer an [`#[implicit]`](../attributes/implicit.md) argument, or
[`UseField`](use_field.md) when the field name must be a wiring decision. `ChainGetters` earns its place
only once the value is more than one hop away.

## Under the hood

`ChainGetters` implements the provider-side getter [`FieldGetter`](../traits/field_getter.md) with two
impls that together recurse over the list, one for a non-empty `Cons` and one for the empty `Nil`. The
`Cons` impl applies the head getter, then delegates the rest of the path to `ChainGetters` over the
tail:

```rust
impl<Context, Tag, Getter, RestGetters, ValueA, ValueB> FieldGetter<Context, Tag>
    for ChainGetters<Cons<Getter, RestGetters>>
where
    Getter: FieldMapper<Context, Tag, Value = ValueA>,
    ChainGetters<RestGetters>: FieldGetter<ValueA, Tag, Value = ValueB>,
{
    type Value = ValueB;

    fn get_field(context: &Context, tag: PhantomData<Tag>) -> &ValueB {
        Getter::map_field(context, tag, |value| {
            <ChainGetters<RestGetters>>::get_field(value, tag)
        })
    }
}
```

The head `Getter` reads `ValueA` from the `Context`, and the rest of the chain reads `ValueB` from that
`ValueA`, so the whole chain's `Value` is `ValueB`, the value at the end of the path. The head is
applied through [`FieldMapper`](../traits/field_mapper.md) rather than `FieldGetter` directly:
`map_field` hands the intermediate reference to a closure that runs the rest of the chain on it, which
is what keeps the borrowed lifetimes inferring across each hop.

The recursion bottoms out at the empty list, where `ChainGetters<Nil>` is the identity getter and
returns the context it was given:

```rust
impl<Context, Tag> FieldGetter<Context, Tag> for ChainGetters<Nil> {
    type Value = Context;

    fn get_field(context: &Context, _tag: PhantomData<Tag>) -> &Context {
        context
    }
}
```

So a chain of one getter resolves to that getter applied to the context, a chain of two applies the
first then the second, and so on down the `Cons` spine. Because `ChainGetters` produces a `FieldGetter`
rather than the getter component's own provider trait, the [`WithProvider`](with_provider.md) adapter is
what turns it into a provider a getter component can be wired to.

## Related constructs

- [`WithProvider`](with_provider.md) — the adapter that turns `ChainGetters` into a getter-component
  provider.
- [`UseField`](use_field.md) and [`UseFieldRef`](use_field_ref.md) — the getters that form each step of
  the chain, implementing the same [`FieldGetter`](../traits/field_getter.md) it composes.
- [`FieldMapper`](../traits/field_mapper.md) — what each step is applied through, to keep borrowed
  lifetimes inferring across hops.
- [`Product!`](../macros/product.md) — builds the list of getters.
- [`#[cgp_getter]`](../macros/cgp_getter.md) — defines the getter this is wired to.
- [`#[derive(HasField)]`](../derives/derive_has_field.md) — supplies the field access each step reads.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — value injection through a
  provider's own requirements, here reaching across nested contexts.

## Source

- Struct and its two `FieldGetter` impls:
  [`chain.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/chain.rs)
- The traits it builds on:
  [`has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs)
  and [`map_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_field.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
