---
sidebar_label: 'PartialData'
---

# `PartialData`

Naming the concrete type a partial value is on its way to becoming.

:::info

### Generated machinery

**You are not expected to implement `PartialData`.**
[`#[derive(BuildField)]`](../derives/derive_build_field.md) and
[`#[derive(ExtractField)]`](../derives/derive_extract_field.md) emit it for every companion type they
generate. What you meet is the projection `Self::Target` in another trait's signature; this page explains
where that projection comes from. The one case for bounding on it is generic code that needs the destination type before the value is complete.

:::

## Overview

A partial record is a companion type — `__PartialPerson<IsNothing, IsPresent>` — and generic code holding
one often needs to know what it will *become* before it is complete: to name the return type of a
routine, to state a bound, to decide what to do next. `PartialData` answers that:

```rust
pub trait PartialData {
    type Target;
}
```

`Target` is the concrete struct or enum the partial value corresponds to. **It is implemented for every
configuration**, complete or not, which is exactly what makes it useful: the destination is knowable from
the first `builder()` call, long before any field is set.

That is the division of labour with [`FinalizeBuild`](./finalize_build.md), which is implemented **only**
at the all-present configuration. One says where you are going; the other says you have arrived.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough. There is no method — naming a type is not
an operation.

You bound on it when a signature must mention the destination while holding a partial value:

```rust
fn describe<Partial>(partial: Partial) -> &'static str
where
    Partial: PartialData,
{
    core::any::type_name::<Partial::Target>()
}
```

The impls come from [`#[derive(BuildField)]`](../derives/derive_build_field.md) for a record and
[`#[derive(ExtractField)]`](../derives/derive_extract_field.md) for an enum — **both** partial families
implement it, which is worth knowing because it is the one trait the builder and extractor sides share
directly.

## Examples

Its most visible use is as [`FinalizeBuild`](./finalize_build.md)'s supertrait, which is how a finalize
knows what to return:

```rust
pub trait FinalizeBuild: PartialData {
    fn finalize_build(self) -> Self::Target;
}
```

`Self::Target` in that signature comes from here. Without it the finalize would need its own associated
type, and the destination would be unnameable until the value was complete.

The same appears in [`FinalizeOptional`](./finalize_optional.md), which returns
`Result<Self::Target, &'static str>` — again projecting the destination through this trait rather than
declaring one of its own.

## When to use it

**Bound on it when generic code needs the destination type mid-build**, and reach for the more specific
traits otherwise.

- **[`FinalizeBuild`](./finalize_build.md)** when the value is complete and should become the struct. It
  supertraits this, so bounding on both is redundant.
- **[`HasBuilder`](./has_builder.md)** when you have the concrete type and want a builder. That is the
  opposite direction: `Person::Builder` from `Person`, rather than `Target` from a partial.
- **`PartialData`** when the code holds a partial value of unknown completeness and must name what it
  belongs to — a signature, a `where` clause, an error message.
- **[`TransformMapFields`](./transform_map_fields.md)** requires it, which is the other place it shows up
  in a bound rather than as a projection.

## Under the hood

The derive emits one impl covering every configuration at once, by leaving the markers generic:

```rust
// conceptually, for the record companion:
//   impl<F0: MapType, F1: MapType> PartialData for __PartialPerson<F0, F1> {
//       type Target = Person;
//   }
```

Nothing about the markers is constrained, which is what makes the destination available at every step of
a build. Contrast [`FinalizeBuild`](./finalize_build.md), whose impl fixes every marker to `IsPresent`.

Splitting the two is what lets generic builder code be written against a destination it can name while
still being *unable* to finalize prematurely — the type is known, the conversion is not available. A
single trait carrying both would have to choose one or the other.

The enum side implements it identically on its extraction companions, with `Target` naming the original
enum, which is how [`HasExtractor`](./has_extractor.md)'s round trip knows what to rebuild.

## Common Mistakes

**It says nothing about completeness.** A `PartialData` bound is satisfied by an empty builder as readily
as by a full one. Requiring completeness is [`FinalizeBuild`](./finalize_build.md).

**Both families implement it.** A `PartialData` bound admits an extraction remainder as well as a record
builder, which is occasionally what you want and occasionally too permissive.

**[`FinalizeBuild`](./finalize_build.md) supertraits it**, so naming both in a bound is redundant.

**`Target` is the original type, not the partial one.** Reading the projection the other way round is the
usual confusion when meeting it in an error.

**It has no method.** Getting a value of `Target` is a finalize; this only names the type.

## Related constructs

- [`FinalizeBuild`](./finalize_build.md) — the subtrait that actually produces the target.
- [`HasBuilder`](./has_builder.md) and [`IntoBuilder`](./into_builder.md) — the opposite direction, from
  a concrete type to a partial one.
- [`UpdateField`](./update_field.md) — what moves a partial value between configurations.
- [`FinalizeOptional`](./finalize_optional.md) — the optional layer's finalize, which projects `Target`
  the same way.
- [`TransformMapFields`](./transform_map_fields.md) — requires this bound.
- [`ExtractField`](./extract_field.md) — the enum family whose companions also implement it.
- [`#[derive(BuildField)]`](../derives/derive_build_field.md) and
  [`#[derive(ExtractField)]`](../derives/derive_extract_field.md) — generate the impls.
- [`MapType`](./map_type.md) — the markers a configuration is written in.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records and the extensible builder
  pattern.

## Source

- [`partial_data.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/partial_data.rs)
  — `PartialData`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
