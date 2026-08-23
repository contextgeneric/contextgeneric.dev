---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Field structure

A type's whole shape as a single type, and the conversions that move a value through it.

## Overview

Where [field access](../field-access/index.md) reads one named field, these traits describe a type's
*whole shape* at once — every field, its name, and its type — as a single associated type. That is what
lets generic code walk a struct or an enum it cannot name: a serializer, a validator, a merge, a
dispatcher. The impls come from [`#[derive(HasFields)]`](../../derives/derive_has_fields.md).

Two traits name the shape, and three convert a value in and out of it.

The **shape traits** describe the type. [`HasFields`](has_fields.md) names the owned shape — a
[`Product!`](../../macros/product.md) for a struct, a [`Sum!`](../../macros/sum.md) for an enum — and
[`HasFieldsRef`](has_fields_ref.md) names the borrowed shape, with every value a reference.

The **conversions** move a concrete value through the shape. [`ToFields`](to_fields.md) takes a value
apart into its shape and [`FromFields`](from_fields.md) rebuilds it, and the two round-trip through the
identical type so nothing can fail. [`ToFieldsRef`](to_fields_ref.md) is the borrowing form, for code
that reads a shape without consuming the value.

## The ideas behind them

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum half.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
