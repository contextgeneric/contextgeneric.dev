---
sidebar_label: 'Nil'
sidebar_position: 6
---

# `Nil`

The end marker of the product, string, and path lists: an empty, constructible list.

## Overview

`Nil` marks the end of a right-nested type-level list. Where a [`Cons`](cons.md) cell pairs a head with
the rest of the list, `Nil` is the rest when there is nothing left, so a list of any length is a `Cons`
chain that finishes in `Nil`. On its own, `Nil` is the empty list.

The same marker terminates every CGP list but one. It ends the product list built from
[`Cons`](cons.md), the string list built from [`Chars`](chars.md), and the path list built from
[`PathCons`](path_cons.md). Only the sum list ends differently, in the uninhabited [`Void`](void.md),
and that difference is the point of both markers: a record, a string, and a path can each be empty and
still exist, so their terminator is a real value.

## Definition

`Nil` is a unit struct:

```rust
#[derive(Eq, PartialEq, Clone, Default, Debug)]
pub struct Nil;
```

It carries no data. Used as a tail it terminates a chain, and used on its own it is the empty list. It
derives `Eq`, `PartialEq`, `Clone`, `Default`, and `Debug`, so a list ending in `Nil` inherits those
traits structurally and `Nil` itself compares equal to `Nil`.

## Behavior

`Nil` is the base case of every recursion over a list it terminates. An operation that folds over a
product implements the step for [`Cons<Head, Tail>`](cons.md) and the base case for `Nil`, and the
recursion bottoms out when it reaches `Nil`. The string and path lists do the same: a walk over
[`Chars`](chars.md) stops at `Nil`, and a walk over [`PathCons`](path_cons.md) stops at `Nil`.

Because `Nil` is a real, constructible value, the empty product `Product![]` is `Nil` and the empty value
`product![]` is `Nil` as a value. A builder or a conversion returns this when a shape has no
fields left, and it is why an empty record is a value the program can hold. The contrast with
[`Void`](void.md) is exact: a value with every field ruled out cannot exist, so a sum ends in an
uninhabited marker, while a record ends in this inhabited one.

## Examples

`Nil` closes off every product chain, visible when the [`Product!`](../macros/product.md) sugar is
expanded:

```rust
use cgp::prelude::*;

// Product![u32, bool] expands to:
type Row = Cons<u32, Cons<bool, Nil>>;

// the empty list is Nil alone:
type Empty = Product![]; // == Nil
```

It also terminates a [`Symbol!`](../macros/symbol.md) character chain and a
[`Path!`](../macros/path.md) segment chain:

```rust
// Symbol!("hi")  ends its Chars chain in Nil
// Path!(@a.b)    ends its PathCons chain in Nil
```

## When to use it

**You read `Nil` at the end of a chain; you do not write it.** The sugar produces it, and recognizing it
is all that is asked.

- **Read `Nil` as "the list ends here."** In an expanded `Cons`, `Chars`, or `PathCons` chain, the `Nil`
  is the terminator and marks the count of cells before it.
- **Expect `Nil`, not [`Void`](void.md), at the end of a record, a string, or a path.** A sum ends in
  `Void`; the other three lists end in `Nil`. Meeting the wrong terminator in an error usually means the
  product and sum families have been crossed.

## Common Mistakes

**`Nil` is inhabited; [`Void`](void.md) is not.** `Nil` is a real value the empty product is, while `Void`
is an uninhabited type an empty choice would be. They are not interchangeable, and the whole extractor
machinery depends on the difference.

**`Nil` terminates three lists, not just the product one.** A `Nil` at the end of a
[`Chars`](chars.md) chain or a [`PathCons`](path_cons.md) chain is the same marker doing the same job, so
seeing it outside a record is expected.

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

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
