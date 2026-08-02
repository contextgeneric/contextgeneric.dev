---
sidebar_label: 'AppendProduct, ConcatProduct & MapFields'
---

# `AppendProduct`, `ConcatProduct` & `MapFields`

The type-level operations over product lists.

## What it's for

A struct's shape in CGP is a type-level list — a [`Product!`](../macros/product.md) of named fields — and an
enum's is the analogous [`Sum!`](../macros/sum.md). Code that processes such a shape generically often needs to
*compute a new shape from an old one*: add a field, splice two lists together, rewrite every entry the same way.

These three traits are exactly those three computations, each a pure function from type lists to type lists:

- **`AppendProduct<Item>`** adds one entry at the end.
- **`ConcatProduct<Items>`** splices a whole list onto the end.
- **`MapFields<Mapper>`** rewrites every entry through a [`MapType`](./map_type.md) marker.

They are evaluated by the trait solver while the compiler type-checks, so they carry no runtime cost and impose
no ordering — there is nothing to run. They are the plumbing beneath the higher-level constructs: building a
record appends, merging two concatenates, and producing a partial-record representation maps a marker over every
field.

**These are the least user-facing traits in the reference.** You will meet them if you write generic code over
shapes, and otherwise only in an error message from code that does.

## Using it

**None of the three is in the prelude.** Import them from `cgp::core::field::traits`, and note that the
[`MapType`](./map_type.md) marker `MapFields` takes may need its own import too — `IsPresent`, `IsNothing`, and
`IsVoid` are in the prelude while `IsOptional` is not.

```rust
use cgp::core::field::impls::IsOptional;
use cgp::core::field::traits::{AppendProduct, ConcatProduct, MapFields};
```

Each trait exposes its result as an associated type and has no methods, because there is nothing to execute.

### `AppendProduct`

```rust
pub trait AppendProduct<Item: ?Sized> {
    type Output;
}
```

It recurses down the `Cons` spine, rebuilding each node until it reaches `Nil`, where it inserts
`Cons<Item, Nil>`. Reading it: every existing entry is preserved in order, and the new one lands last.

### `ConcatProduct`

```rust
pub trait ConcatProduct<Items> {
    type Output;
}
```

The same recursion, except that at `Nil` it substitutes the entire `Items` list rather than a single element. So
**append is the single-entry special case of concat**, which is the shortest way to hold both in mind.

Both have `Nil` identities worth knowing: concatenating onto `Nil` yields the other list unchanged, and
concatenating `Nil` onto a list leaves it unchanged.

### `MapFields`

```rust
pub trait MapFields<Mapper> {
    type Mapped;
}
```

`MapFields` is the transforming one, and the only one defined over **both** spines: it walks `Cons`/`Nil` for a
product and `Either`/`Void` for a sum, applying `Mapper::Map` to each entry. The list's length and order never
change; each entry type `T` becomes `Mapper::Map<T>`.

Which means the marker decides the whole effect. With `IsPresent` it is the identity; with `IsNothing` every
entry collapses to `()`; with `IsOptional` every entry becomes `Option<_>`; with `IsVoid` every entry becomes
uninhabited.

## Examples

Appending and concatenating compute new product types. The results are types, so the check is a type equality
rather than a value comparison:

```rust
use cgp::core::field::traits::{AppendProduct, ConcatProduct};
use cgp::prelude::*;

type Base = Product![Field<Symbol!("host"), String>];

// one entry added at the end
type WithPort = <Base as AppendProduct<Field<Symbol!("port"), u16>>>::Output;
// = Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>]

// a whole list spliced on
type Extra = Product![Field<Symbol!("tls"), bool>];
type Full = <WithPort as ConcatProduct<Extra>>::Output;
// = Product![host, port, tls]
```

`MapFields` rewrites every entry uniformly. Applying `IsOptional` turns a product of values into a product of
optionals — the shape a partial builder uses to track what is not yet filled:

```rust
use cgp::core::field::impls::IsOptional;
use cgp::core::field::traits::MapFields;

type Fields = Product![String, u16, bool];

type Optional = <Fields as MapFields<IsOptional>>::Mapped;
// = Product![Option<String>, Option<u16>, Option<bool>]
```

and the same marker applies over a sum, which is what lets one operation produce both a partial record and a
partial enum:

```rust
type Variants = Sum![String, u16];

type OptionalVariants = <Variants as MapFields<IsOptional>>::Mapped;
// = Sum![Option<String>, Option<u16>]
```

## When to reach for it, and when not

**Reach for these when you are writing generic code that computes a shape**, and essentially never otherwise.
They are building blocks the extensible-data machinery uses; the machinery itself is what application code
touches.

- **`AppendProduct` and `ConcatProduct`** when a routine must describe the shape it *will* produce — a builder
  that adds a field, a merge that combines two records — in a signature or an associated type.
- **`MapFields`** when a routine must describe a uniformly re-wrapped shape, which is what a partial
  representation is.
- **Use [`HasFields`](./has_fields.md) instead** if you only need a type's *existing* shape. These traits compute
  new shapes; they are not how you obtain one.
- **Use the [builder](./has_builder.md) or [extractor](./extract_field.md) family instead** if you are moving
  *values* through a shape. These three never touch a value — they only name types — so if you want something to
  happen at run time, they are the wrong layer.

One name worth disambiguating: `MapFields` applies one marker across a whole list, while
[`MapType`](./map_type.md) is a single marker naming one field's storage and `MapField` (singular, on the
[`HasField`](./has_field.md) page) is a lifetime helper for reading a nested field. Three similar names, three
unrelated jobs.

## Under the hood

:::note

### Advanced

This section shows the recursions. They are short, and reading one makes the others obvious — which is the
quickest way to be able to predict what any of these traits will produce.

:::

Each trait is a pair of impls: one for the spine's `Cons` node and one for its terminator. `AppendProduct` keeps
each head and rebuilds the tail, grafting a single-element list at the end:

```rust
impl<Item> AppendProduct<Item> for Nil {
    type Output = Cons<Item, Nil>;
}

impl<Head, Tail, Item> AppendProduct<Item> for Cons<Head, Tail>
where
    Tail: AppendProduct<Item>,
{
    type Output = Cons<Head, Tail::Output>;
}
```

`ConcatProduct` is character-for-character the same recursion with one line different — at `Nil` it substitutes
the whole list:

```rust
impl<Items> ConcatProduct<Items> for Nil {
    type Output = Items;          // the only difference
}

impl<Head, Tail, Items> ConcatProduct<Items> for Cons<Head, Tail>
where
    Tail: ConcatProduct<Items>,
{
    type Output = Cons<Head, Tail::Output>;
}
```

That is the precise sense in which append is a special case of concat.

`MapFields` transforms the head rather than preserving it, and this is the pair that exists twice — once for
each spine:

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

The `Either`/`Void` impls mirror these exactly, with `Either` for `Cons` and `Void` for `Nil`. Because the
recursion only ever replaces a head type, the spine's length and shape are structurally preserved — which is why
a mapped product is still a product of the same arity, and a mapped sum still a sum.

All of this is resolved during type checking. There is no `fn` anywhere on this page.

## Gotchas

**None of the three is in the prelude.** Import from `cgp::core::field::traits`. This is the first thing that
goes wrong when reaching for them.

**The result is an associated type, and the two names differ.** `AppendProduct` and `ConcatProduct` expose
`Output`; `MapFields` exposes `Mapped`. Reaching for the wrong one is a plain unresolved-associated-type error,
but the asymmetry is easy to forget.

**`MapFields` covers both spines; the other two are product-only.** There is no `AppendSum`. Growing a sum is
not an operation this layer provides.

**These compute types, not values.** A `MapFields<IsOptional>` result type does not wrap anything at run time by
itself — something still has to build the wrapped values, which on a partial record is
[`TransformMapFields`](./map_type.md).

**Order is preserved, and order is part of the type.** Appending to a product yields a *different* type from
prepending, and two products with the same entries in different orders are unrelated types.

**A long list means a deep recursion.** These are trait-resolution recursions over the spine, so a very wide
struct costs compile time proportional to its width — one of the places CGP's compile-time cost actually comes
from.

## Related constructs

- [`Product!`](../macros/product.md) and [`Sum!`](../macros/sum.md) — the sugar for the lists these operate on.
- [Type-level spines](../types/type_level_spines.md) — the `Cons`/`Nil` and `Either`/`Void` chains underneath.
- [`HasFields`](./has_fields.md) — where a type's existing shape comes from.
- [`MapType`](./map_type.md) — the markers `MapFields` applies, and the neighbouring trait most easily confused
  with it.
- [`Field`](../types/field.md) — the entries a field list is usually made of.
- [`HasBuilder`](./has_builder.md) and [`ExtractField`](./extract_field.md) — the families that move values
  through the shapes these compute.
- [`CanUpcast`](./cast.md) — conversions that walk a shape rather than reshape it.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — where computing a new shape is put to work.

## Source

- [`append_product.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/append_product.rs)
  — `AppendProduct`
- [`concat_product.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/concat_product.rs)
  — `ConcatProduct`
- [`map_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_fields.rs)
  — `MapFields`, over both spines

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
