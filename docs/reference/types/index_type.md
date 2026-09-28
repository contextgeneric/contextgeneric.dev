---
title: 'Index — a tuple field position as a type'
sidebar_label: 'Index'
sidebar_position: 3
description: 'A number lifted into a type, so a tuple-struct field can be named by its position the way Symbol! names a field by its string.'
---

# `Index`

A number lifted into a type, so a tuple-struct field can be named by its position the way
[`Symbol!`](../macros/symbol.md) names a field by its string.

## Overview

`Index<const I: usize>` encodes a `usize` at the type level. CGP's getter mechanism keys every field by a
*type* tag, but a tuple-struct field has only a position and lacks a string name to turn into a
[`Symbol!`](../macros/symbol.md). So the position itself becomes a type, and positional fields can use the
same trait-resolution machinery as named fields. `Index<I>` is that type. It carries a `usize` as a const
parameter and nothing else, so `Index<0>`, `Index<1>`, and `Index<2>` are distinct types that stand for a
tuple struct's positional fields.

Encoding the position as a type lets positional field access resolve through traits. Here a
**context** is the type the method runs on, which supplies the values it needs as its fields.
Because `Index<0>` is a type, a context can carry a
[`HasField<Index<0>>`](../traits/field-access/has_field.md) impl for its first field and a
`HasField<Index<1>>` impl for its second, side by side. The compiler selects the right one from the
tag alone, as it does for differently named `Symbol!` tags. `Index` is the numeric counterpart to
`Symbol!`. A field is keyed by a `Symbol!` when it has a name, and by an `Index` when it has only a
position.

`Index` is a zero-sized marker without runtime data. Its only job is to make a number available at the
type level. So it can serve as a [`HasField`](../traits/field-access/has_field.md) tag, as the tag of
a [`Field`](field.md) entry inside a tuple struct's shape, and as the type inside a
[`PhantomData`](phantom_data.md) wherever compile-time code needs a positional name.

## Definition

`Index` is a zero-sized struct parameterized only by a const `usize`:

```rust
#[derive(Eq, PartialEq, Clone, Copy, Default)]
pub struct Index<const I: usize>;
```

`I` is the position the type represents: `Index<0>` for the field at offset zero, and so on. The
struct is empty, so a value of `Index<I>` holds nothing, and the number lives entirely in the type.
The derived `Default`, `Clone`, and `Copy` make a value available whenever code needs one. `Eq` and
`PartialEq` treat any two values of the same `Index<I>` as equal, because they hold nothing.
`Index<I>` also implements `Display` and `Debug`, and both print the underlying number. So
`Index<0>` displays as `0`, and output and diagnostics show the position a tag stands for. It is in
the prelude, so `use cgp::prelude::*;` is enough.

## Behavior

A tuple struct keys each of its fields by `Index<N>`, counting from zero, so code reads the field at
position `N` through the tag `Index<N>`. When a tuple struct derives
[`#[derive(HasField)]`](../derives/derive_has_field.md), the generated impls use `Index<0>` for the
`.0` field, `Index<1>` for `.1`, and so on, with a `HasFieldMut` impl beside each `HasField` one.
Each `get_field(PhantomData::<Index<N>>)` call maps to the matching positional access.

The same tags name the entries of a tuple struct's [`HasFields`](../traits/shape/has_fields.md)
shape when it has two or more fields, a list of `Field<Index<N>, _>` entries. So generic code that
walks the field list reads positions where it would read `Symbol!` names for a named struct. A tuple
struct with exactly one field is the exception: its shape is the field's type itself, without a
`Field<Index<0>, _>` around it, although its `HasField<Index<0>>` impl still exists.

Because `Index<I>` is zero-sized and the position lives in the type, a field access by index resolves
entirely at compile time. The access does not check an array bound and does not index at run time.
Selecting the wrong index is a type error rather than a panic, because a three-field struct lacks a
`HasField` impl for `Index<5>`.

## Examples

A tuple struct that derives `HasField` is read by position, and an `Index` prints the number it
stands for:

```rust
use cgp::prelude::*;

#[derive(HasField)]
pub struct Pair(pub u32, pub String);

pub fn demo() {
    let pair = Pair(7, "hi".to_owned());

    assert_eq!(*pair.get_field(PhantomData::<Index<0>>), 7);
    assert_eq!(pair.get_field(PhantomData::<Index<1>>), "hi");

    // `Display` and `Debug` both print the number.
    assert_eq!(Index::<2>.to_string(), "2");
    assert_eq!(format!("{:?}", Index::<2>), "2");
}
```

The derive generates one impl pair per position. For the first field it writes:

```rust
impl HasField<Index<0>> for Pair {
    type Value = u32;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<0>>) -> &Self::Value {
        &self.0
    }
}
```

and a matching `HasFieldMut<Index<0>>` impl returning `&mut self.0`.

## When to use it

**Use `Index` for a tuple-field tag, and let the derive produce it where it can.** The derive generates it
for every field of a tuple struct that derives [`#[derive(HasField)]`](../derives/derive_has_field.md),
so you write it only when you tag a positional field yourself.

- **Use [`Symbol!`](../macros/symbol.md) for a named field,** not an `Index`. Named and positional fields
  use different tags, and the derive generates the right one for each.
- **Write `Index<N>` for a tuple position,** never a `Symbol!` of the number. `Symbol!("0")` is a string
  tag and does not match any field of a tuple struct.
- **Read `Index<N>` in an error as a position.** A missing `HasField<Index<2>>` bound names the third
  field of a tuple struct.

## Common Mistakes

**A position the struct lacks is a type error, not a panic.** Reading `Index<5>` on a three-field
tuple struct does not compile:

```rust
#[derive(HasField)]
pub struct Point(pub f64, pub f64, pub f64);

pub fn sixth(point: &Point) -> f64 {
    *point.get_field(PhantomData::<Index<5>>)
}
```

The compiler reports
``error[E0277]: the trait bound `Point: cgp::prelude::HasField<cgp::prelude::Index<5>>` is not satisfied``,
and lists the three `HasField<Index<0>>` to `HasField<Index<2>>` impls the struct does have. Indices
count from zero, as Rust's own `.0` access does, so an off-by-one error surfaces the same way.

**`Index<N>` and `Symbol!("N")` are different types.** A tuple field is keyed by the number lifted
into a type, not by a string of the digit, so `Symbol!("0")` never matches a tuple field and the
derive never generates it. `pair.get_field(PhantomData::<Symbol!("0")>)` on the `Pair` above fails
with
``error[E0277]: the trait bound `Pair: cgp::prelude::HasField<cgp::prelude::Symbol<1, cgp::prelude::Chars<'0', Nil>>>` is not satisfied``.

## Related constructs

- [`Symbol!`](../macros/symbol.md): the string tag for a named field, the counterpart to this numeric
  one.
- [`HasField`](../traits/field-access/has_field.md): what a tag is looked up through.
- [`#[derive(HasField)]`](../derives/derive_has_field.md): generates an `Index` tag per tuple field.
- [`Field`](field.md): carries an `Index` tag beside a positional value.
- [`PhantomData`](phantom_data.md): how an `Index` tag is passed to `get_field`.
- [`HasFields`](../traits/shape/has_fields.md): the tuple struct's shape whose entries this tag names.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): where positional field access is used at
  scale.

## Source

- The type and its `Display` and `Debug` impls:
  [`index.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/index.rs)
- The `#[derive(HasField)]` codegen that tags tuple fields with `Index<N>`:
  [`cgp_data/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data),
  and the `HasField` trait it targets:
  [`has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
