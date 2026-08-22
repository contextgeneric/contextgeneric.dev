---
sidebar_label: 'ConcatProduct'
---

# `ConcatProduct`

Splicing one type-level product list onto the end of another.

:::info

### Generated machinery

**You are not expected to use `ConcatProduct` directly.** It is a type-level
operation the extensible-data machinery computes with, and you will most likely meet it in an error
message from code that walks a shape rather than in code you wrote. This page explains what it
produces. The one case for naming it is generic code that must describe the shape produced by combining two others.

:::

## Overview

Merging two records means merging their shapes, and the shapes are type-level lists — a
[`Product!`](../macros/product.md) of named fields each. `ConcatProduct<Items>` names the list you get by
following one product with another:

```rust
pub trait ConcatProduct<Items> {
    type Output;
}
```

There is no method, because there is nothing to execute. The result is an associated type evaluated by
the trait solver during type checking, so it costs nothing at run time.

It is the general form of [`AppendProduct`](./append_product.md): **append is the single-entry special
case of concat**, and that is the shortest way to hold both in mind.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::ConcatProduct;
```

`Self` is the first product, `Items` is the second, and `Output` is the first's entries followed by the
second's. Order is preserved on both sides.

Two identities are worth knowing, because they let a generic bound resolve in the degenerate cases:
concatenating onto `Nil` yields the other list unchanged, and concatenating `Nil` onto a list leaves it
unchanged.

## Examples

The results are types, so the check is a type equality:

```rust
use cgp::core::field::traits::{AppendProduct, ConcatProduct};
use cgp::prelude::*;

type Base = Product![Field<Symbol!("host"), String>];

type WithPort = <Base as AppendProduct<Field<Symbol!("port"), u16>>>::Output;

type Extra = Product![Field<Symbol!("tls"), bool>];

type Full = <WithPort as ConcatProduct<Extra>>::Output;
// = Product![host, port, tls]
```

Which is the type-level counterpart of what [`CanBuildFrom`](./can_build_from.md) does with values: this
names the combined shape, that moves the fields into it.

## When to reach for it, and when not

**Reach for it when generic code must name the shape produced by combining two others** — a merge, a
routine that extends a record with a caller-supplied set of fields — and essentially never otherwise.

- **Reach for [`AppendProduct`](./append_product.md)** for a single entry. It is the same recursion with
  a one-element graft, and it reads better for the one-field case.
- **Use [`HasFields`](./has_fields.md) instead** if you only need a type's *existing* shape.
- **Use [`CanBuildFrom`](./can_build_from.md) instead** if you want the *values* merged rather than the
  shapes named. This trait never touches a value.
- **Note that [`ConcatPath`](./concat_path.md) is the path-level analogue**, doing the same job for the
  segment lists behind [`Path!`](../macros/path.md), with the same two-impl recursion.

There is no `ConcatSum`. Combining two sums is not an operation this layer provides.

## Under the hood

:::note

### Advanced

This section shows the recursion, which is [`AppendProduct`](./append_product.md#under-the-hood)'s with
one line changed.

:::

Two impls, one per spine node. Each head is kept and the tail rebuilt; at the terminator, the whole
second list is substituted:

```rust
impl<Items> ConcatProduct<Items> for Nil {
    type Output = Items;          // the only difference from AppendProduct
}

impl<Head, Tail, Items> ConcatProduct<Items> for Cons<Head, Tail>
where
    Tail: ConcatProduct<Items>,
{
    type Output = Cons<Head, Tail::Output>;
}
```

Set `Items = Cons<Item, Nil>` and this is `AppendProduct` exactly, which is the precise sense in which
append is a special case.

The recursion walks the *first* list only, so its cost is proportional to the left operand's length
rather than to the result's.

## Gotchas

**It is not in the prelude.** Import from `cgp::core::field::traits`.

**The result is `Output`, while [`MapFields`](./map_fields.md) exposes `Mapped`.** The asymmetry between
the three product operations is easy to forget.

**It is product-only.** For paths, the analogue is [`ConcatPath`](./concat_path.md); for sums, there is
nothing.

**Duplicate entries are not detected.** Concatenating two products that share a field name yields a list
with the name twice, which is a legal type and will fail later, at whatever consumes it, rather than
here.

**Order is part of the type.** `A` then `B` is a different type from `B` then `A`, and nothing normalizes
them.

**A long list means a deep recursion**, so concatenating two wide shapes costs compile time proportional
to the first one's width.

## Related constructs

- [`AppendProduct`](./append_product.md) — the single-entry case.
- [`MapFields`](./map_fields.md) — rewriting every entry rather than adding any.
- [`ConcatPath`](./concat_path.md) — the same operation over type-level paths.
- [`Product!`](../macros/product.md) — the sugar for the lists this operates on.
- [Type-level spines](../types/type_level_spines.md) — the `Cons`/`Nil` chain underneath.
- [`CanBuildFrom`](./can_build_from.md) — merging the values whose shapes this combines.
- [`HasFields`](./has_fields.md) — where a type's existing shape comes from.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — where computing a new shape is put to work.

## Source

- [`concat_product.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/concat_product.rs)
  — `ConcatProduct`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
