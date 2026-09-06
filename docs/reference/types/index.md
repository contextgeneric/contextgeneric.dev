---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Types

The types CGP is built from: the type-level building blocks that carry information in the type system, and
the one ordinary runtime type you return from a getter.

## Overview

Most types on these pages are **type-level building blocks**, and you read them far more often than you
write them. Much of CGP's machinery runs at the type level, where a field name, a variant, a position, or
a whole record shape is a *type* rather than a value. These building blocks carry that information. A
macro writes almost all of them for you, so you meet them when you read generated code or when a wiring
mistake reports a deeply nested type. This section exists to make such a type readable.

One idea runs through nearly all of them: a type can carry information the program never stores as a
value. A struct can name a type parameter without keeping a field of that type, so a name, a number, or a
lifetime becomes part of the type's identity, and the compiler matches on it during trait resolution.
[`PhantomData`](phantom_data.md) is the standard-library marker that makes this legal. It comes first
here, because the other types build on it.

The building blocks are either markers or recursive type-level lists.

Each **marker** attaches one piece of information at the type level. [`Index`](index_type.md) turns a
tuple-field position into a type, in the same way that [`Symbol!`](../macros/symbol.md) turns a field
name into one. [`Life`](life.md) lifts a lifetime into a type, so that it can pass through machinery that
accepts only types. [`Field`](field.md) uses such a tag: it pairs a value with its type-level name, so a
record entry or a variant entry carries its own name.

The **recursive type-level lists** are the chains that every structural shape is built from. Each list
pairs a head-and-tail cell with a terminator, and generic code takes the list apart one element at a time.
Each family covers one job, and the head cell's page carries the full account of that family.
[`Cons`](cons.md) and [`Nil`](nil.md) build a record, and the `Cons` page explains the type-level product.
[`Either`](either.md) and [`Void`](void.md) build a variant, and the `Either` page explains the type-level
sum. [`Chars`](chars.md) builds a string, and [`PathCons`](path_cons.md) builds a routing path. You write
these lists through the macros ([`Product!`](../macros/product.md), [`Sum!`](../macros/sum.md),
[`Symbol!`](../macros/symbol.md), and [`Path!`](../macros/path.md)) and read them in expansions and
errors.

[`MRef`](mref.md) is the one type here that is not a building block. It is an ordinary runtime value, the
owned-or-borrowed return type of a getter. It belongs in this section because it is the one type here that
you write yourself.

## The ideas behind them

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): where the zero-sized
  provider structs that carry these markers come from.
- [Extensible records](/docs/concepts/extensible-records): the record shape the product list describes.
- [Extensible variants](/docs/concepts/extensible-variants): the variant shape the sum list describes.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
