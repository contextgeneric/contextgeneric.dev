---
sidebar_label: 'CanBuildWithDefault'
sidebar_position: 6
---

# `CanBuildWithDefault`

Widening one record into another in a single call.

## Overview

Turning a narrow record into a wider one whose extra fields have sensible defaults is a three-step motion:
start a builder, copy the shared fields, default the rest. `CanBuildWithDefault` is that motion as one
call.

`Self` is the target record and `Source` the narrower one. **It is the field-level counterpart of an
[upcast](../casting/can_upcast.md)**: turning a `Point2d` into a `Point3d` whose extra `z` is `0`, naming no field
explicitly.

It chains [`builder()`](../builder/has_builder.md), [`build_from`](../casting/can_build_from.md), and
[`finalize_with_default`](./can_finalize_with_default.md), and it is worth reaching for precisely because
those three lines together read as plumbing.

## Definition

`CanBuildWithDefault<Source>` is keyed by the source record and builds the target directly:

```rust
pub trait CanBuildWithDefault<Source> {
    fn build_with_default(source: Source) -> Self;
}
```

`Source` is the narrower record and `Self` is the target. `build_with_default` is an associated function
with no receiver, and it returns a fully built `Self`. The trait declares no associated type and no
supertrait; it is a blanket impl chaining a builder, a merge, and a defaulted finalize, shown in
[*Under the hood*](#under-the-hood). Two requirements follow from that chain: the source needs
[`HasFields`](../shape/has_fields.md), because the merge walks its field list, and every field the source
does not supply needs `Default`, because the finalize fills it. It is not in the prelude; import it from
`cgp-field-extra`.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::CanBuildWithDefault;
```

It is an **associated function**, called on the target: `Point3d::build_with_default(point_2d)`.

## Examples

Widening a record with no field named anywhere:

```rust
use cgp::extra::field::impls::CanBuildWithDefault;
use cgp::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, CgpData)]
struct Point2d { x: u64, y: u64 }

#[derive(Debug, Clone, Eq, PartialEq, CgpData)]
struct Point3d { x: u64, y: u64, z: u64 }

let point_3d = Point3d::build_with_default(Point2d { x: 1, y: 2 });

assert_eq!(point_3d, Point3d { x: 1, y: 2, z: 0 });   // z defaulted
```

Neither struct knows about the other. They share field *names*, matched at the type level, and `z` is
filled because `u64: Default`.

Written out, the same thing is three calls:

```rust
let point_3d: Point3d = Point3d::builder()
    .build_from(Point2d { x: 1, y: 2 })
    .finalize_with_default();
```

## When to use it

**Reach for it for a one-call widening from a narrower record**, and reach for the pieces when anything
in between needs to happen.

- **[`CanBuildFrom`](../casting/can_build_from.md) plus an explicit finalize** when some fields must be set by
  hand as well as copied. This trait offers no place to insert a `build_field`.
- **[`CanFinalizeWithDefault`](./can_finalize_with_default.md)** when there is no source to merge from.
- **[`CanUpcast`](../casting/can_upcast.md)** for the enum analogue: widening a variant set rather than a field
  set.
- **A plain `From` impl** when both types are yours and the conversion is one you would write once. A
  hand-written `From` is clearer, requires nothing of either type, and lets you choose values other than
  `Default`.

## Under the hood

The impl chains the three steps and constrains each with a bound:

```rust
// conceptually:
//   Self: HasBuilder,
//   Self::Builder: CanBuildFrom<Source>,
//   <Self::Builder as CanBuildFrom<Source>>::Output: CanFinalizeWithDefault<Output = Self>
//
//   fn build_with_default(source: Source) -> Self {
//       Self::builder().build_from(source).finalize_with_default()
//   }
```

Each bound is where one of the requirements comes from: [`CanBuildFrom`](../casting/can_build_from.md) brings the
[`HasFields`](../shape/has_fields.md) obligation on the source, and
[`CanFinalizeWithDefault`](./can_finalize_with_default.md) brings the `Default` obligation on the
remaining fields, through a [`TransformMapFields`](../type-level/transform_map_fields.md) walk carrying
[`TransformMapDefault`](./transform_map_default.md).

So an unsatisfied bound here is always really an unsatisfied bound one layer down, which is worth knowing
because the error names that layer rather than this trait.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**The source needs [`HasFields`](../shape/has_fields.md), not just a builder.** Deriving only
[`BuildField`](../../derives/derive_build_field.md) on both looks symmetric and fails, the same trap
[`CanBuildFrom`](../casting/can_build_from.md) carries.

**Every field the source does not supply needs `Default`.** The error names the missing
[`TransformMap`](../type-level/transform_map.md) impl rather than the field.

**It is an associated function on the target.** `Point3d::build_with_default(source)`, not a method on
the source.

**There is no place to set a field explicitly.** If one field needs a real value, use the three-call form.

**A field the target lacks is silently dropped**, since the merge matches on the target's slots.

## Related constructs

- [`CanBuildFrom`](../casting/can_build_from.md): the merge step, and where the source's requirements come from.
- [`CanFinalizeWithDefault`](./can_finalize_with_default.md): the defaulting finalize step.
- [`HasBuilder`](../builder/has_builder.md): the builder it starts from.
- [`TransformMapDefault`](./transform_map_default.md): the marker behind the defaulting.
- [`CanUpcast`](../casting/can_upcast.md): the enum analogue of widening.
- [`HasFields`](../shape/has_fields.md): what the source must derive.
- [`#[derive(CgpRecord)]`](../../derives/derive_cgp_record.md): what makes both records eligible.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): merging records through a builder.

## Source

- [`build_default.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/build_default.rs):
  `CanBuildWithDefault`, `CanFinalizeWithDefault`, and `TransformMapDefault`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
