---
sidebar_label: 'TransformOptional'
---

# `TransformOptional`

The transform marker that makes every field optional.

:::info

### Generated machinery

**You are not expected to name `TransformOptional` directly.** It is the
marker [`ToOptional`](./to_optional.md) and [`HasOptionalBuilder`](./has_optional_builder.md) drive, and
calling one of those is what you write. This page explains the conversion behind them, and why it needs
less of a field's type than its defaulting counterpart does. The one case for naming it is writing a capability of your own that drives the optional conversion.

:::

## What it's for

[`ToOptional`](./to_optional.md) re-marks every field of a partial record to `IsOptional`, so it can then
be set freely and finalized either way. `TransformOptional` is the marker carrying the per-field
conversions that make that possible.

It is a zero-sized type implementing [`TransformMap`](./transform_map.md), and reading its impls is the
shortest description of what the optional conversion does:

| a field currently | becomes |
|---|---|
| `IsPresent` | `Some(value)` |
| `IsNothing` | `None` |

Both target `IsOptional`, which is the configuration [`SetOptional`](./set_optional.md) requires and the
one an optional builder lives at.

**It is the exact mirror of [`TransformMapDefault`](./transform_map_default.md)**, which targets
`IsPresent` instead — same walk, different destination. That symmetry is why the defaulted and optional
workflows behave so alike.

**You name it only when extending the layer.** Using the optional workflow means calling
[`optional_builder()`](./has_optional_builder.md) or [`to_optional()`](./to_optional.md), with no marker
in sight.

## Using it

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::TransformOptional;
```

There is nothing to call. The marker is named in a
[`TransformMapFields`](./transform_map_fields.md) bound, which is where a capability says which
conversion it drives:

```rust
Builder: TransformMapFields<TransformOptional, IsOptional>
```

Unlike its defaulting counterpart, **it requires nothing of the field types** — wrapping a value in
`Some` and producing `None` need no `Default` and no other bound, which is why the optional path applies
to records the defaulting path does not.

## Examples

You meet it in the bound behind [`ToOptional`](./to_optional.md):

```rust
use cgp::core::field::traits::TransformMapFields;
use cgp::extra::field::impls::TransformOptional;
use cgp::prelude::*;

// conceptually:
//   impl<Builder> ToOptional for Builder
//   where Builder: TransformMapFields<TransformOptional, IsOptional>
//   {
//       fn to_optional(self) -> Self::Output {
//           self.transform_map_fields()
//       }
//   }
```

Swap the marker for [`TransformMapDefault`](./transform_map_default.md) and the target for `IsPresent`,
add a [`finalize_build`](./finalize_build.md), and you have
[`CanFinalizeWithDefault`](./can_finalize_with_default.md). The two capabilities differ by exactly that
much.

Writing a marker of your own follows the same shape; the
[`TransformMap`](./transform_map.md#examples) page shows one with three impls.

## When to reach for it, and when not

**Name it only when writing a capability that drives the optional conversion.**

- **[`ToOptional`](./to_optional.md)** to relax an existing partial value.
- **[`HasOptionalBuilder`](./has_optional_builder.md)** to start an all-optional builder, which is
  `builder().to_optional()`.
- **[`TransformMapDefault`](./transform_map_default.md)** when the destination is `IsPresent` rather than
  `IsOptional`.
- **Write your own [`TransformMap`](./transform_map.md) marker** for a conversion neither covers.

## Under the hood

:::note

### Advanced

This section shows why it needs fewer impls than its counterpart.

:::

The marker is zero-sized and carries [`TransformMap`](./transform_map.md) impls distinguished by their
source marker, each targeting `IsOptional`:

```rust
// IsPresent -> IsOptional: wrap
//   fn transform_mapped(value: T) -> Option<T> { Some(value) }
//
// IsNothing -> IsOptional: nothing to wrap
//   fn transform_mapped(_value: ()) -> Option<T> { None }
```

Note what is absent: **no `Default` bound anywhere.** Its counterpart
[`TransformMapDefault`](./transform_map_default.md) needs one on two of its three impls, because filling
an absent field means producing a value from nothing. Producing `None` does not, so this conversion
applies to every field type.

[`TransformMapFields`](./transform_map_fields.md#under-the-hood) is what applies it, visiting each field
of the target's [`HasFields`](./has_fields.md) shape and using
[`UpdateField`](./update_field.md) twice per field — once to take the value out and learn its marker,
once to write it back under `IsOptional`.

The result is a partial value at a configuration the core builder never reaches on its own, which is what
makes [`SetOptional`](./set_optional.md) resolve and the strict
[`FinalizeBuild`](./finalize_build.md) not.

## Gotchas

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**It always targets `IsOptional`.** Reaching `IsPresent` is
[`TransformMapDefault`](./transform_map_default.md)'s job.

**It is a marker, not a capability.** There is no method and nothing to wire — it is named in a bound.

**It requires nothing of the field types**, which is the one place the two markers genuinely differ in
what they can be applied to.

**Converting gives up the compile-time completeness check**, since every field becomes optional whether
it was set or not.

**A field whose type is already `Option<T>` becomes `Option<Option<T>>`.** The outer layer is the
builder's presence tracking, and the two are easy to conflate in an error.

## Related constructs

- [`ToOptional`](./to_optional.md) — the conversion built directly on it.
- [`HasOptionalBuilder`](./has_optional_builder.md) — the entry point built on that.
- [`TransformMapDefault`](./transform_map_default.md) — the counterpart marker, targeting `IsPresent`.
- [`TransformMap`](./transform_map.md) — the trait it implements.
- [`TransformMapFields`](./transform_map_fields.md) — the walk that applies it.
- [`SetOptional`](./set_optional.md) — what the resulting configuration makes available.
- [`MapType`](./map_type.md) — the `IsOptional` marker it targets.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records and how their states change.

## Source

- [`to_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/to_optional.rs)
  — `TransformOptional`, `ToOptional`, and `HasOptionalBuilder`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
