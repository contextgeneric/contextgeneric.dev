---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Dispatch combinators

The providers that route an extensible-data value (a record or a variant) to per-field or per-variant
handlers, and assemble or finalize the result.

## Overview

The dispatch combinators handle an arbitrary record or enum generically, where the set of fields or
variants is not known at the call site and the handling logic for each one lives in a separate
provider. A hand-written `match` names every variant in one place; these combinators instead drive the
extractor and builder trait families to do the same work over a value whose shape is only known at the
type level, dispatching each field or variant to a handler chosen by type. They run on a **context**,
the type a capability runs against, and are all [`Computer`](../../components/handler/computer.md)-family
providers, several also [`Handler`](../../components/handler/handler.md) and
[`TryComputer`](../../components/handler/try_computer.md) providers. Like every CGP provider, each is zero-sized.

The combinators divide into two halves.

The **matcher** half consumes a sum type. It tries each variant in turn, hands the matched payload to a
handler, and proves the match exhaustive without a wildcard arm:

- [`MatchWithHandlers`](match_with_handlers.md) runs a spelled-out list of per-variant handlers over an
  owned input.
- [`MatchFirstWithHandlers`](match_first_with_handlers.md) does the same for the multi-argument calling
  convention, where the input carries extra arguments alongside the value.
- [`MatchWithValueHandlers`](match_with_value_handlers.md) and
  [`MatchWithFieldHandlers`](match_with_field_handlers.md) build that list automatically from the input
  type's own field list.

The per-variant list a matcher runs is normally a list of **adapters**, each trying one variant and
forwarding the payload:

- [`ExtractFieldAndHandle`](extract_field_and_handle.md) extracts one variant and hands the payload,
  still tagged, to an inner provider.
- [`HandleFieldValue`](handle_field_value.md) strips the tag so the inner provider receives the bare
  value.
- [`DowncastAndHandle`](downcast_and_handle.md) matches a whole group of variants at once.

The **builder** half produces a product type. It starts from an empty builder, runs a handler per field,
and finalizes the fully-populated record:

- [`BuildAndSetField`](build_and_set_field.md) computes and sets one field.
- [`BuildAndMerge`](build_and_merge.md) copies a whole record's worth of fields in at once.
- [`BuildWithHandlers`](build_with_handlers.md) is the entry point that runs a list of builder adapters
  and finalizes.
- [`BuildAndMergeOutputs`](build_and_merge_outputs.md) is the wrapper for a list of plain field-producing
  providers.

Both halves are handler providers, so they compose with the [handler combinators](../handler/index.md),
nest inside [`UseInputDelegate`](../handler/use_input_delegate.md), and can be wired into a context with
[`delegate_components!`](../../macros/delegate_components.md). The matcher loop they share is
`DispatchMatchers`, a type alias for [`PipeMonadic`](../monad/pipe_monadic.md) under the `OkMonadic`
monad; it is an implementation detail rather than a construct a user names, and it is explained under
the hood on [`MatchWithHandlers`](match_with_handlers.md).

## Related constructs

- [`#[cgp_auto_dispatch]`](../../macros/cgp_auto_dispatch.md) — generates a matcher-backed handler impl
  automatically.
- [`extract_field`](../../traits/variant/extract_field.md), [`has_extractor`](../../traits/variant/has_extractor.md),
  [`finalize_extract`](../../traits/variant/finalize_extract.md) — the enum-deconstruction traits the matchers
  stand on.
- [`has_builder`](../../traits/builder/has_builder.md), [`build_field`](../../traits/builder/build_field.md),
  [`finalize_build`](../../traits/builder/finalize_build.md) — the record-assembly traits the builders stand on.
- [`UseInputDelegate`](../handler/use_input_delegate.md) — the input dispatcher the convenience matchers
  use.
- [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md) — the derive that gives a type the shape these
  operate over.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to per-field and
  per-variant handlers.
- [Extensible records](/docs/concepts/extensible-records) and
  [Extensible variants](/docs/concepts/extensible-variants) — the data patterns the builders and matchers
  serve.

## Source

- The provider structs are in `cgp-dispatch` under
  [`providers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-dispatch/src/providers).
  The prelude re-exports the value-handler matchers; the rest are reached through `cgp::extra::dispatch`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
