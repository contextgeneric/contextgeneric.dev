---
title: 'Cons — the product list cell'
sidebar_label: 'Cons'
sidebar_position: 5
description: 'The head-and-tail cell of the type-level product list, which describes a record one field at a time so generic code can walk any struct''s fields.'
---

# `Cons`

The head-and-tail cell of the product list: the recursive list that describes a record, one field at a
time.

## Overview

`Cons<Head, Tail>` represents an ordered sequence of types as a single type, so that generic code can
reason about a collection of fields. A Rust tuple holds several things at once, but generic code cannot
take it apart element by element. A recursive list allows that. `Cons` pairs the
first element with the rest of the list, and [`Nil`](nil.md) marks the end. Together they form an
*anonymous product type*: a record-shaped value that code can walk without knowing the concrete struct it
came from.

This list makes structural, field-by-field operations work across every struct in the same way.
[`HasFields`](../traits/shape/has_fields.md) exposes a struct's fields as one list type, so a provider
written once to recurse over `Cons` and `Nil` can iterate, transform, read, or rebuild *any* struct's
fields. Each step handles the `Head`, then recurses into the `Tail`, until it reaches `Nil` and stops.

You write this list through the [`Product!`](../macros/product.md) macro rather than directly.
`Product![A, B, C]` is the right-nested `Cons` chain, and the value macro `product![a, b, c]` builds a
matching value. The elements are most often [`Field`](field.md) entries that pair a name with a value, so
a struct's layout becomes a `Product!` of `Field` cells over this list.

## Definition

`Cons` is a tuple struct holding the first element and the rest of the list:

```rust
#[derive(Eq, PartialEq, Clone, Default, Debug)]
pub struct Cons<Head, Tail>(pub Head, pub Tail);
```

`Head` is the first element's type, and `Tail` is the rest of the list, which is another `Cons` or,
at the end, [`Nil`](nil.md). Both positional fields are public, so `Cons(head, tail)` builds a cell,
and `.0` and `.1` reach its parts. Unlike the zero-sized [`Chars`](chars.md) and
[`PathCons`](path_cons.md) lists, `Cons` holds real values. It is as large as its nested elements,
without boxing or indirection. It derives `Eq`, `PartialEq`, `Clone`, `Default`, and `Debug`, so a
list of values that implement those traits inherits them structurally: equality compares head to
head down the chain, and `Default` gives the list of defaults. It is in the prelude, and so is
`Nil`.

## Behavior

A list of any length is a `Cons` chain ending in `Nil`, nested to the right. The type
`Product![A, B, C]` is `Cons<A, Cons<B, Cons<C, Nil>>>`, and the empty `Product![]` is `Nil` alone.
The tuple-struct constructor builds the matching value, `Cons(a, Cons(b, Cons(c, Nil)))`. So a
`product!` value is an ordinary owned value, and its type is exactly the one `Product!` produces
over the same element types.

Generic code consumes the list by recursing on its cases. A trait impl for `Nil` supplies the base case,
the empty list. A blanket impl for `Cons<Head, Tail>` supplies the recursive step, and it usually
constrains `Tail` to implement the same trait, so that the recursion ends at `Nil`. This pairing of a
`Nil` impl with a `Cons<Head, Tail>` impl is the standard shape for any operation that folds over a
product. It is how the field machinery processes a struct of any width without per-field code.

## Examples

A `product!` value is the nested `Cons` value, a struct's shape is a `Cons` chain of
[`Field`](field.md) entries, and a fold over the list is one impl for `Nil` and one for `Cons`:

```rust
use cgp::prelude::*;

#[derive(HasFields)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

pub type Row = Product![u32, String, bool];

// A fold over a product list: `Nil` is the base case, `Cons` the step.
pub trait Len {
    const LEN: usize;
}

impl Len for Nil {
    const LEN: usize = 0;
}

impl<Head, Tail: Len> Len for Cons<Head, Tail> {
    const LEN: usize = 1 + Tail::LEN;
}

pub fn demo() {
    // `product!` builds the same nested value a hand-written chain does.
    let row: Row = product![1, "hi".to_owned(), true];
    let by_hand: Cons<u32, Cons<String, Cons<bool, Nil>>> =
        Cons(1, Cons("hi".to_owned(), Cons(true, Nil)));
    assert_eq!(row, by_hand);

    // A struct's shape is a `Cons` chain of `Field` entries.
    let Cons(name, Cons(age, Nil)): Cons<
        Field<Symbol!("name"), String>,
        Cons<Field<Symbol!("age"), u8>, Nil>,
    > = Person {
        name: "Alice".to_owned(),
        age: 30,
    }
    .to_fields();
    assert_eq!((name.value.as_str(), age.value), ("Alice", 30));

    assert_eq!(<Row as Len>::LEN, 3);
    assert_eq!(<<Person as HasFields>::Fields as Len>::LEN, 2);
}
```

The annotation on the `Person` pattern is the check that the derive assigned the `Cons` chain
written there. [`#[derive(HasFields)]`](../derives/derive_has_fields.md) writes it with the macro,
as `type Fields = Product![Field<Symbol!("name"), String>, Field<Symbol!("age"), u8>];`.

## When to use it

**Read `Cons` in an expansion, and write [`Product!`](../macros/product.md) instead.** The macro produces
the list. Writing the chain out yourself is longer, harder to change, and identical in meaning.

- **Use [`#[derive(HasFields)]`](../derives/derive_has_fields.md) for a struct's shape** rather than
  declaring the `Cons` chain yourself, because a list you write yourself restates the struct, and the two
  drift apart.
- **Use [`Product!`](../macros/product.md) for a list you write deliberately,** such as a handler
  pipeline, and let the macro build the chain.
- **Name `Cons` and `Nil` in the impls of a fold,** as `Len` does above. That is the one place the
  cells themselves are written, since an impl matches on the cell rather than on the macro.
- **Decode a `Cons` chain in an error by counting cells.** The compiler reports a field-list mismatch as
  a mismatch between two `Cons` chains, and the position where they diverge is the field that differs.

## Common Mistakes

**A one-element list is not the element.** `Product![T]` is `Cons<T, Nil>`, a distinct type from
`T`, so a list of one still carries its cell. A one-field tuple struct is the exception on the
derive's side: its shape is the field's type itself rather than a list of one, as
[`Field`](field.md) describes.

**Element order is part of the type.** `Cons<A, Cons<B, Nil>>` and `Cons<B, Cons<A, Nil>>` are unrelated
types. For a field list this matters less than it sounds, because the entries are name-tagged
[`Field`](field.md)s and the operations match on names. For a handler pipeline, the order is the
execution order.

**The empty product is [`Nil`](nil.md), a real value.** This differs from the sum list, whose empty form
is the uninhabited [`Void`](void.md). An empty record can exist, but an empty choice cannot.

## Related constructs

- [`Nil`](nil.md): the end marker that terminates this list.
- [`Either`](either.md) and [`Void`](void.md): the sum list, the choice-shaped dual of this one.
- [`Chars`](chars.md): the same list specialized to a `const char` head.
- [`Product!`](../macros/product.md): the macro that folds elements onto this list.
- [`Field`](field.md): what the elements usually are, pairing a name with a value.
- [`HasFields`](../traits/shape/has_fields.md): exposes a struct's shape as one of these lists.
- [`#[derive(HasFields)]`](../derives/derive_has_fields.md): generates that list for a struct.
- [`AppendProduct`](../traits/type-level/append_product.md): one of the operations that transform a
  product list.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): the record representation this list encodes.

## Source

- The type:
  [`cons.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/cons.rs),
  with [`Nil`](nil.md) in
  [`nil.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/nil.rs)
- The `Product!`/`product!` folds:
  [`types/product/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/product)
- The `HasFields` machinery that recurses over it:
  [`has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_fields.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
