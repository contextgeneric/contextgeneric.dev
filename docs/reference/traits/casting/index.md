---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Structural casts

Converting between two data types by their shared fields or variants, with no hand-written conversion.

## Overview

Two types that share a subset of named fields or variants can convert into one another generically,
without a hand-written `From` or `TryFrom`. Because CGP represents each record as a product of named
fields and each enum as a sum of named variants, a conversion becomes a matter of routing each named
entry to the target's slot of the same name. Both types must derive the extensible-data machinery, and
the names are matched at the type level. None of these traits is in the prelude. Import each from
`cgp::core::field::impls`.

These traits cover the directions that routing can take.

For **enums**, [`CanUpcast`](can_upcast.md) widens a value into an enum whose variants are a superset,
which always succeeds, and [`CanDowncast`](can_downcast.md) narrows into a smaller enum, which can fail
and hands back a remainder. [`CanDowncastFields`](can_downcast_fields.md) continues a narrowing chain on
that remainder against a further candidate.

For **records**, [`CanBuildFrom`](can_build_from.md) fills a builder with every field it shares with
another record, so several sources can be merged into one target before it is finalized.

## The ideas behind them

- [Extensible variants](/docs/concepts/extensible-variants): upcasting and downcasting between enums.
- [Extensible records](/docs/concepts/extensible-records): merging records through a builder.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
