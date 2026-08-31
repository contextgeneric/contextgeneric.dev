---
sidebar_label: 'Cons'
sidebar_position: 5
---

# `Cons`

The head-and-tail cell of the product list: the recursive list that describes a record, one field at a
time.

## Overview

`Cons<Head, Tail>` represents an ordered sequence of types as a single type, so a collection of fields
can be reasoned about generically. A Rust tuple holds several things at once but cannot be taken apart
by generic code element by element; a recursive list can. `Cons` pairs the first element with the rest of
the list, and [`Nil`](nil.md) marks the end, so the two together form an *anonymous product type*: a
record-shaped value that code can walk without knowing the concrete struct it came from.

This list makes structural, field-by-field operations work across every struct uniformly. A
struct's fields are exposed as one list type through [`HasFields`](../traits/shape/has_fields.md), so a
provider written once to recurse over `Cons` and `Nil` can iterate, transform, read, or rebuild *any*
struct's fields. Each step handles the `Head`, then recurses into the `Tail`, until it reaches `Nil` and
stops.

You write this list through the [`Product!`](../macros/product.md) macro rather than by hand.
`Product![A, B, C]` is the right-nested `Cons` chain, and the value macro `product![a, b, c]` builds a
matching value. The elements are most often [`Field`](field.md) entries pairing a name with a value,
so a struct's layout becomes a `Product!` of `Field` cells over this list.

## Definition

`Cons` is a tuple struct holding the first element and the rest of the list:

```rust
#[derive(Eq, PartialEq, Clone, Default, Debug)]
pub struct Cons<Head, Tail>(pub Head, pub Tail);
```

`Head` is the first element's type and `Tail` is the rest of the list, itself another `Cons` or, at the
end, [`Nil`](nil.md). Both positional fields are public, so `Cons(head, tail)` builds a cell and `.0` and
`.1` reach its parts. Unlike the zero-sized [`Chars`](chars.md) and [`PathCons`](path_cons.md) lists,
`Cons` holds real values: it is as large as its elements laid out by nesting, with nothing boxed or
virtual. It derives `Eq`, `PartialEq`, `Clone`, `Default`, and `Debug`, so a list of values that
implement those traits inherits them structurally, comparing head to head down the chain.

## Behavior

A list of any length is a `Cons` chain ending in `Nil`, nested to the right. The type `Product![A, B, C]`
is `Cons<A, Cons<B, Cons<C, Nil>>>`, and the empty `Product![]` is just `Nil`. The matching value is
built with the tuple-struct constructor, `Cons(a, Cons(b, Cons(c, Nil)))`, so a `product!` value is an
ordinary owned value whose type is exactly the one `Product!` produces over the same elements' types.

Generic code consumes the list by recursing on its two cases. A trait implemented for `Nil` supplies
the base case, the empty list, and a blanket impl for `Cons<Head, Tail>` supplies the recursive step,
usually constraining `Tail` to implement the same trait so the recursion bottoms out at `Nil`. This
pairing of a `Nil` impl with a `Cons<Head, Tail>` impl is the standard shape for any operation that
folds over a product, and it is how the field machinery processes a struct of any width with no per-field
code.

## Examples

The product list appears most visibly as the `Fields` of a struct that derives
[`HasFields`](../derives/derive_has_fields.md), where the [`Product!`](../macros/product.md) sugar
hides the `Cons`/`Nil` chain:

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
//     // i.e. Cons<Field<Symbol!("name"), String>,
//     //          Cons<Field<Symbol!("age"), u8>, Nil>>
// }
```

A standalone list type and a matching value can also be written through the sugar, which expands to the
nested `Cons` form:

```rust
use cgp::prelude::*;

type Row = Product![u32, String, bool];
let row: Row = product![1, "hi".to_string(), true];
// Row == Cons<u32, Cons<String, Cons<bool, Nil>>>
// row == Cons(1, Cons("hi".to_string(), Cons(true, Nil)))
```

## When to use it

**Read `Cons` in an expansion; write [`Product!`](../macros/product.md) instead.** The sugar produces
the list, and spelling it out by hand is longer, harder to change, and identical in meaning.

- **Use [`#[derive(HasFields)]`](../derives/derive_has_fields.md) for a struct's shape** rather than
  declaring the `Cons` chain yourself, since a hand-written list restates the struct and the two drift
  apart.
- **Use [`Product!`](../macros/product.md) for a list you write on purpose,** such as a handler
  pipeline, and let it build the list.
- **Decode a `Cons` chain in an error by counting cells.** A field-list mismatch is reported as a
  mismatch between two `Cons` chains, and the position where they diverge is the field that differs.

## Common Mistakes

**A one-element list is not the element.** `Product![T]` is `Cons<T, Nil>`, a distinct type from `T`, so a
list of one still carries its cell.

**Element order is part of the type.** `Cons<A, Cons<B, Nil>>` and `Cons<B, Cons<A, Nil>>` are unrelated
types. For a field list this matters less than it sounds, because the entries are name-tagged
[`Field`](field.md)s and the operations match on names; for a handler pipeline the order is the
execution order.

**The empty product is [`Nil`](nil.md), a real value.** That is the difference from the sum list, whose
empty form is the uninhabited [`Void`](void.md). An empty record exists; an empty choice cannot.

## Related constructs

- [`Nil`](nil.md) — the end marker that terminates this list.
- [`Either`](either.md) and [`Void`](void.md) — the sum list, the choice-shaped dual of this one.
- [`Chars`](chars.md) — the same list specialized to a `const char` head.
- [`Product!`](../macros/product.md) — the macro that folds elements onto this list.
- [`Field`](field.md) — what the elements usually are, pairing a name with a value.
- [`HasFields`](../traits/shape/has_fields.md) — exposes a struct's shape as one of these lists.
- [`#[derive(HasFields)]`](../derives/derive_has_fields.md) — generates that list for a struct.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — the record representation this list encodes.

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

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
