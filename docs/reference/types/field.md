---
sidebar_label: 'Field'
sidebar_position: 2
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

`Tag` is the type-level name of the field. It appears only inside `PhantomData<Tag>`, never in a stored
field. It is usually a type-level string such as `Symbol!("name")` for a named field, or a type-level
number such as `Index<0>` for a tuple position. `Value` is the field's actual type, and `value` is the
only data the struct keeps. Apart from the tag, a `Field` is a thin wrapper around its `Value`.

## Behavior

You build a `Field` from a value without a tag argument, because the tag comes from the target type
rather than from you. The `From<Value>` impl fills in `value` and sets `phantom` to `PhantomData`. So
`let f: Field<Symbol!("name"), String> = "Alice".to_string().into();` compiles, and the compiler infers
the tag from the expected type. This is why generated code builds each entry with a plain `.into()`.

The other trait impls defer to the value and ignore the tag, so a `Field` behaves like its `Value` for
comparison and printing. `Debug` forwards to the value's `Debug` and does not show the tag.
`PartialEq` and `Eq` compare only `value`, each gated on the matching bound on `Value`. Two `Field`
values are equal when their values are equal. The tag is a compile-time matter and plays no part at run
time.

Because the tag lives only in `PhantomData`, code that needs the name reads it from the `Tag` parameter
through trait resolution rather than from stored data. For example, it matches a
`Field<Symbol!("name"), _>` against a [`HasField<Symbol!("name")>`](../traits/field-access/has_field.md)
bound.

## Examples

`Field` appears most often inside the shape a derive generates, where each struct field becomes one
entry tagged by its [`Symbol!`](../macros/symbol.md) name:

```rust
use cgp::prelude::*;

#[derive(HasFields)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

// generated:
// impl HasFields for Person {
//     type Fields = Product![
//         Field<Symbol!("name"), String>,
//         Field<Symbol!("age"), u8>,
//     ];
// }
```

You can also build a single `Field` directly from its value, with the type annotation supplying the tag:

```rust
use cgp::prelude::*;

let name: Field<Symbol!("name"), String> = "Alice".to_string().into();
assert_eq!(name.value, "Alice");
```

For a tuple-struct field the tag is an [`Index`](index_type.md) rather than a `Symbol!`, so the same
wrapper names a positional field, as `Field<Index<0>, u32>`.

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

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
