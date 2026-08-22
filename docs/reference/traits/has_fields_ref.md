---
sidebar_label: 'HasFieldsRef'
---

# `HasFieldsRef`

A type's whole shape, with every value borrowed.

:::info

### Generated machinery

**You are not expected to implement `HasFieldsRef`.**
[`#[derive(HasFields)]`](../derives/derive_has_fields.md) emits it alongside the owned shape. What
generic code writes is a bound; this page explains the type the derive produces, including the reserved
lifetime name that shows up in an error message.

:::

## Overview

[`HasFields`](./has_fields.md) describes a type as a list of named entries holding owned values. Code
that only *reads* a value should not have to consume it to walk its shape — a validator, a serializer, a
routine that inspects a struct and hands it back — so there is a second description in which every entry
holds a borrow instead:

```rust
pub trait HasFieldsRef {
    type FieldsRef<'a>
    where
        Self: 'a;
}
```

`FieldsRef<'a>` is the same shape as `Fields` with each value borrowed for `'a`. It is a **generic
associated type**, which is what lets one trait describe the shape at whatever lifetime a caller has.

The two shape traits are independent rather than one extending the other, and that is deliberate: a type
may be described owned, borrowed, or both. What connects the borrowed description to a value is
[`ToFieldsRef`](./to_fields_ref.md).

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough. There is no method — like its owned
counterpart, this trait only names a type.

The `where Self: 'a` clause on the associated type is required and worth recognizing: it says the borrow
cannot outlive the value it describes, which every use site inherits.

The impls come from [`#[derive(HasFields)]`](../derives/derive_has_fields.md), which emits this alongside
[`HasFields`](./has_fields.md) rather than as a separate opt-in.

## Examples

The borrowed shape of a struct pairs each name with a reference:

```rust
use cgp::prelude::*;

#[derive(HasField, HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

// Config::FieldsRef<'a> is
//   Product![Field<Symbol!("host"), &'a String>, Field<Symbol!("port"), &'a u16>]
```

Bounding on it is how generic read-only code names what it walks:

```rust
fn inspect<T>(value: &T)
where
    T: HasFieldsRef + ToFieldsRef,
{
    let _fields = value.to_fields_ref();
    // walk the borrowed shape; `value` is untouched
}
```

Requiring [`ToFields`](./to_fields.md) there instead would force every caller to give up its value or
clone it.

## When to reach for it, and when not

**Bound on it when generic code reads a type's shape without consuming it**, and pair it with
[`ToFieldsRef`](./to_fields_ref.md), which is the only way to obtain a value in this shape.

- **Use [`HasFields`](./has_fields.md)** when the code takes ownership — building, converting,
  destructuring.
- **Use `HasFieldsRef` + [`ToFieldsRef`](./to_fields_ref.md)** when the original must survive. This is
  the weaker requirement, so prefer it when it suffices.
- **Use [`HasField`](./has_field.md)** when only one named field is needed. The whole-shape traits are
  for code that walks everything.

Being precise about which of the five shape traits a routine requires pays, because each one narrows what
a caller must supply.

## Under the hood

:::note

### Advanced

This section shows the generated impl, and the reserved lifetime name in it.

:::

The derive emits the borrowed shape with a **reserved lifetime name**, `'__a`, so it cannot collide with
a lifetime of yours:

```rust
impl HasFieldsRef for Person {
    type FieldsRef<'__a> = Product![
        Field<Symbol!("name"), &'__a String>,
        Field<Symbol!("age"), &'__a u8>,
    ]
    where
        Self: '__a;
}
```

The transformation is uniform: each entry's value type `T` becomes `&'__a T`, and the spine is otherwise
untouched — same length, same order, same tags. An enum's borrowed shape is the dual, an `Either` chain
whose arms carry borrowed payloads.

Because the rewrite is per-entry rather than structural, **a field that is already a reference gains
another one**: a field of type `&'a Name` appears as `&'__a &'a Name`. That is correct, and it is the one
thing about this trait that surprises people in an error message.

## Gotchas

**A field that is already borrowed gets a second borrow.** `&'a Name` becomes `&'__a &'a Name` in
`FieldsRef`, which reads oddly in a diagnostic and is not a bug.

**The `where Self: 'a` clause propagates.** Any bound naming `FieldsRef<'a>` inherits it, so a signature
that borrows the shape usually needs the lifetime spelled out rather than elided.

**It is emitted by [`#[derive(HasFields)]`](../derives/derive_has_fields.md), not by a derive of its
own.** There is no `#[derive(HasFieldsRef)]`.

**Naming the shape is not obtaining it.** [`ToFieldsRef`](./to_fields_ref.md) is the method half; this
trait has none.

**Two enum variant names are reserved**, `Fields` and `FieldsRef`, because these associated types are
named through `Self::…` in the generated impls. The
[derive's page](../derives/derive_has_fields.md) lists them with the others.

## Related constructs

- [`HasFields`](./has_fields.md) — the owned shape, and where the family is explained in full.
- [`ToFieldsRef`](./to_fields_ref.md) — the conversion that produces a value in this shape.
- [`ToFields`](./to_fields.md) and [`FromFields`](./from_fields.md) — the owned conversions.
- [`#[derive(HasFields)]`](../derives/derive_has_fields.md) — generates this impl.
- [`Product!`](../macros/product.md) and [`Sum!`](../macros/sum.md) — the list types a shape is built
  from.
- [`Field`](../types/field.md) — one entry of a shape.
- [`MapTypeRef`](./map_type_ref.md) — the borrow markers the extractor family uses for the same job.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields.

## Source

- [`has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_fields.rs)
  — `HasFieldsRef` and `HasFields`
- Derive codegen: [`cgp_data/derive_has_fields/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
