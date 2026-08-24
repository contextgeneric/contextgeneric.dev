---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Type-level list spines

The recursive lists CGP folds over: one for a record, one for a variant, one for a string, and one for
a routing path. You write them through sugar and read them in expansions and errors.

## Overview

A **spine** is a list encoded as a type. Instead of holding several things in one flat type the way a
tuple does, a spine pairs a head with the rest of the list and terminates in an end marker, so generic
code can take the list apart one element at a time. That is what a tuple cannot do and a spine can, and
it is the mechanism behind every field-by-field and variant-by-variant operation in CGP.

All the spines share one shape: right-nested, terminated by an end marker, and consumed by recursing on
two cases. A trait implemented for the end marker gives the base case, and a blanket impl for the
head-and-tail cell gives the recursive step, usually constraining the tail to implement the same trait
so the recursion stops at the end. This one pattern processes a list of any length with no per-element
code.

Four spines cover four jobs, and they differ in what sits in the head and what marks the end.

The **product spine** describes a record. [`Cons`](cons.md) pairs a field with the rest, and
[`Nil`](nil.md) marks the end. A record is a `Cons` chain ending in `Nil`, and it is what
[`Product!`](../../macros/product.md) builds.

The **sum spine** describes a variant. [`Either`](either.md) selects the head or defers to the rest, and
[`Void`](void.md) marks the end. A variant is an `Either` chain ending in the uninhabited `Void`, and it
is what [`Sum!`](../../macros/sum.md) builds. The end marker is where the two data spines differ: a
record ends in the constructible `Nil`, a variant in the impossible `Void`.

The **string spine** describes a field name. [`Chars`](chars.md) holds one character and the rest of the
string, again ending in `Nil`, and it is what a [`Symbol!`](../../macros/symbol.md) wraps. It is the
product spine specialized so the head is a `const char` rather than a type.

The **path spine** describes a route through delegation tables. [`PathCons`](path_cons.md) holds one
segment and the rest of the path, ending in `Nil`, and it is what [`Path!`](../../macros/path.md) builds.
It differs from the product spine in one way: its segments may be unsized, because they are pure markers
rather than values.

You write none of these by hand. The sugar builds them, and the reason to know them is that a mismatched
shape or a routing failure prints the expanded spine, and counting the cells is how you read it.

## The pages

- [`Cons`](cons.md) — the head-and-tail cell of a record.
- [`Nil`](nil.md) — the end marker shared by the product, string, and path spines.
- [`Either`](either.md) — the head-or-rest cell of a variant.
- [`Void`](void.md) — the uninhabited end marker of a variant.
- [`Chars`](chars.md) — one character of a type-level string.
- [`PathCons`](path_cons.md) — one segment of a type-level path.

## The ideas behind them

- [Extensible records](/docs/concepts/extensible-records) — the record shape the product spine
  describes.
- [Extensible variants](/docs/concepts/extensible-variants) — the variant shape the sum spine describes.
- [Namespaces](/docs/concepts/namespaces) — where a path spine routes a component lookup.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
