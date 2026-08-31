---
sidebar_label: 'Either'
sidebar_position: 7
---

# `Either`

The head-or-rest cell of the sum list: the recursive list that describes an enum, one variant at a
time.

:::info

### Generated machinery

**You are not expected to write `Either` by hand.** You build a sum list with the
[`Sum!`](../macros/sum.md) macro, and an enum's variant list comes from
[`#[derive(HasFields)]`](../derives/derive_has_fields.md). You meet `Either` in an expansion and in a
variant-mismatch error, and this page explains its shape so those read clearly.

:::

## Overview

`Either<Head, Tail>` represents a choice among several types as a single type, so an enum's variants can
be reasoned about generically. Where the [product list](cons.md) holds a value for *every* element at
once, the sum list holds a value for exactly *one* of its branches: a tagged union, or *anonymous sum
type*. `Either` branches at each step, and [`Void`](void.md) marks the end, so the two together form a
coproduct that code can walk without knowing the concrete enum it came from.

This list makes structural, variant-by-variant operations work across every enum uniformly. An
enum's variants are exposed as one sum type through [`HasFields`](../traits/shape/has_fields.md), so a
provider written once to recurse over the `Either` and `Void` branches can match, dispatch on, or
construct *any* enum's variants. This is the basis for CGP's extensible-variant machinery: a variant is
reached by walking the nested branches rather than by a hand-written `match` against a fixed enum.

You write this list through the [`Sum!`](../macros/sum.md) macro. `Sum![A, B, C]` is the right-nested
`Either` chain terminated by `Void`. The branches are most often [`Field`](field.md) entries pairing a
variant name with its payload, so an enum's shape becomes a `Sum!` of `Field` branches over this list.

## Definition

`Either` is a two-case enum that selects the head or defers to the rest:

```rust
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Either<Head, Tail> {
    Left(Head),
    Right(Tail),
}
```

`Head` is the type of the current branch and `Tail` is the rest of the chain, itself another `Either`
or, at the end, [`Void`](void.md). `Left(Head)` carries a value of the head type; `Right(Tail)` carries a
value belonging somewhere further down the chain. It derives `Eq`, `PartialEq`, `Debug`, and `Clone`, so
a sum of values that implement those traits inherits them.

## Behavior

A sum of any width is an `Either` chain ending in `Void`, nested to the right. The type `Sum![A, B, C]`
is `Either<A, Either<B, Either<C, Void>>>`, and the empty `Sum![]` is just `Void`. A value selects one
branch by how deep it sits: `Left(a)` is an `A`, `Right(Left(b))` is a `B`, and `Right(Right(Left(c)))`
is a `C`. Reaching the `Void` position would mean the value matched none of the listed branches, which is
impossible, because `Void` has no values, so the chain is closed off at its end.

Generic code consumes the sum by recursing on its two cases, mirroring how it folds the product list but
branching instead of pairing. A `Left` is handled directly as the head; a `Right` defers to a trait impl
on the `Tail`, recursing until a `Left` is found. The base case is the [`Void`](void.md) terminator, and
here the difference from the product list matters: a product ends in the constructible [`Nil`](nil.md),
but a sum ends in the uninhabited `Void`, because an empty choice has no value to pick.

## Examples

The sum list appears most visibly as the `Fields` of an enum that derives
[`#[derive(HasFields)]`](../derives/derive_has_fields.md), where the [`Sum!`](../macros/sum.md) sugar
hides the `Either`/`Void` chain:

```rust
use cgp::prelude::*;

#[derive(HasFields)]
pub enum Shape {
    Circle(f64),
    Rectangle { width: f64, height: f64 },
}

// generated (schematically):
// impl HasFields for Shape {
//     type Fields = Sum![
//         Field<Symbol!("Circle"), f64>,
//         Field<Symbol!("Rectangle"), Product![
//             Field<Symbol!("width"), f64>,
//             Field<Symbol!("height"), f64>,
//         ]>,
//     ];
//     // i.e. Either<Field<Symbol!("Circle"), f64>,
//     //          Either<Field<Symbol!("Rectangle"), _>, Void>>
// }
```

A standalone sum type can also be written through the sugar, and a value picks one branch by its nesting
depth:

```rust
use cgp::prelude::*;

type Token = Sum![u32, String, bool];
// Token == Either<u32, Either<String, Either<bool, Void>>>

let t: Token = Either::Right(Either::Left("hi".to_string())); // the String branch
```

## When to use it

**Read `Either` in an expansion; write [`Sum!`](../macros/sum.md) instead.** The sugar produces the
list, and spelling it out by hand is longer, harder to change, and identical in meaning.

- **Use [`#[derive(HasFields)]`](../derives/derive_has_fields.md) for an enum's shape** rather than
  declaring the `Either` chain yourself.
- **Decode an `Either` chain in an error by counting the `Right` wrappers.** The depth is which variant a
  value selects, and a mismatch is reported as a mismatch between two `Either` chains.
- **Reach for [`Cons`](cons.md), not `Either`, when every element is present at once.** A product holds a
  value for every element; a sum holds one for exactly one. They are duals, and mixing them produces a
  type error rather than a subtle bug.

## Common Mistakes

**A sum holds one branch, not all of them.** `Either<A, Either<B, Void>>` is a value that is *either* an
`A` or a `B`, not both. This is the opposite of the [product list](cons.md), and confusing the two is the
usual cause of a "expected `Either`, found `Cons`" error.

**The empty sum is the uninhabited [`Void`](void.md), not a value.** `Sum![]` is `Void`, which has no
values, so an empty choice cannot be constructed. An empty record can, because it ends in
[`Nil`](nil.md).

**Branch order is part of the type.** `Sum![A, B]` and `Sum![B, A]` are unrelated types. Because the
branches are name-tagged [`Field`](field.md)s and the operations match on names, this bites less than
it might, but the types still differ.

## Related constructs

- [`Void`](void.md) — the uninhabited end marker that terminates this list.
- [`Cons`](cons.md) and [`Nil`](nil.md) — the product list, the record-shaped dual of this one.
- [`Sum!`](../macros/sum.md) — the macro that folds element types onto this list.
- [`Field`](field.md) — what the branches usually are, pairing a variant name with its payload.
- [`HasFields`](../traits/shape/has_fields.md) — exposes an enum's shape as one of these lists.
- [`ExtractField`](../traits/variant/extract_field.md) — the extractor family that walks this list.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — the variant representation this list encodes.

## Source

- `Either` and [`Void`](void.md) are both in
  [`sum.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/sum.rs)
- The `Sum!` fold:
  [`types/sum.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/sum.rs)
- The enum `HasFields` derive that emits a `Sum!` of entries:
  [`derive_has_fields/sum.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields/sum.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
