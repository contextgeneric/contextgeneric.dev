---
sidebar_label: 'TransformMapFields'
---

# `TransformMapFields`

Applying one per-field transform across a whole partial record.

:::info

### Generated machinery

**You are not expected to call `transform_map_fields` directly.** It is the
recursion the optional-field layer is built from, and
[`CanFinalizeWithDefault`](./can_finalize_with_default.md) and [`ToOptional`](./to_optional.md) are what
you call. This page explains the walk, which is what makes those two behave so symmetrically.

:::

## What it's for

[`TransformMap`](./transform_map.md) converts *one* field from one marker's storage to another's.
`TransformMapFields` lifts that conversion across an entire partial record, so every field is re-marked
in one call:

```rust
pub trait TransformMapFields<Transform, TargetMap> {
    type Output;

    fn transform_map_fields(self) -> Self::Output;
}
```

`Self` is the partial value, `Transform` is the marker carrying the per-field functions, and `TargetMap`
is the [`MapType`](./map_type.md) marker every field should end in. `Output` is the partial type with
every field re-marked accordingly.

**This is the motion the optional-field layer is made of.** Re-marking every field to `IsPresent` and
then calling the ordinary [`finalize_build`](./finalize_build.md) is exactly what
[`CanFinalizeWithDefault`](./can_finalize_with_default.md) does, and re-marking to `IsOptional` is what
[`ToOptional`](./to_optional.md) does.

## Using it

**It is not in the prelude.** Import it from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::TransformMapFields;
```

It is implemented for any [`PartialData`](./partial_data.md) whose fields the transform can convert, so
there is nothing to implement — what you supply is the `Transform` marker, by writing
[`TransformMap`](./transform_map.md) impls for it.

The two type parameters are usually inferred from the surrounding bound rather than written at a call
site, which is why the method reads as a bare `self.transform_map_fields()`.

## Examples

You reach it through the layer built on it far more often than directly. This is the whole body of
[`CanFinalizeWithDefault`](./can_finalize_with_default.md):

```rust
use cgp::core::field::traits::TransformMapFields;
use cgp::prelude::*;

impl<Builder, Output> CanFinalizeWithDefault for Builder
where
    Builder: TransformMapFields<TransformMapDefault, IsPresent>,
    Builder::Output: FinalizeBuild<Target = Output>,
{
    type Output = Output;

    fn finalize_with_default(self) -> Output {
        self.transform_map_fields().finalize_build()
    }
}
```

Read the two bounds together: the first says every field can be re-marked to `IsPresent` by
`TransformMapDefault`, and the second says the result is then finalizable. The strict presence check
still runs — it just always passes, because the transform guaranteed presence first.

Applying a transform of your own follows the same shape, with your marker in place of
`TransformMapDefault`.

## When to reach for it, and when not

**Bound on it when writing a capability that re-marks a whole record**, which is what extending the
optional-field layer means.

- **Use [`CanFinalizeWithDefault`](./can_finalize_with_default.md) or
  [`ToOptional`](./to_optional.md)** if what you want is one of the two conversions CGP already ships.
- **Write a [`TransformMap`](./transform_map.md) marker and apply it here** for a conversion they do not
  cover.
- **Use [`UpdateField`](./update_field.md)** when a *single* field changes state. This trait is the
  whole-record form, and reaching for it to change one field walks every other field for nothing.
- **Use [`MapFields`](./map_fields.md)** when you only need to *name* the re-marked shape as a type.
  That trait computes the type; this one converts the values.

## Under the hood

:::note

### Advanced

This section shows the recursion and the two limits it explains.

:::

`TransformMapFields` walks the target's [`HasFields`](./has_fields.md) product one entry at a time. For
each `Field<Tag, Value>` it uses [`UpdateField`](./update_field.md) **twice**: it takes the field out —
replacing its marker with `IsNothing` and reading what the marker *was* — applies
`Transform::transform_mapped` to convert the value into the `TargetMap` wrapping, then writes it back
under `TargetMap`.

So the result type has every field re-marked to `TargetMap`, with the values converted accordingly. Two
things follow from that construction, and both explain limits you might otherwise trip over.

**The transform must have an impl for every source marker a field might currently be in**, or the walk
does not resolve — which is why a transform like `FillDefaults` needs three
[`TransformMap`](./transform_map.md) impls rather than one.

**The recursion is driven by the target's shape**, so it re-marks exactly the fields the concrete struct
declares, in declaration order, and a field the shape does not mention is not visited at all.

The intermediate `IsNothing` state is invisible from outside: it exists only between the two
`UpdateField` calls for one field, and no observable value is ever in a half-transformed configuration.

## Gotchas

**It is not in the prelude.** Import from `cgp::core::field::traits`.

**A missing [`TransformMap`](./transform_map.md) impl stops the whole walk.** The error names the missing
impl for one field's source marker, not the record, so it reads as being about a marker pair rather than
about the transform being incomplete.

**`Output` is a partial type, not the finished struct.** Finalizing is still
[`FinalizeBuild`](./finalize_build.md)'s job — this trait only guarantees the configuration that impl
requires.

**It walks every field.** For a single field's transition, [`UpdateField`](./update_field.md) is the
right tool.

**It is [`MapFields`](./map_fields.md)'s value-level counterpart, and they are easy to confuse.** That
one computes a type; this one converts values and needs a transform to do it.

**It requires [`PartialData`](./partial_data.md).** A plain struct is not a partial value; obtain one
with [`HasBuilder`](./has_builder.md) or [`IntoBuilder`](./into_builder.md) first.

## Related constructs

- [`TransformMap`](./transform_map.md) — the per-field conversion this lifts.
- [`MapType`](./map_type.md) — the markers naming each state.
- [`UpdateField`](./update_field.md) — the primitive the walk calls twice per field.
- [`PartialData`](./partial_data.md) — what a partial value is, and the bound this requires.
- [`FinalizeBuild`](./finalize_build.md) — what an all-`IsPresent` result is accepted by.
- [`CanFinalizeWithDefault`](./can_finalize_with_default.md) and [`ToOptional`](./to_optional.md) — the
  two capabilities built directly on this.
- [`MapFields`](./map_fields.md) — the type-level counterpart.
- [`HasFields`](./has_fields.md) — the shape the walk follows.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records and how their states change.

## Source

- [`transform_map.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/transform_map.rs)
  — `TransformMapFields` and `TransformMap`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
