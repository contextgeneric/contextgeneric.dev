---
title: 'FieldMapper — the provider side of MapField'
sidebar_label: 'FieldMapper'
sidebar_position: 6
description: 'The provider-side mirror of MapField, a blanket impl over every field getter that lets a getter provider read through the field another provider returns.'
---

# `FieldMapper`

The provider-side mirror of `MapField`.

:::info

### Generated machinery

**You are not expected to implement `FieldMapper`.** It is a blanket impl over every
[`FieldGetter`](./field_getter.md), and [`ChainGetters`](../../providers/chain_getters.md) is the
provider that calls it to descend into a nested value. The one case for naming it is a getter
provider of your own that has to read through another provider's field, as the example on this page
does.

:::

## Overview

[`MapField`](./map_field.md) reads through a field of a generic
[**context**](/docs/reference/glossary#context), the type the method runs on, which supplies the
values it needs as its fields, by handing the borrowed field to a closure rather than returning it.
`FieldMapper` is the same operation in provider shape, with a provider as `Self` and the context as
an explicit type argument. This is the form a getter provider needs, because a provider trait's
method has its lifetimes fixed by the trait, so a chained read inside it cannot take the lifetime
bound a free function could. The group's two-by-two shape holds here too:

| | you bound against | you wire |
|---|---|---|
| plain read | [`HasField`](./has_field.md) | [`FieldGetter`](./field_getter.md) |
| read through | [`MapField`](./map_field.md) | `FieldMapper` |

## Definition

`FieldMapper<Context, Tag>` mirrors [`MapField`](./map_field.md), with the context as an explicit
type argument:

```rust
pub trait FieldMapper<Context, Tag>: FieldGetter<Context, Tag> {
    fn map_field<T>(
        context: &Context,
        _tag: PhantomData<Tag>,
        mapper: impl for<'a> FnOnce(&'a Self::Value) -> &'a T,
    ) -> &T;
}
```

`Self` is the provider and `Context` is the type being read from. It is a
[supertrait](/docs/reference/glossary#supertrait) extension of [`FieldGetter`](./field_getter.md),
so `Value` comes from there, and the `for<'a>` bound on `mapper` ties the returned `&T` to the
context's borrow.

## Usage

It is not in the prelude. Import it from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::FieldMapper;
```

Nothing implements it by hand: a blanket impl gives it to every `FieldGetter` whose provider and tag
are `'static`, so every field-getter provider can be read through. A provider of your own bounds an
inner provider on `FieldMapper` and calls `map_field` on it.

## Examples

A two-step getter provider that reads through the field one provider returns with another, wired for
a getter, and a direct call through `UseContext`:

```rust
use cgp::prelude::*;
use cgp::core::field::traits::FieldMapper;

pub struct ThenGet<First, Second>(pub PhantomData<(First, Second)>);

impl<Context, Tag, First, Second, Mid, Value> FieldGetter<Context, Tag> for ThenGet<First, Second>
where
    First: FieldMapper<Context, Tag, Value = Mid>,
    Second: FieldGetter<Mid, Tag, Value = Value>,
{
    type Value = Value;

    fn get_field(context: &Context, tag: PhantomData<Tag>) -> &Value {
        First::map_field(context, tag, |mid| Second::get_field(mid, tag))
    }
}

#[cgp_getter(NameGetter)]
pub trait HasName {
    fn name(&self) -> &String;
}

#[derive(HasField)]
pub struct Inner {
    pub name: String,
}

#[derive(HasField)]
pub struct Outer {
    pub inner: Inner,
}

delegate_components! {
    Outer {
        NameGetterComponent:
            WithProvider<ThenGet<UseField<Symbol!("inner")>, UseField<Symbol!("name")>>>,
    }
}

check_components! {
    Outer {
        NameGetterComponent,
    }
}

pub fn demo() {
    let outer = Outer {
        inner: Inner {
            name: "Alice".to_owned(),
        },
    };

    assert_eq!(outer.name(), "Alice");

    let name = <UseContext as FieldMapper<Outer, Symbol!("inner")>>::map_field(
        &outer,
        PhantomData,
        |inner| inner.get_field(PhantomData::<Symbol!("name")>),
    );
    assert_eq!(name, "Alice");
}
```

`ThenGet` is a two-element [`ChainGetters`](../../providers/chain_getters.md), written out to show
the mechanism; in real wiring use `ChainGetters` itself. Its `get_field` could not be written as
`Second::get_field(First::get_field(context, tag), tag)`: that fails with `E0311`, because `Mid` is
not known to outlive the borrow. `Outer` is a [value
context](/docs/reference/glossary#value-context).

## When to use it

**Reach for [`ChainGetters`](../../providers/chain_getters.md)** to give a getter a nested field by
wiring. Name `FieldMapper` only when writing a getter provider of your own that must read through
another provider's field, and use [`MapField`](./map_field.md) when the descent is on a generic
context in an ordinary function.

## Under the hood

`FieldMapper` is a blanket impl over every [`FieldGetter`](./field_getter.md):

```rust
impl<Getter, Context, Tag> FieldMapper<Context, Tag> for Getter
where
    Getter: FieldGetter<Context, Tag> + 'static,
    Tag: 'static,
{
    fn map_field<T>(
        context: &Context,
        tag: PhantomData<Tag>,
        mapper: impl for<'a> FnOnce(&'a Self::Value) -> &'a T,
    ) -> &T {
        mapper(Getter::get_field(context, tag))
    }
}
```

Both `'static` bounds are on types that are `'static` in practice, the tag and the provider, a
zero-sized marker; the field's value stays free. `ChainGetters` nests these calls, one per step, so
a chain of any length stays free of lifetime bounds.

## Common Mistakes

**A chained `get_field` inside a provider fails.** As the example notes, a provider that calls one
getter's `get_field` on another's result fails with `E0311` on the intermediate type; call
`map_field` on the first getter instead.

**`Self` is the provider, not the context.** The context is the first type parameter.

**A provider with a non-`'static` parameter loses the blanket impl.** The bound on `Self` holds for
an ordinary marker type and not for one carrying a lifetime, a rare shape and a puzzling failure
when it happens.

## Related constructs

- [`MapField`](./map_field.md): the consumer-side twin, where the lifetime problem is explained in
  full.
- [`FieldGetter`](./field_getter.md): the supertrait, and the plain wired read.
- [`ChainGetters`](../../providers/chain_getters.md): the provider built on this trait.
- [`UseField`](../../providers/use_field.md) and [`UseContext`](../../providers/use_context.md): the
  providers a chain is usually built from.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): the split this trait
  is an instance of.

## Source

- [`traits/map_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_field.rs):
  `FieldMapper` and `MapField`
- [`impls/chain.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/chain.rs):
  `ChainGetters`, which calls it

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
