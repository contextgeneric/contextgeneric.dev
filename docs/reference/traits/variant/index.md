---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Extensible variants

Constructing an enum from one named variant, and taking one apart variant by variant.

## Overview

These traits are the enum counterpart of the [record builder](../builder/index.md): where a record is
a product of named fields, an enum is a sum of named variants, and the same name-driven, decoupled
treatment applies. They let generic code build and match an enum it cannot name, with exhaustiveness
proven in the type rather than by a wildcard arm. The impls come from
[`#[derive(ExtractField)]`](../../derives/derive_extract_field.md) and
[`#[derive(FromVariant)]`](../../derives/derive_from_variant.md).

The family splits into deconstruction and construction.

**Deconstruction** takes an enum apart. [`ExtractField`](extract_field.md) pulls one variant out of an
*extractor* or hands back a narrowed remainder, and the three accessors obtain that extractor in one
ownership mode each: [`HasExtractor`](has_extractor.md) consumes the value,
[`HasExtractorRef`](has_extractor_ref.md) borrows it, and [`HasExtractorMut`](has_extractor_mut.md)
borrows it mutably. [`FinalizeExtract`](finalize_extract.md) discharges the exhausted remainder with no
wildcard, and [`FinalizeExtractResult`](finalize_extract_result.md) is the form that closes a chain from
the `Result` a step returns.

**Construction** is the smaller half. [`FromVariant`](from_variant.md) builds an enum from a single
variant chosen by its type-level name, which lets code work in a narrow local enum and widen the
result.

## The ideas behind them

- [Extensible variants](/docs/concepts/extensible-variants): partial variants, the exhaustiveness
  argument, and the extensible visitor pattern.
- [Dispatching](/docs/concepts/dispatching): routing a variant to the implementation that handles it.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
