---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Types

The types CGP is built out of: the type-level building blocks that carry information in the type system,
and the one ordinary runtime data type you return from a getter.

## Overview

Most of the types on these pages are **type-level building blocks**. Much of CGP's machinery works at the
type level, where a field name, a variant, a position, or a whole record shape is a *type* rather than a
value. These building blocks are how that is done, and you write almost none of them by hand. You meet
them when you read the code a macro generated, or when a wiring mistake prints a wall of nested types,
and this section exists so that such a type is legible.

One idea runs through nearly all of them, and it is worth holding before the rest: a type can carry
information the program never stores as a value. A struct can name a type parameter it keeps no field of,
so a name, a number, or a lifetime becomes part of a type's identity and the compiler matches on it during
trait resolution. [`PhantomData`](phantom_data.md) is the standard-library tool that makes this legal, so
it is the first page here and the mechanism the others build on.

The building blocks divide into two groups.

The **markers** each attach one piece of information at the type level. [`Index`](index_type.md) turns a
tuple-field position into a type, the way [`Symbol!`](../macros/symbol.md) turns a field name into one,
and [`Life`](life.md) lifts a lifetime into a type so it can travel through machinery that only accepts
types. [`Field`](field.md) uses such a tag: it pairs a value with its type-level name, so a record or a
variant entry knows what it is called.

The **recursive type-level lists** are the chains everything structural is built from: a head-and-tail
cell paired with a terminator, which generic code takes apart one element at a time. Four cover four jobs,
and each family's full account lives on its head cell. [`Cons`](cons.md) and [`Nil`](nil.md) build a
record, and `Cons` owns the type-level product. [`Either`](either.md) and [`Void`](void.md) build a
variant, and `Either` owns the type-level sum. [`Chars`](chars.md) builds a string, and
[`PathCons`](path_cons.md) builds a routing path. You write these through the sugar
([`Product!`](../macros/product.md), [`Sum!`](../macros/sum.md), [`Symbol!`](../macros/symbol.md), and
[`Path!`](../macros/path.md)) and read them in expansions and errors.

One type is not a building block at all. [`MRef`](mref.md) is an ordinary runtime value, the
owned-or-borrowed return type of a getter, and it is here because it is the one type in this section you
write on purpose.

## The ideas behind them

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — where the zero-sized
  provider structs that carry these markers come from.
- [Extensible records](/docs/concepts/extensible-records) — the record shape the product list describes.
- [Extensible variants](/docs/concepts/extensible-variants) — the variant shape the sum list describes.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
