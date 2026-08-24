---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Type-level building blocks

The small types CGP is built out of: the markers that carry information in the type system, the
recursive lists that describe a struct or an enum, and the wrappers you meet in an expansion or an
error message.

## Overview

Most CGP machinery works at the type level, where a field name, a variant, a position, or a whole
record shape is a *type* rather than a value. The types on these pages are how that is done. You write
almost none of them by hand. You meet them when you read the code a macro generated, or when a wiring
mistake prints a wall of nested types, and this section exists so that such a type is legible.

One idea runs through nearly all of them, and it is worth holding before the rest: a type can carry
information the program never stores as a value. A struct can name a type parameter it keeps no field
of, so a name, a number, or a lifetime becomes part of a type's identity and the compiler matches on
it during trait resolution. [`PhantomData`](phantom_data.md) is the standard-library tool that makes
this legal, so it is the first page here and the mechanism the others build on.

The rest divide into a few groups.

The **markers** each attach one piece of information at the type level. [`Index`](index_type.md) turns a
tuple-field position into a type, the way [`Symbol!`](../macros/symbol.md) turns a field name into one,
and [`Life`](life.md) lifts a lifetime into a type so it can travel through machinery that only accepts
types. [`Field`](field.md) uses such a tag: it pairs a value with its type-level name, so a record or a
variant entry knows what it is called.

The **[list spines](spines/index.md)** are the recursive lists everything structural is built from:
[`Cons`](spines/cons.md) and [`Nil`](spines/nil.md) for a record, [`Either`](spines/either.md) and
[`Void`](spines/void.md) for a variant, [`Chars`](spines/chars.md) for a string, and
[`PathCons`](spines/path_cons.md) for a routing path. You write these through the sugar —
[`Product!`](../macros/product.md), [`Sum!`](../macros/sum.md), [`Symbol!`](../macros/symbol.md), and
[`Path!`](../macros/path.md) — and read them in expansions and errors.

One type stands apart: [`MRef`](mref.md) is an ordinary runtime value, not a type-level marker. It is
the owned-or-borrowed return type of a getter, and it is here because it is the one type in this group
you write on purpose.

## The ideas behind them

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — where the zero-sized
  provider structs that carry these markers come from.
- [Extensible records](/docs/concepts/extensible-records) — the record shape the product spine
  describes.
- [Extensible variants](/docs/concepts/extensible-variants) — the variant shape the sum spine
  describes.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
