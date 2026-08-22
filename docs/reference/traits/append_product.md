---
sidebar_label: 'AppendProduct'
---

# `AppendProduct`

Adding one entry to the end of a type-level product list.

:::info

### Generated machinery

**You are not expected to use `AppendProduct` directly.** It is a type-level
operation the extensible-data machinery computes with — the builder and merge recursions are its callers
in practice. You will most likely meet it in an error message from code that walks a shape; this page
explains what it produces so that message is legible. The one case for naming it is generic code that must describe in a signature the shape it *will* produce.

:::

## Overview

A struct's shape in CGP is a type-level list — a [`Product!`](../macros/product.md) of named fields.
Code that processes such a shape generically sometimes needs to describe a *new* shape computed from an
old one: the shape a builder will have after one more field is set, say, or the shape a routine promises
to return.

`AppendProduct<Item>` is that computation for a single entry. It takes a product and an item and names
the product with that item added **at the end**:

```rust
pub trait AppendProduct<Item: ?Sized> {
    type Output;
}
```

There is no method, because there is nothing to execute. The result is an associated type, evaluated by
the trait solver while the compiler type-checks, so it carries no runtime cost and imposes no ordering.

**This is one of the least user-facing traits in the reference.** You meet it if you write generic code
over shapes, and otherwise only in an error message from code that does.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::AppendProduct;
```

`Self` is the existing product, `Item` is the entry to add, and `Output` is the result. `Item` may be
unsized. Every existing entry is preserved in order, and the new one lands last.

Appending onto `Nil` — the empty product — yields a one-element list, which is the recursion's base case
and occasionally useful to know when reading a bound.

## Examples

The results are types, so the check is a type equality rather than a value comparison:

```rust
use cgp::core::field::traits::AppendProduct;
use cgp::prelude::*;

type Base = Product![Field<Symbol!("host"), String>];

type WithPort = <Base as AppendProduct<Field<Symbol!("port"), u16>>>::Output;
// = Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>]
```

In generic code the same thing appears as a bound whose projection names the routine's result shape:

```rust
fn with_extra_field<Fields, Extra>() -> <Fields as AppendProduct<Extra>>::Output
where
    Fields: AppendProduct<Extra>,
{
    todo!()
}
```

## When to reach for it, and when not

**Reach for it when a routine must describe the shape it *will* produce** — in a signature, an associated
type, or a `where` clause — and essentially never otherwise. It is a building block the extensible-data
machinery uses; the machinery itself is what application code touches.

- **Reach for [`ConcatProduct`](./concat_product.md)** to splice a whole list rather than one entry.
  Append is its single-entry special case, so if you find yourself appending in a loop, you wanted
  concat.
- **Use [`HasFields`](./has_fields.md) instead** if you only need a type's *existing* shape. This trait
  computes new shapes; it is not how you obtain one.
- **Use the [builder](./has_builder.md) or [extractor](./extract_field.md) family instead** if you are
  moving *values* through a shape. This never touches a value, so if you want something to happen at run
  time, it is the wrong layer.

There is no `AppendSum`. Growing a sum is not an operation this layer provides.

## Under the hood

:::note

### Advanced

This section shows the recursion. It is four lines, and reading it makes
[`ConcatProduct`](./concat_product.md) obvious for free.

:::

The trait is a pair of impls, one for the spine's `Cons` node and one for its `Nil` terminator. Each head
is kept and the tail rebuilt, with a single-element list grafted on at the end:

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

Because the recursion only ever rebuilds the spine, order is structurally preserved — which is why
appending yields a *different* type from prepending, and why two products with the same entries in
different orders are unrelated types.

All of this is resolved during type checking. There is no `fn` anywhere on this page.

## Gotchas

**It is not in the prelude.** Import from `cgp::core::field::traits`. This is the first thing that goes
wrong when reaching for it.

**The result is `Output`, while [`MapFields`](./map_fields.md) exposes `Mapped`.** The asymmetry between
the three product operations is easy to forget, and reaching for the wrong name is a plain
unresolved-associated-type error.

**It is product-only.** There is no sum equivalent.

**Order is part of the type.** Appending is not commutative with prepending, and nothing reorders a list
to make two shapes match.

**A long list means a deep recursion.** This is trait resolution over the spine, so a very wide struct
costs compile time proportional to its width — one of the places CGP's compile-time cost actually comes
from.

## Related constructs

- [`ConcatProduct`](./concat_product.md) — the general form; append is its single-entry case.
- [`MapFields`](./map_fields.md) — rewriting every entry rather than adding one.
- [`Product!`](../macros/product.md) — the sugar for the lists this operates on.
- [Type-level spines](../types/type_level_spines.md) — the `Cons`/`Nil` chain underneath.
- [`HasFields`](./has_fields.md) — where a type's existing shape comes from.
- [`Field`](../types/field.md) — the entries a field list is usually made of.
- [`HasBuilder`](./has_builder.md) — the family that moves values through the shapes this computes.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — where computing a new shape is put to work.

## Source

- [`append_product.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/append_product.rs)
  — `AppendProduct`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
