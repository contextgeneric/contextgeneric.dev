---
title: 'Field — a value tagged with its name'
sidebar_label: 'Field'
sidebar_position: 2
description: 'A value paired with its type-level name tag, so a struct''s fields and an enum''s variants can be described one named entry at a time.'
---

# `Field`

A value paired with its type-level name tag, so a struct's fields and an enum's variants can be
described one entry at a time without naming the concrete type.

## Overview

`Field<Tag, Value>` carries a field's *name* and its *value* together in one type. A bare
[`Product!`](../macros/product.md) list such as `Product![String, u8]` records the value types and their
order, but it cannot tell `name: String` from any other `String`. Wrapping each element in a `Field`,
as `Field<Symbol!("name"), String>`, attaches the name as a type, so a struct's shape describes itself.
Code that walks the list matches on the tag to find the field it wants.

The name is a type because the program needs it only at compile time, for trait resolution and
dispatch, and never at run time. A `Field<Tag, Value>` is exactly as large as its `Value`, because
the tag lives in a zero-sized [`PhantomData<Tag>`](phantom_data.md). So naming a field this way
costs nothing at run time and makes field-by-field generic code possible.
[`Symbol!`](../macros/symbol.md) provides the same type-as-name encoding for a named field, and
[`Index`](index_type.md) provides it for a tuple position. `Field` joins that tag to the value it
labels.

`Field` fills both structural lists. In a record it is the element of a [`Product!`](../macros/product.md)
of entries, one per field. In a variant it is the element of a [`Sum!`](../macros/sum.md) of entries, one
per variant, where the `Value` is the variant's payload. The same `Field<Tag, Value>` shape names a
field in a struct and a variant in an enum.

## Definition

`Field` holds the value alongside a phantom tag:

```rust
pub struct Field<Tag, Value> {
    pub value: Value,
    pub phantom: PhantomData<Tag>,
}
```

`Tag` is the type-level name of the field. It appears only inside `PhantomData<Tag>`, never in a
stored field. It is usually a [type-level string](/docs/reference/glossary#type-level-string) such
as `Symbol!("name")` for a named field, or a type-level number such as `Index<0>` for a tuple
position. `Value` is the field's actual type, and `value` is the only data the struct keeps. Apart
from the tag, a `Field` is a thin wrapper around its `Value`. It is in the prelude, so
`use cgp::prelude::*;` is enough.

## Behavior

You build a `Field` from a value without a tag argument, because the tag comes from the target type
rather than from you. The `From<Value>` impl fills in `value` and sets `phantom` to `PhantomData`. So
`let f: Field<Symbol!("name"), String> = "Alice".to_string().into();` compiles, and the compiler infers
the tag from the expected type. This is why generated code builds each entry with a plain `.into()`.

The other trait impls defer to the value and ignore the tag, so a `Field` behaves like its `Value`
for comparison and printing. `Debug` forwards to the value's `Debug` and does not show the tag.
`PartialEq` and `Eq` compare only `value`, each gated on the matching bound on `Value`. Two `Field`
values are equal when their values are equal. The tag is a compile-time matter and does not take
part at run time. `Field` implements nothing else: it is not `Clone`, `Copy`, or `Default`, whatever
its `Value` is.

Because the tag lives only in `PhantomData`, code that needs the name reads it from the `Tag` parameter
through trait resolution rather than from stored data. For example, it matches a
`Field<Symbol!("name"), _>` against a [`HasField<Symbol!("name")>`](../traits/field-access/has_field.md)
bound.

**A tuple struct with exactly one field is the exception to the wrapping.** Its shape is the field's
type itself, without a `Field` around it or a list, so `struct Meters(u32)` has `Fields = u32`. A
tuple struct with two or more fields tags each entry by position, as `Field<Index<0>, u32>`. Inside
an enum, a variant with a single unnamed field has that field's type as its payload in the variant's
`Field`.

## Examples

A struct's shape, as [`#[derive(HasFields)]`](../derives/derive_has_fields.md) assigns it, is a list
of `Field` entries, and [`to_fields`](../traits/shape/to_fields.md) turns a value into that list:

```rust
use cgp::prelude::*;

#[derive(HasFields)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

#[derive(HasFields)]
pub struct Point(pub u32, pub u32);

#[derive(HasFields)]
pub struct Meters(pub u32);

pub fn demo() {
    // A struct's shape is a list of `Field` entries, each tagged by its name.
    let fields: Product![Field<Symbol!("name"), String>, Field<Symbol!("age"), u8>] = Person {
        name: "Alice".to_owned(),
        age: 30,
    }
    .to_fields();

    let Cons(name, Cons(age, Nil)) = fields;
    assert_eq!(name.value, "Alice");
    assert_eq!(age.value, 30);

    // A tuple struct's entries are tagged by position.
    let Cons(x, Cons(y, Nil)): Product![Field<Index<0>, u32>, Field<Index<1>, u32>] =
        Point(3, 4).to_fields();
    assert_eq!((x.value, y.value), (3, 4));

    // A one-field tuple struct's shape is the field's type itself.
    let inner: u32 = Meters(7).to_fields();
    assert_eq!(inner, 7);

    // One entry, built from its value; the annotation supplies the tag.
    let entry: Field<Symbol!("name"), String> = "Bob".to_owned().into();
    assert_eq!(entry.value, "Bob");

    // The tag adds no size, and `Debug` prints the value alone.
    assert_eq!(
        core::mem::size_of::<Field<Symbol!("name"), String>>(),
        core::mem::size_of::<String>()
    );
    assert_eq!(format!("{entry:?}"), "\"Bob\"");
}
```

The annotations on `fields` and on the tuple pattern are the check: each names the list type the
derive assigned, and the program compiles only if it matches. The derive writes that list as
`type Fields = Product![Field<Symbol!("name"), String>, Field<Symbol!("age"), u8>];` for `Person`,
and as `type Fields = u32;` for `Meters`.

## When to use it

**You read `Field` far more often than you write it.** The common case is to recognize it in a generated
shape or in an error, and this page is mostly for that.

- **Let the derives build it.** [`#[derive(HasFields)]`](../derives/derive_has_fields.md) and
  [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) produce the `Field` entries of a type's shape, so
  a type you own gets its entries from one derive rather than from a list you write yourself.
- **Construct a `Field` directly** only in generic shape code of your own, where you assemble or rewrite
  a record or a variant entry by entry. Build one with `.into()`, and let the expected type supply the
  tag.
- **Do not use a bare [`Product!`](../macros/product.md) of values** when you need the names. A list of
  `Field` entries carries the names, and the operations that walk a shape match on them.

## Common Mistakes

**The tag is compile-time only, so it never affects equality or printing.** Two `Field` values compare
equal when their values are equal, and `Debug` shows the value alone. The tag does not distinguish them at
run time. It distinguishes them only in the type.

**A `Field` is the size of its `Value`, not larger.** The tag occupies zero bytes, so wrapping a value in
a `Field` is free at run time.

**A one-field tuple struct's shape lacks a `Field`.** Code that expects every shape to be a list of
`Field` entries meets a bare `u32` for `struct Meters(u32)`, since the derive treats such a struct
as a newtype around its field.

**The tag must match exactly for a lookup to resolve.** `Field<Symbol!("first_name"), _>` and
`Field<Symbol!("firstName"), _>` carry unrelated tags, so a name mismatch appears as an unsatisfied
[`HasField`](../traits/field-access/has_field.md) bound rather than as a typo.

## Related constructs

- [`Symbol!`](../macros/symbol.md): the tag for a named field or variant.
- [`Index`](index_type.md): the tag for a tuple-struct position.
- [`PhantomData`](phantom_data.md): where the tag is stored, at zero size.
- [`Product!`](../macros/product.md) and [`Sum!`](../macros/sum.md): the lists of `Field` entries that
  describe a record and a variant.
- [`Cons`](cons.md) and [`Either`](either.md): the product and sum lists those entries are built into.
- [`HasFields`](../traits/shape/has_fields.md): exposes a type's whole list of `Field` entries.
- [`HasField`](../traits/field-access/has_field.md): single-field access against a matching tag.
- [`#[derive(HasFields)]`](../derives/derive_has_fields.md): assigns the list of entries to a type.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): where field-name entries are used at scale.
- [Extensible variants](/docs/concepts/extensible-variants): the same entry naming a variant.

## Source

- The type and its `From`, `Debug`, `PartialEq`, and `Eq` impls:
  [`field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/field.rs)
- Consumed by the field machinery:
  [`has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_fields.rs),
  with rebuilding in
  [`from_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/from_fields.rs)
  and
  [`to_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/to_fields.rs)
- The derive that emits `Product!`/`Sum!` lists of entries:
  [`derive_has_fields/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
