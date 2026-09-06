---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Type-level list algebra

The per-field state markers and list operations the extensible-data families are built from.

## Overview

These are the least user-facing traits in the reference. They are the type-level plumbing beneath the
[builders](../builder/index.md), [extractors](../variant/index.md), and [casts](../casting/index.md):
the markers that record what state one field is in, and the operations that grow and reshape a type's
field list. You meet them in an error message from code that walks a shape far more often than in code
you wrote, and this group exists so that such a message is legible. Most are not in the prelude.

The group divides into the state markers and the list operations.

The **state markers** name how one field of a partial type is stored. [`MapType`](map_type.md) is the
marker trait, and its `IsPresent`, `IsNothing`, and `IsVoid` markers are what a builder or extractor
error prints. [`MapTypeRef`](map_type_ref.md) does the same for a borrowed view, adding a lifetime.
[`TransformMap`](transform_map.md) carries the function that converts one field between markers, and
[`TransformMapFields`](transform_map_fields.md) lifts that conversion across a whole record.

The **list operations** compute new field lists from old ones. [`AppendProduct`](append_product.md) adds
one entry to a product and [`ConcatProduct`](concat_product.md) splices two together, while
[`MapFields`](map_fields.md) rewrites every entry of a product or a sum through one marker.

## The ideas behind them

- [Extensible records](/docs/concepts/extensible-records): presence tracking on a partial record.
- [Extensible variants](/docs/concepts/extensible-variants): possibility tracking on a partial variant.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
