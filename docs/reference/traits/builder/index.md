---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Record builders

Assembling a struct one field at a time, with field presence tracked in the type.

## Overview

These traits build a record incrementally, where the fields come from several independent places and
the result must still be checked at compile time. The build starts from an empty *partial* value and
adds fields one by one, and each field's presence lives in the type — so finalizing an incomplete
record is a missing impl rather than a runtime panic. The impls come from
[`#[derive(BuildField)]`](../../derives/derive_build_field.md), which also generates the partial
companion type.

The family divides into entry points, the update primitive and its two directions, and the finalize
step.

The **entry points** obtain a partial value to build into. [`HasBuilder`](has_builder.md) produces an
empty one, and [`IntoBuilder`](into_builder.md) turns a complete value into a full one to redistribute.

The **field operations** move one field between states. [`UpdateField`](update_field.md) is the
primitive the derive writes; [`BuildField`](build_field.md) sets an absent field and
[`TakeField`](take_field.md) removes a present one, each a blanket impl over the primitive in one
direction.

The **finalize step** ends a build. [`FinalizeBuild`](finalize_build.md) turns a fully-present partial
value back into the concrete struct, and exists only at that configuration.
[`PartialData`](partial_data.md) names the destination type at every configuration, which is how
generic builder code knows what it is building before the build is complete.

## The ideas behind them

- [Extensible records](/docs/concepts/extensible-records) — partial records and the extensible builder
  pattern.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
