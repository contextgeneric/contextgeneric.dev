---
title: 'MapFields — rewrite every entry of a list'
sidebar_label: 'MapFields'
sidebar_position: 7
description: 'Rewrite every entry of a type-level product or sum through one MapType marker, as CGP does to wrap each provider in a pipeline''s list.'
---

# `MapFields`

Rewriting every entry of a type-level list through one marker.

:::info

### Generated machinery

**You are not expected to use `MapFields` directly.** CGP uses it to rewrite lists of providers:
[`PipeMonadic`](../../providers/monad/pipe_monadic.md) maps its handler list through a promotion
marker, and the record builder's `BuildAndMergeOutputs` maps its handler list the same way. You will
most likely meet it in an error message from one of those; this page explains what it produces, and
how it differs from the two similarly-named traits beside it. The one case for naming it is generic
code that must describe a uniformly re-wrapped [shape](/docs/reference/glossary#shape).

:::

## Overview

Some type-level lists need every entry changed the same way: each value wrapped in an `Option`, or
replaced by `()`, or each provider wrapped in an adapter. Describing that as a type means rewriting
every entry of the list uniformly, and `MapFields<Mapper>` is that operation. The partial records and
enums of the extensible-data derives are not computed with it; [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md)
generates those types directly.

The list's length and order never change. Each entry type `T` becomes `Mapper::Map<T>`, where the mapper
is a [`MapType`](./map_type.md) marker, so **the marker decides the whole effect**. With `IsPresent` it
is the identity; with `IsNothing` every entry collapses to `()`; with `IsOptional` every entry becomes
`Option<_>`; with `IsVoid` every entry becomes uninhabited.

It is the transforming member of the three product operations, and the only one defined over **both**
lists: it walks `Cons`/`Nil` for a product and `Either`/`Void` for a sum, so one marker applies to
either.

## Definition

`MapFields` is parameterized by a marker and exposes the rewritten list as an associated type:

```rust
pub trait MapFields<Mapper> {
    type Mapped;
}
```

`Self` is the list, `Mapper` is the [`MapType`](./map_type.md) marker to apply, and the result is
exposed as `Mapped`. Note the name, which differs from the `Output` its two siblings
[`AppendProduct`](./append_product.md) and [`ConcatProduct`](./concat_product.md) expose. The trait
lacks a method, because nothing needs executing.

## Usage

**It is not in the prelude**, and neither is `IsOptional`. Import the trait from
`cgp::core::field::traits` and any non-prelude marker from `cgp::core::field::impls`:

```rust
use cgp::core::field::impls::IsOptional;
use cgp::core::field::traits::MapFields;
```

`IsPresent`, `IsNothing`, and `IsVoid` are in the prelude; `IsOptional` is not.

## Examples

One marker applied over a product and over a sum, checked as type equalities:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::IsOptional;
use cgp::core::field::traits::MapFields;

pub type Fields = Product![String, u16, bool];

pub type Optional = <Fields as MapFields<IsOptional>>::Mapped;

pub fn assert_optional(
    fields: Optional,
) -> Product![Option<String>, Option<u16>, Option<bool>] {
    fields
}

pub type Variants = Sum![String, u16];

pub type OptionalVariants = <Variants as MapFields<IsOptional>>::Mapped;

pub fn assert_optional_variants(
    variants: OptionalVariants,
) -> Sum![Option<String>, Option<u16>] {
    variants
}

/// The page's claim that `IsPresent` is the identity, and that `IsNothing` collapses each entry.
pub fn assert_present(fields: <Fields as MapFields<IsPresent>>::Mapped) -> Fields {
    fields
}

pub fn assert_nothing(
    fields: <Fields as MapFields<IsNothing>>::Mapped,
) -> Product![(), (), ()] {
    fields
}
```

`IsOptional` wraps each entry, `IsPresent` leaves the list unchanged, and `IsNothing` collapses each
entry to `()`. The sum is rewritten by the same marker, since `MapFields` covers both lists.

## When to use it

**Reach for it when generic code must name a uniformly re-wrapped list**, such as a list of
providers each wrapped in the same adapter, and essentially never otherwise.

- **Reach for [`AppendProduct`](./append_product.md) or [`ConcatProduct`](./concat_product.md)** when the
  shape grows rather than changing its wrapping.
- **Use [`TransformMapFields`](./transform_map_fields.md) instead** when you want the *values* re-wrapped
  rather than the type named. This trait computes a type; nothing wraps anything at run time by itself.
- **Use [`HasFields`](../shape/has_fields.md) instead** if you only need a type's existing shape.

One name worth disambiguating, because three similar ones sit close together. `MapFields` applies one
marker across a whole list; [`MapType`](./map_type.md) is a single marker naming *one* field's storage;
and [`MapField`](../field-access/map_field.md), singular, with no *s*, is a lifetime helper for reading a nested field.
Three similar names, three unrelated jobs.

## Under the hood

Like its two siblings, `MapFields` is a pair of impls per list, one for the node and one for the
terminator. Unlike them, it *transforms* the head rather than preserving it:

```rust
impl<Mapper, Current, Rest> MapFields<Mapper> for Cons<Current, Rest>
where
    Mapper: MapType,
    Rest: MapFields<Mapper>,
{
    type Mapped = Cons<Mapper::Map<Current>, Rest::Mapped>;
}

impl<Mapper> MapFields<Mapper> for Nil {
    type Mapped = Nil;
}
```

The `Either`/`Void` impls mirror these exactly, with `Either` standing where `Cons` does and `Void` where
`Nil` does. Because the recursion only ever replaces a head *type*, the list's length and shape are
structurally preserved, which is why a mapped product is still a product of the same arity, and a mapped
sum still a sum.

All of it resolves during type checking, so a mapped shape is a name for a type rather than a
computation that runs.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::traits`, and remember that `IsOptional`
needs its own import from `cgp::core::field::impls`.

**The result is `Mapped`, not `Output`.** [`AppendProduct`](./append_product.md) and
[`ConcatProduct`](./concat_product.md) both expose `Output`, and this one does not.
`<Product![u8, u16] as MapFields<IsPresent>>::Output` fails with:

```text
error[E0576]: cannot find associated type `Output` in trait `MapFields`
```

**It computes types, not values.** A `MapFields<IsOptional>` result does not wrap anything at run time.
Something still has to build the wrapped values, such as
[`TransformMapFields`](./transform_map_fields.md).

**A marker without a `MapType` impl does not resolve.** Mapping through a plain
`pub struct Wrapped;`, as `<Product![u8, u16] as MapFields<Wrapped>>::Mapped`, names the missing
bound and the list that needed it:

```text
error[E0277]: the trait bound `Wrapped: MapType` is not satisfied
...
   = note: required for `Cons<u8, Cons<u16, Nil>>` to implement `MapFields<Wrapped>`
```

**It is the only one of the three that covers sums.** Neither `AppendSum` nor `ConcatSum` exists.

**A long list means a deep recursion**, so mapping a wide shape costs compile time proportional to its
width.

## Related constructs

- [`MapType`](./map_type.md): the markers this applies, and the trait most easily confused with it.
- [`AppendProduct`](./append_product.md) and [`ConcatProduct`](./concat_product.md): the two operations
  that grow a list rather than rewrite it.
- [`TransformMapFields`](./transform_map_fields.md): the value-level counterpart, which actually converts
  the fields.
- [`Product!`](../../macros/product.md) and [`Sum!`](../../macros/sum.md): the sugar for both lists.
- [Type-level lists](../../types/index.md): the `Cons`/`Nil` and `Either`/`Void` chains
  underneath.
- [`HasFields`](../shape/has_fields.md): where a type's existing shape comes from.
- [`MapField`](../field-access/map_field.md): the similarly-named lifetime helper, which does something else entirely.

The ideas behind it:

- [Monadic handlers](/docs/concepts/monadic-handlers): the handler pipelines whose provider lists it
  rewrites.
- [Extensible records](/docs/concepts/extensible-records): the builder whose handler list it rewrites.

## Source

- [`map_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_fields.rs):
  `MapFields`, over both lists

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
