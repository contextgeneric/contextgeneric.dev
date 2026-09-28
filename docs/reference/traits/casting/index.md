---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'The traits that convert between two data types by their shared variant or field names: widening and narrowing enums, and merging records into a builder.'
---

# Structural casts

Converting between two data types by their shared fields or variants, without a hand-written
conversion.

## Overview

Two types that share a subset of named fields or variants can convert into one another generically,
without a hand-written `From` or `TryFrom`. Because CGP represents each record as a product of named
fields and each enum as a sum of named variants, a conversion becomes a matter of routing each named
entry to the target's slot of the same name. Each side derives the part of the extensible-data
machinery its role needs, and the names are matched at the type level. None of these traits is in
the prelude. Import each from `cgp::core::field::impls`.

These traits cover the directions that routing can take.

For **enums**, [`CanUpcast`](can_upcast.md) widens a value into an enum whose variants are a superset,
which always succeeds, and [`CanDowncast`](can_downcast.md) narrows into a smaller enum, which can fail
and hands back a remainder. [`CanDowncastFields`](can_downcast_fields.md) continues a narrowing chain on
that remainder against a further candidate.

For **records**, [`CanBuildFrom`](can_build_from.md) moves every field of another record into a
builder, so several sources can be merged into one target before it is finalized.

## The ideas behind them

- [Extensible variants](/docs/concepts/extensible-variants): upcasting and downcasting between enums.
- [Extensible records](/docs/concepts/extensible-records): merging records through a builder.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
