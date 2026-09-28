---
title: 'PartialData — the type a builder becomes'
sidebar_label: 'PartialData'
sidebar_position: 7
description: 'Name the concrete struct a partial record will become, at every configuration, so generic code can mention the destination mid-build.'
---

# `PartialData`

Naming the concrete type a partial value is on its way to becoming.

:::info

### Generated machinery

**You are not expected to implement `PartialData`.**
[`#[derive(BuildField)]`](../../derives/derive_build_field.md) and
[`#[derive(ExtractField)]`](../../derives/derive_extract_field.md) emit it for every companion type they
generate. What you meet is the projection `Self::Target` in another trait's signature; this page explains
where that projection comes from. The one case for bounding on it is generic code that needs the destination type before the value is complete.

:::

## Overview

A [partial record](/docs/reference/glossary#partial-record) is a companion type (`__PartialPerson<IsNothing, IsPresent>`), and generic code holding
one often needs to know what it will *become* before it is complete: to name the return type of a
routine, to state a bound, to decide what to do next. `PartialData` answers that: its single `Target`
associated type is the concrete struct or enum the partial value corresponds to. **It is implemented for
every configuration**, complete or not, and that coverage is the point: the destination is knowable from
the first `builder()` call, long before any field is set.

That is the division of labour with [`FinalizeBuild`](./finalize_build.md), which is implemented **only**
at the all-present configuration. One says where you are going; the other says you have arrived.

## Definition

`PartialData` is a single associated type and nothing else:

```rust
pub trait PartialData {
    type Target;
}
```

`Self` is a partial companion type, a record builder or an extraction remainder. `Target` is the concrete
struct or enum it belongs to. There is no method, because naming a type is not an operation; the trait
exists so that generic code can project `Self::Target` while holding a value of unknown completeness.
[`FinalizeBuild`](./finalize_build.md) and [`FinalizeOptional`](../optional/finalize_optional.md) both
read the destination through this projection rather than declaring one of their own.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

You bound on it when a signature must mention the destination while holding a partial value, as
`Partial::Target` in the `target_name` function of the [example](#examples).

The impls come from [`#[derive(BuildField)]`](../../derives/derive_build_field.md) for a record and
[`#[derive(ExtractField)]`](../../derives/derive_extract_field.md) for an enum. **Both** partial families
implement it, which is worth knowing because it is the one trait the builder and extractor sides share
directly.

## Examples

A generic function naming the destination of a partial value, at two configurations:

```rust
use cgp::prelude::*;

#[derive(Debug, PartialEq, BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

pub fn target_name<Partial>(_partial: &Partial) -> &'static str
where
    Partial: PartialData,
{
    core::any::type_name::<Partial::Target>()
}

pub fn demo() {
    let empty = Person::builder();
    assert!(target_name(&empty).ends_with("Person"));

    let half = empty.build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());
    assert!(target_name(&half).ends_with("Person"));
}
```

`Partial::Target` is `Person` for the empty builder and the half-built one alike, since the impl
covers every configuration.

## When to use it

**Bound on it when generic code needs the destination type mid-build**, and reach for the more specific
traits otherwise.

- **[`FinalizeBuild`](./finalize_build.md)** when the value is complete and should become the struct. It
  supertraits this, so bounding on both is redundant.
- **[`HasBuilder`](./has_builder.md)** when you have the concrete type and want a builder. That is the
  opposite direction: `Person::Builder` from `Person`, rather than `Target` from a partial.
- **`PartialData`** when the code holds a partial value of unknown completeness and must name what it
  belongs to: a signature, a `where` clause, an error message.
- **[`TransformMapFields`](../type-level/transform_map_fields.md)** requires it, which is the other place it shows up
  in a bound rather than as a projection.

## Under the hood

The derive emits one impl covering every configuration at once, by leaving the markers generic.
`cargo cgp expand` on a two-field `Person` shows:

```rust
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialPerson<__F0__, __F1__> {
    type Target = Person;
}
```

Nothing about the markers is constrained, so the destination stays available at every step of a
build. Contrast [`FinalizeBuild`](./finalize_build.md), whose impl fixes every marker to
`IsPresent`. Splitting the two lets generic builder code name a destination while still being unable
to finalize early. The enum side implements it the same way on its extraction companions, with
`Target` naming the original enum.

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

- [`FinalizeBuild`](./finalize_build.md): the subtrait that actually produces the target.
- [`HasBuilder`](./has_builder.md) and [`IntoBuilder`](./into_builder.md): the opposite direction, from
  a concrete type to a partial one.
- [`UpdateField`](./update_field.md): what moves a partial value between configurations.
- [`FinalizeOptional`](../optional/finalize_optional.md): the optional layer's finalize, which projects `Target`
  the same way.
- [`TransformMapFields`](../type-level/transform_map_fields.md): requires this bound.
- [`ExtractField`](../variant/extract_field.md): the enum family whose companions also implement it.
- [`#[derive(BuildField)]`](../../derives/derive_build_field.md) and
  [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md): generate the impls.
- [`MapType`](../type-level/map_type.md): the markers a configuration is written in.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and the extensible builder
  pattern.

## Source

- [`partial_data.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/partial_data.rs):
  `PartialData`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
