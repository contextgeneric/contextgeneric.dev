---
title: 'TransformMapFields — convert every field'
sidebar_label: 'TransformMapFields'
sidebar_position: 4
description: 'Apply one per-field transform across a whole partial record, re-marking every field to a target marker in one call; the optional layer is built on it.'
---

# `TransformMapFields`

Applying one per-field transform across a whole [partial record](/docs/reference/glossary#partial-record).

:::info

### Generated machinery

**You are not expected to call `transform_map_fields` for CGP's own conversions.** It is the
recursion the optional-field layer is built from, and
[`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) and
[`ToOptional`](../optional/to_optional.md) are what you call. This page explains the walk, which
makes those two behave so symmetrically. The case for naming it is applying a transform of your own,
or writing an operation that re-marks a whole record.

:::

## Overview

[`TransformMap`](./transform_map.md) converts *one* field from one marker's storage to another's.
`TransformMapFields` lifts that conversion across an entire partial record, so every field is re-marked
in one call.

**This is the motion the optional-field layer is made of.**
[`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) re-marks every field to
`IsPresent` and then calls the ordinary [`finalize_build`](../builder/finalize_build.md), and
[`ToOptional`](../optional/to_optional.md) re-marks every field to `IsOptional`.

## Definition

`TransformMapFields` is parameterized by a transform marker and a target marker, and carries one method:

```rust
pub trait TransformMapFields<Transform, TargetMap> {
    type Output;

    fn transform_map_fields(self) -> Self::Output;
}
```

`Self` is the partial value. `Transform` is the marker carrying the per-field functions, written as
[`TransformMap`](./transform_map.md) impls, and `TargetMap` is the [`MapType`](./map_type.md) marker
every field should end in. `Output` is the partial type with every field re-marked accordingly, and
`transform_map_fields` consumes the partial value and returns it.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::TransformMapFields;
```

It is implemented for any [`PartialData`](../builder/partial_data.md) whose fields the transform can convert, so
there is nothing to implement. What you supply is the `Transform` marker, by writing
[`TransformMap`](./transform_map.md) impls for it.

**The method does not take type arguments, so something must fix `Transform` and `TargetMap`.**
Inside generic code a `where` clause does, and the method is called bare as
`builder.transform_map_fields()`. At a concrete call site, name both on the trait, as
`TransformMapFields::<Double, IsPresent>::transform_map_fields(builder)`; a bare call there does not
compile, as [Common Mistakes](#common-mistakes) shows.

## Examples

A transform of your own, applied at a concrete call site and through a bound:

```rust
use cgp::prelude::*;
use cgp::core::field::traits::{TransformMap, TransformMapFields};

// Doubles every present number and keeps it present.
pub struct Double;

impl<T: core::ops::Add<Output = T> + Copy> TransformMap<IsPresent, IsPresent, T> for Double {
    fn transform_mapped(value: T) -> T {
        value + value
    }
}

#[derive(Debug, PartialEq, CgpData)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

// At a concrete site, name the transform and the target marker on the trait.
pub fn doubled(point: Point) -> Point {
    TransformMapFields::<Double, IsPresent>::transform_map_fields(point.into_builder())
        .finalize_build()
}

// In generic code, the bound fixes both, so the method is called bare.
pub fn double_all<Builder>(builder: Builder) -> Builder::Output
where
    Builder: TransformMapFields<Double, IsPresent>,
{
    builder.transform_map_fields()
}

pub fn demo() {
    assert_eq!(doubled(Point { x: 1, y: 2 }), Point { x: 2, y: 4 });

    let point = double_all(Point { x: 3, y: 4 }.into_builder()).finalize_build();
    assert_eq!(point, Point { x: 6, y: 8 });
}
```

At a concrete site nothing fixes the transform or the target marker, so `doubled` names both on the
trait. Inside `double_all` the `where` clause fixes them, so the method is called bare.
[`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) and
[`ToOptional`](../optional/to_optional.md) are this second shape with CGP's own markers.

## When to use it

**Bound on it when writing an operation that re-marks a whole record.** Extending the optional-field
layer requires exactly that.

- **Use [`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) or
  [`ToOptional`](../optional/to_optional.md)** if what you want is one of the two conversions CGP already ships.
- **Write a [`TransformMap`](./transform_map.md) marker and apply it here** for a conversion they do not
  cover.
- **Use [`UpdateField`](../builder/update_field.md)** when a *single* field changes state. This trait is the
  whole-record form, and reaching for it to change one field walks every other field for nothing.
- **Use [`MapFields`](./map_fields.md)** when you only need to *name* the re-marked shape as a type.
  That trait computes the type; this one converts the values.

## Under the hood

The trait's one impl applies to any [`PartialData`](../builder/partial_data.md) whose target has a
[`HasFields`](../shape/has_fields.md) shape, and hands the value to a private walk over that shape:

```rust
impl<ContextA, ContextB, Transform, TargetMap, Output> TransformMapFields<Transform, TargetMap>
    for ContextA
where
    ContextA: PartialData<Target = ContextB>,
    ContextB: HasFields,
    ContextB::Fields: TransformMapFieldsImpl<ContextA, Transform, TargetMap, Output = Output>,
{
    type Output = Output;

    fn transform_map_fields(self) -> Self::Output {
        ContextB::Fields::transform_map_fields(self)
    }
}
```

The walk's `Cons` impl converts one field with [`UpdateField`](../builder/update_field.md)
**twice**, after the rest of the list:

```rust
fn transform_map_fields(context_a: ContextA) -> Self::Output {
    let context_b = Tail::transform_map_fields(context_a);

    let (value_a, context_c) = context_b.update_field(PhantomData, ());
    let value_b = Transform::transform_mapped(value_a);
    let (_, context_d) = context_c.update_field(PhantomData, value_b);

    context_d
}
```

The first `update_field` takes the field out, re-marking it `IsNothing` and reporting as `SourceMap`
the marker it was in; `Transform::transform_mapped` converts the value from `SourceMap`'s storage to
`TargetMap`'s; the second writes it back under `TargetMap`. So the result type has every field
re-marked to `TargetMap`, with the values converted accordingly. Three things follow.

**The transform must have an impl for every source marker a field might currently be in**, or the
walk does not resolve, which is why a transform like
[`FillDefaults`](./transform_map.md#examples) needs three [`TransformMap`](./transform_map.md) impls
rather than one.

**The recursion is driven by the target's shape**, so it re-marks exactly the fields the concrete
struct declares, and a field the shape does not mention is not visited at all.

**It visits the fields last to first**, because each cell recurses into the rest of the list before
converting its own field. The order matters only to a transform with a side effect, such as one that
logs: on `struct Triple { a: u32, b: u32, c: u32 }` it sees `c`, then `b`, then `a`.

The intermediate `IsNothing` state is invisible from outside: it exists only between the two
`UpdateField` calls for one field, and no observable value is ever in a half-transformed
configuration.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::traits`.

**A missing [`TransformMap`](./transform_map.md) impl stops the whole walk.** The useful error names
the missing impl for one field's source marker and value type, after a misleading `UpdateField`
mismatch; [`TransformMap`](./transform_map.md#common-mistakes) shows both.

**A bare call at a concrete site does not compile.** Nothing fixes the transform or the target
marker, so `Point { x: 1 }.into_builder().transform_map_fields()` fails with:

```text
error[E0283]: type annotations needed for `__PartialPoint<_>`
...
   = note: required for `__PartialPoint<cgp::prelude::IsPresent>` to implement `TransformMapFields<_, _>`
```

Name both on the trait, as the [example](#examples)'s `doubled` does.

**`Output` is a partial type, not the finished struct.** Finalizing is still
[`FinalizeBuild`](../builder/finalize_build.md)'s job. This trait only guarantees the configuration that impl
requires.

**It walks every field.** For a single field's transition, [`UpdateField`](../builder/update_field.md) is the
right tool.

**It is [`MapFields`](./map_fields.md)'s value-level counterpart, and they are easy to confuse.** That
one computes a type; this one converts values and needs a transform to do it.

**It requires [`PartialData`](../builder/partial_data.md).** A plain struct is not a partial value; obtain one
with [`HasBuilder`](../builder/has_builder.md) or [`IntoBuilder`](../builder/into_builder.md) first.

## Related constructs

- [`TransformMap`](./transform_map.md): the per-field conversion this lifts.
- [`MapType`](./map_type.md): the markers naming each state.
- [`UpdateField`](../builder/update_field.md): the primitive the walk calls twice per field.
- [`PartialData`](../builder/partial_data.md): what a partial value is, and the bound this requires.
- [`FinalizeBuild`](../builder/finalize_build.md): what an all-`IsPresent` result is accepted by.
- [`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) and [`ToOptional`](../optional/to_optional.md): the
  two operations built directly on this.
- [`MapFields`](./map_fields.md): the type-level counterpart.
- [`HasFields`](../shape/has_fields.md): the shape the walk follows.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and how their states change.

## Source

- [`transform_map.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/transform_map.rs):
  `TransformMapFields` and `TransformMap`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
