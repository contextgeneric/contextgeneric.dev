---
sidebar_label: 'MapFields'
---

# `MapFields`

Rewriting every entry of a type-level list through one marker.

:::info

### Generated machinery

**You are not expected to use `MapFields` directly.** It is the type-level operation
behind a partial record's shape, computed by the machinery
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md) generates. You will most likely meet it in an error
message; this page explains what it produces, and how it differs from the two similarly-named traits
beside it. The one case for naming it is generic code that must describe a uniformly re-wrapped shape.

:::

## Overview

A partial record is the same record with every field's storage changed the same way — each value wrapped
in an `Option`, or replaced by `()`, or left alone. Describing that as a type means rewriting every entry
of the shape uniformly, and `MapFields<Mapper>` is that operation:

```rust
pub trait MapFields<Mapper> {
    type Mapped;
}
```

The list's length and order never change. Each entry type `T` becomes `Mapper::Map<T>`, where the mapper
is a [`MapType`](./map_type.md) marker — so **the marker decides the whole effect**. With `IsPresent` it
is the identity; with `IsNothing` every entry collapses to `()`; with `IsOptional` every entry becomes
`Option<_>`; with `IsVoid` every entry becomes uninhabited.

It is the transforming member of the three product operations, and the only one defined over **both**
spines: it walks `Cons`/`Nil` for a product and `Either`/`Void` for a sum. That is what lets one
operation produce both a partial record and a partial enum.

## Usage

**It is not in the prelude**, and neither is `IsOptional`. Import the trait from
`cgp::core::field::traits` and any non-prelude marker from `cgp::core::field::impls`:

```rust
use cgp::core::field::impls::IsOptional;
use cgp::core::field::traits::MapFields;
```

`IsPresent`, `IsNothing`, and `IsVoid` are in the prelude; `IsOptional` is not.

`Self` is the list, `Mapper` is the marker, and the result is `Mapped` — note the name, which differs
from the `Output` its two siblings expose. There is no method, because there is nothing to execute.

## Examples

Applying `IsOptional` turns a product of values into a product of optionals — the shape a partial builder
uses to track what is not yet filled:

```rust
use cgp::core::field::impls::IsOptional;
use cgp::core::field::traits::MapFields;
use cgp::prelude::*;

type Fields = Product![String, u16, bool];

type Optional = <Fields as MapFields<IsOptional>>::Mapped;
// = Product![Option<String>, Option<u16>, Option<bool>]
```

The same marker applies over a sum, which is what makes one operation serve both halves of the
extensible-data machinery:

```rust
type Variants = Sum![String, u16];

type OptionalVariants = <Variants as MapFields<IsOptional>>::Mapped;
// = Sum![Option<String>, Option<u16>]
```

## When to reach for it, and when not

**Reach for it when generic code must name a uniformly re-wrapped shape**, which is what a partial
representation is — and essentially never otherwise.

- **Reach for [`AppendProduct`](./append_product.md) or [`ConcatProduct`](./concat_product.md)** when the
  shape grows rather than changing its wrapping.
- **Use [`TransformMapFields`](./transform_map_fields.md) instead** when you want the *values* re-wrapped
  rather than the type named. This trait computes a type; nothing wraps anything at run time by itself.
- **Use [`HasFields`](./has_fields.md) instead** if you only need a type's existing shape.

One name worth disambiguating, because three similar ones sit close together. `MapFields` applies one
marker across a whole list; [`MapType`](./map_type.md) is a single marker naming *one* field's storage;
and [`MapField`](./map_field.md) — singular, no *s* — is a lifetime helper for reading a nested field.
Three similar names, three unrelated jobs.

## Under the hood

Like its two siblings, `MapFields` is a pair of impls per spine, one for the node and one for the
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
`Nil` does. Because the recursion only ever replaces a head *type*, the spine's length and shape are
structurally preserved — which is why a mapped product is still a product of the same arity, and a mapped
sum still a sum.

All of it resolves during type checking, so a mapped shape is a name for a type rather than a
computation that runs.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::traits`, and remember that `IsOptional`
needs its own import from `cgp::core::field::impls`.

**The result is `Mapped`, not `Output`.** [`AppendProduct`](./append_product.md) and
[`ConcatProduct`](./concat_product.md) both expose `Output`, and this one does not.

**It computes types, not values.** A `MapFields<IsOptional>` result does not wrap anything at run time —
something still has to build the wrapped values, which on a partial record is
[`TransformMapFields`](./transform_map_fields.md).

**A marker with no `MapType` impl does not resolve**, and the error names the missing `MapType` bound
rather than the marker's role.

**It is the only one of the three that covers sums.** There is no `AppendSum` or `ConcatSum`.

**A long list means a deep recursion**, so mapping a wide shape costs compile time proportional to its
width.

## Related constructs

- [`MapType`](./map_type.md) — the markers this applies, and the trait most easily confused with it.
- [`AppendProduct`](./append_product.md) and [`ConcatProduct`](./concat_product.md) — the two operations
  that grow a list rather than rewrite it.
- [`TransformMapFields`](./transform_map_fields.md) — the value-level counterpart, which actually converts
  the fields.
- [`Product!`](../macros/product.md) and [`Sum!`](../macros/sum.md) — the sugar for both spines.
- [Type-level spines](../types/type_level_spines.md) — the `Cons`/`Nil` and `Either`/`Void` chains
  underneath.
- [`HasFields`](./has_fields.md) — where a type's existing shape comes from.
- [`MapField`](./map_field.md) — the similarly-named lifetime helper, which does something else entirely.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — presence tracking on a partial record.
- [Extensible variants](/docs/concepts/extensible-variants) — possibility tracking on a partial variant.

## Source

- [`map_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_fields.rs)
  — `MapFields`, over both spines

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
