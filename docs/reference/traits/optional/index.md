---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Optional and defaulted fields

Relaxing the strict record builder, so a field can be set in any order or left unset.

## Overview

The core [record builder](../builder/index.md) is deliberately strict: a field is set once, and a
partial record becomes its struct only when every field is present. That strictness catches a missing
field at compile time, and it is too rigid for a record whose fields arrive unpredictably or have
sensible defaults. These traits, from `cgp-field-extra`, relax it while reusing the same machinery
underneath. Nothing here is in the prelude. Import each from `cgp::extra::field::impls`.

The layer divides into entry points, the setter, the two endings, and the transform markers behind
them.

The **entry points** produce an all-optional builder, whose every field is an `Option`.
[`HasOptionalBuilder`](has_optional_builder.md) starts one from nothing, and [`ToOptional`](to_optional.md)
converts an existing builder into one.

[`SetOptional`](set_optional.md) is the **setter**, which sets a field as many times as you like,
because an optional field's marker does not change when it is written.

The **two endings** are chosen at the finalize call rather than when the builder is made.
[`FinalizeOptional`](finalize_optional.md) requires every field and reports the first missing one, while
[`CanFinalizeWithDefault`](can_finalize_with_default.md) fills a missing field from `Default`.
[`CanBuildWithDefault`](can_build_with_default.md) chains a merge and a defaulted finalize into one call.

The **transform markers** carry the per-field conversions the endings run.
[`TransformMapDefault`](transform_map_default.md) makes every field present, and
[`TransformOptional`](transform_optional.md) makes every field optional.

## The ideas behind them

- [Extensible records](/docs/concepts/extensible-records): partial records, and where relaxing presence
  fits.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
