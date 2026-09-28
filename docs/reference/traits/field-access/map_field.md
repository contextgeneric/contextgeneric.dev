---
title: 'MapField — read through a field'
description: 'A blanket helper on every HasField that passes a borrowed field to a closure, so generic code can read a field of a field without a lifetime bound.'
sidebar_label: 'MapField'
sidebar_position: 5
---

# `MapField`

Read through a field of a generic context without adding a lifetime bound on its type.

## Overview

Reading a field of a field looks like two chained reads, and on a concrete type it is. In generic
code it is not: when a [**context**](/docs/reference/glossary#context), the type the method runs on,
which supplies the values it needs as its fields, holds a field of a generic type `Inner`,
`context.get_field(..).get_field(..)` returns a borrow that the compiler cannot tie to the context's
borrow without knowing that `Inner` outlives it. A free function can add that bound, but a provider
trait's method has its lifetimes fixed by the trait, so there the only bound that fits is `'static`.

`MapField` avoids the problem by taking a closure instead of returning the intermediate borrow.
`map_field` reads the field and hands it to a closure that returns a borrow derived from it, and the
closure's higher-ranked bound lets the compiler tie the result to the context's own borrow. Every
[`HasField`](./has_field.md) has it through a blanket impl, and its provider-side twin
[`FieldMapper`](./field_mapper.md) is what [`ChainGetters`](../../providers/chain_getters.md) uses
to descend into a nested value.

## Definition

`MapField<Tag>` extends [`HasField<Tag>`](./has_field.md) with a `map_field` method that reads the
field and hands it to a closure:

```rust
pub trait MapField<Tag>: HasField<Tag> {
    fn map_field<T>(
        &self,
        _tag: PhantomData<Tag>,
        mapper: impl for<'a> FnOnce(&'a Self::Value) -> &'a T,
    ) -> &T;
}
```

It is a [supertrait](/docs/reference/glossary#supertrait) extension, so `Value` comes from
`HasField`. The `for<'a>` bound on `mapper` requires the closure to work for any lifetime of its
argument, so the `&T` it returns is a borrow of the field, and `map_field` can return it as a borrow
of `self`.

## Usage

It is not in the prelude. Import it from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::MapField;
```

Nothing implements it by hand: a blanket impl gives it to every `HasField` whose tag is `'static`,
which every [`Symbol!`](../../macros/symbol.md) and [`Index<N>`](../../types/index_type.md) is. So
bounding a context on `HasField` is enough to call `map_field` on it.

## Examples

A generic function that reads the `name` of a context's `inner` field, and the wiring that performs
the same descent for a getter:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::ChainGetters;
use cgp::core::field::traits::MapField;

pub fn inner_name<Context, Inner>(context: &Context) -> &String
where
    Context: HasField<Symbol!("inner"), Value = Inner>,
    Inner: HasField<Symbol!("name"), Value = String>,
{
    context.map_field(PhantomData::<Symbol!("inner")>, |inner| {
        inner.get_field(PhantomData::<Symbol!("name")>)
    })
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
        NameGetterComponent: WithProvider<
            ChainGetters<Product![
                UseField<Symbol!("inner")>,
                UseField<Symbol!("name")>,
            ]>,
        >,
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

    assert_eq!(inner_name(&outer), "Alice");
    assert_eq!(outer.name(), "Alice");
}
```

The closure receives the borrowed `Inner` and returns a borrow derived from it, so no bound on
`Inner`'s lifetime appears. The `ChainGetters` entry reaches the same field for the `HasName`
getter, chosen at the wiring site rather than written at a call site. `Outer` is a
[value context](/docs/reference/glossary#value-context).

## When to use it

**Reach for [`ChainGetters`](../../providers/chain_getters.md) to give a getter a nested field**,
and for an [`#[implicit]`](../../attributes/implicit.md) argument or [`HasField`](./has_field.md)
for a field of the context itself, which is the common case and needs none of this. Call `map_field`
directly when generic code of your own must read through a field, as `inner_name` does, and use
[`FieldMapper`](./field_mapper.md) when that code is a provider, where the context is a type
argument rather than `self`.

## Under the hood

The blanket impl reads the field once and passes the borrow to the mapper:

```rust
impl<Context, Tag> MapField<Tag> for Context
where
    Context: HasField<Tag>,
    Tag: 'static,
{
    fn map_field<T>(
        &self,
        tag: PhantomData<Tag>,
        mapper: impl for<'a> FnOnce(&'a Self::Value) -> &'a T,
    ) -> &T {
        mapper(self.get_field(tag))
    }
}
```

The `'static` bound is on the tag, not on the value, and that is the point: the tag is always
`'static`, while the value, the intermediate whose lifetime the chained read could not express,
stays free.

## Common Mistakes

**Two chained reads fail in generic code.** Written as
`context.get_field(PhantomData::<Symbol!("inner")>).get_field(PhantomData::<Symbol!("name")>)`,
the example's `inner_name` fails:

```text
error[E0311]: the parameter type `Inner` may not live long enough
```

rustc suggests naming the lifetime and adding `Inner: 'a`, which works in a free function. In a
provider trait's method, whose signature the trait fixes, that bound cannot be written, so use
`map_field` or [`FieldMapper`](./field_mapper.md) instead.

**The closure must return a borrow derived from its argument.** Returning a reference to something
else does not satisfy the `for<'a>` binder, and the error talks about lifetimes rather than about
the closure's body.

**It cannot map to an owned value.** The signature returns `&T`; a getter that must produce a value
returns [`MRef`](../../types/mref.md) instead.

**`MapField` is not [`MapFields`](../type-level/map_fields.md) or
[`MapType`](../type-level/map_type.md).** This one reads through a field; `MapFields` applies a
marker across a type-level list, and `MapType` names one field's storage on a partial record.

## Related constructs

- [`FieldMapper`](./field_mapper.md): the provider-side twin.
- [`HasField`](./has_field.md): the supertrait, and the plain read.
- [`ChainGetters`](../../providers/chain_getters.md): the provider that descends through nested
  fields.
- [`MRef`](../../types/mref.md): the return type for a getter that may produce rather than lend.

The ideas behind it:

- [Implicit arguments](/docs/concepts/implicit-arguments): the ergonomic surface over field access.

## Source

- [`traits/map_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_field.rs):
  `MapField` and `FieldMapper`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
