---
description: 'The dispatch combinators: match an enum one variant at a time, and build a record one field at a time, with a handler per variant or field.'
sidebar_label: 'Overview'
sidebar_position: 0
---

# Dispatch combinators

The providers that route an extensible-data value (an enum or a record) to per-variant or per-field
handlers: the matchers take an enum apart, and the builders assemble a record.

## Overview

The dispatch combinators handle an enum or a record generically, where the logic for each variant
or field lives in its own provider. A hand-written `match` names every variant in one place. These
combinators drive the extractor and builder traits instead, trying one variant or filling one field
at a time on a [**context**](/docs/reference/glossary#context), the type the implementation runs
against, and they prove at compile time that every variant is matched or every field is set. Like
every CGP provider, each is zero-sized: its type parameters ride in `PhantomData`.

The two halves implement different members of the handler family. The matchers and their adapters
implement [`Computer`](../../components/handler/computer.md) and
[`AsyncComputer`](../../components/handler/async_computer.md) only, so a fallible slot takes them
through a [promotion](../handler/index.md). The builders implement `Computer`,
[`TryComputer`](../../components/handler/try_computer.md), and
[`Handler`](../../components/handler/handler.md), propagating a step's error in the fallible forms.

The value matchers `MatchWithValueHandlers`, `MatchWithValueHandlersRef`,
`MatchWithValueHandlersMut`, and their `MatchFirstWith…` counterparts are in the prelude. Every
other name here is imported from `cgp::extra::dispatch`.

## The matchers

A **matcher** consumes an enum. It tries each variant in turn, hands the matched payload to a
handler, and needs no wildcard arm, because a variant with no handler is a compile error:

- [`MatchWithHandlers`](match_with_handlers.md) runs a spelled-out list of per-variant handlers,
  over an owned or a borrowed input.
- [`MatchFirstWithHandlers`](match_first_with_handlers.md) does the same for an `(Input, Args)`
  input, passing the extra arguments to every handler.
- [`MatchWithValueHandlers`](match_with_value_handlers.md) and
  [`MatchWithFieldHandlers`](match_with_field_handlers.md) build that list from the enum's own
  variants, passing each payload bare or tagged with its variant name.

The list a matcher runs is a list of **adapters**, each trying one variant or group:

- [`ExtractFieldAndHandle`](extract_field_and_handle.md) extracts one variant and hands its payload,
  tagged, to an inner provider.
- [`HandleFieldValue`](handle_field_value.md) strips the tag, so the inner provider receives the
  bare payload.
- [`DowncastAndHandle`](downcast_and_handle.md) narrows the input to a smaller enum and hands a
  whole group of variants to one provider.

## The builders

A **builder** produces a record. It starts from an empty
[partial record](/docs/reference/glossary#partial-record), runs a step per field or group of
fields, and finalizes the record once every field is set:

- [`BuildWithHandlers`](build_with_handlers.md) runs a list of builder steps and finalizes.
- [`BuildAndSetField`](build_and_set_field.md) computes and sets one field.
- [`BuildAndMerge`](build_and_merge.md) builds a sub-record and copies its fields in.
- [`BuildAndMergeOutputs`](build_and_merge_outputs.md) takes a list of sub-record providers and adds
  the merge step to each.

Both halves are handler providers, so they compose with the
[handler combinators](../handler/index.md) and wire into a context with
[`delegate_components!`](../../macros/delegate_components.md).

## Related constructs

- [`#[cgp_auto_dispatch]`](../../macros/cgp_auto_dispatch.md) — generates a matcher-backed trait
  impl for an enum.
- [`ExtractField`](../../traits/variant/extract_field.md),
  [`HasExtractor`](../../traits/variant/has_extractor.md),
  [`FinalizeExtract`](../../traits/variant/finalize_extract.md) — the enum-deconstruction traits the
  matchers stand on.
- [`HasBuilder`](../../traits/builder/has_builder.md),
  [`BuildField`](../../traits/builder/build_field.md),
  [`FinalizeBuild`](../../traits/builder/finalize_build.md) — the record-assembly traits the
  builders stand on.
- [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md) — the derive that gives a type the shape
  these operate over.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to per-variant and
  per-field handlers.
- [Extensible variants](/docs/concepts/extensible-variants) and
  [Extensible records](/docs/concepts/extensible-records) — the data patterns the matchers and
  builders serve.

## Source

- The provider structs are in `cgp-dispatch` under
  [`providers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-dispatch/src/providers),
  and the prelude re-exports the value matchers from
  [`cgp-extra`](https://github.com/contextgeneric/cgp/blob/main/crates/main/cgp-extra/src/prelude.rs).

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
