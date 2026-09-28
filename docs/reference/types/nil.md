---
title: 'Nil — the end of a type-level list'
sidebar_label: 'Nil'
sidebar_position: 6
description: 'The end marker of the product, string, and path lists: an empty, constructible list that is also the shape of a unit struct.'
---

# `Nil`

The end marker of the product, string, and path lists: an empty, constructible list.

## Overview

`Nil` marks the end of a right-nested type-level list. Where a [`Cons`](cons.md) cell pairs a head with
the rest of the list, `Nil` is the rest when nothing is left, so a list of any length is a `Cons` chain
that finishes in `Nil`. On its own, `Nil` is the empty list.

The same marker terminates every CGP list except the sum list. It ends the product list built from
[`Cons`](cons.md), the string list built from [`Chars`](chars.md), and the path list built from
[`PathCons`](path_cons.md). Only the sum list ends differently, in the uninhabited [`Void`](void.md). That
difference is the point of both markers. A record, a string, and a path can each be empty and still exist,
so their terminator is a real value.

## Definition

`Nil` is a unit struct:

```rust
#[derive(Eq, PartialEq, Clone, Default, Debug)]
pub struct Nil;
```

It holds nothing. Used as a tail it terminates a chain, and used on its own it is the empty list. It
derives `Eq`, `PartialEq`, `Clone`, `Default`, and `Debug`, so a list ending in `Nil` inherits those
traits structurally, and `Nil` itself compares equal to `Nil`. It is in the prelude, so
`use cgp::prelude::*;` is enough.

## Behavior

`Nil` is the base case of every recursion over a list it terminates. An operation that folds over a
product implements the step for [`Cons<Head, Tail>`](cons.md) and the base case for `Nil`, and the
recursion stops when it reaches `Nil`. The string and path lists work the same way: a walk over
[`Chars`](chars.md) stops at `Nil`, and a walk over [`PathCons`](path_cons.md) stops at `Nil`.

Because `Nil` is a real, constructible value, the empty product `Product![]` is `Nil`, and the empty
value `product![]` is the `Nil` value. The empty string `Symbol!("")` is `Symbol<0, Nil>`, and a
unit struct that derives [`HasFields`](../derives/derive_has_fields.md) has `Nil` as its shape. A
builder or a conversion returns `Nil` when every field of a shape has been consumed. This is why an
empty record is a value the program can hold. The contrast with [`Void`](void.md) is exact. A value
with every branch ruled out cannot exist, so a sum ends in an uninhabited marker, while a record
ends in this inhabited one.

## Examples

Each list ends in `Nil`, and the empty product, the empty string, and a unit struct's shape are all
`Nil`:

```rust
use cgp::prelude::*;

#[derive(HasFields)]
pub struct Marker;

pub fn demo() {
    // Each list ends in `Nil`; the annotations are the check.
    let _row: PhantomData<Cons<u32, Cons<bool, Nil>>> = PhantomData::<Product![u32, bool]>;
    let _name: PhantomData<Symbol<2, Chars<'h', Chars<'i', Nil>>>> =
        PhantomData::<Symbol!("hi")>;
    let _path: PhantomData<PathCons<Symbol!("a"), PathCons<Symbol!("b"), Nil>>> =
        PhantomData::<Path!(@a.b)>;

    // The empty product, the empty string, and a unit struct's shape are all `Nil`.
    let empty: Product![] = product![];
    assert_eq!(empty, Nil);
    let _empty_name: PhantomData<Symbol<0, Nil>> = PhantomData::<Symbol!("")>;
    let fields: Nil = Marker.to_fields();
    assert_eq!(fields, Nil);
}
```

Each `PhantomData` annotation compiles only if the macro on its right expands to the chain on its
left.

## When to use it

**You read `Nil` at the end of a chain, and you write it as the base case of a fold.** The macros
produce it everywhere else.

- **Read `Nil` as "the list ends here."** In an expanded `Cons`, `Chars`, or `PathCons` chain, the `Nil`
  is the terminator and marks the count of cells before it.
- **Implement a trait for `Nil`** as the base case when you fold over a product, beside the impl for
  [`Cons<Head, Tail>`](cons.md).
- **Expect `Nil`, not [`Void`](void.md), at the end of a record, a string, or a path.** A sum ends in
  `Void`, and the other lists end in `Nil`. The wrong terminator in an error usually means the product and
  sum families have been mixed up.

## Common Mistakes

**`Nil` is inhabited, and [`Void`](void.md) is not.** `Nil` is the real value of the empty product,
while `Void` is the uninhabited type of an empty choice. They are not interchangeable: variant
extraction relies on `Void` being uninhabited, which `Nil` is not.

**`Nil` terminates the string and path lists as well as the product list.** A `Nil` at the end of a
[`Chars`](chars.md) chain or a [`PathCons`](path_cons.md) chain is the same marker doing the same job, so
it is expected outside a record.

## Related constructs

- [`Cons`](cons.md): the head-and-tail cell this marker terminates in a record.
- [`Void`](void.md): the sum list's uninhabited end marker, the counterpart to this one.
- [`Chars`](chars.md) and [`PathCons`](path_cons.md): the string and path lists `Nil` also terminates.
- [`Product!`](../macros/product.md): the macro whose empty form is `Nil`.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): where the empty and terminating record shape
  matters.

## Source

- The type:
  [`nil.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/nil.rs)
- The lists it terminates:
  [`cons.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/cons.rs),
  [`chars.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/chars.rs),
  and
  [`path.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/path.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
