---
title: 'CanBuildWithDefault — widen a record'
sidebar_label: 'CanBuildWithDefault'
sidebar_position: 6
description: 'Build a wider record from a narrower one in one call, copying the shared fields by name and filling the rest from Default.'
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
[supertrait](/docs/reference/glossary#supertrait); it is a blanket impl chaining a builder, a merge, and a defaulted finalize, shown in
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

Widening a record with no field named anywhere, in one call and as the three calls it stands for:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::CanBuildFrom;
use cgp::extra::field::impls::{CanBuildWithDefault, CanFinalizeWithDefault};

#[derive(Debug, Clone, Eq, PartialEq, CgpData)]
pub struct Point2d {
    pub x: u64,
    pub y: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, CgpData)]
pub struct Point3d {
    pub x: u64,
    pub y: u64,
    pub z: u64,
}

pub fn demo() {
    let point_3d = Point3d::build_with_default(Point2d { x: 1, y: 2 });
    assert_eq!(point_3d, Point3d { x: 1, y: 2, z: 0 });

    let written_out: Point3d = Point3d::builder()
        .build_from(Point2d { x: 1, y: 2 })
        .finalize_with_default();
    assert_eq!(written_out, point_3d);
}
```

Neither struct knows about the other. They share field *names*, matched at the type level, and `z` is
filled because `u64: Default`.

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

The trait's one impl chains the three steps and constrains each with a bound:

```rust
impl<Source, Target, Builder> CanBuildWithDefault<Source> for Target
where
    Target: HasBuilder<Builder = Builder>,
    Builder: CanBuildFrom<Source>,
    Builder::Output: CanFinalizeWithDefault<Output = Target>,
{
    fn build_with_default(source: Source) -> Target {
        Target::builder().build_from(source).finalize_with_default()
    }
}
```

Each bound is where one of the requirements comes from:
[`CanBuildFrom`](../casting/can_build_from.md) brings the [`HasFields`](../shape/has_fields.md)
obligation on the source and needs every source field to exist on the target, and
[`CanFinalizeWithDefault`](./can_finalize_with_default.md) brings the `Default` obligation on the
remaining fields, through a [`TransformMapFields`](../type-level/transform_map_fields.md) walk
carrying [`TransformMapDefault`](./transform_map_default.md).

So an unsatisfied bound here is always really an unsatisfied bound one layer down. Because the call
is an associated function checked against these `where` clauses, not a method looked up on a
receiver, rustc follows the chain to the root cause and prints each layer as a `required for` note.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**The source needs [`HasFields`](../shape/has_fields.md), not only a builder.** Deriving only
[`BuildField`](../../derives/derive_build_field.md) on both looks symmetric and fails, the same trap
[`CanBuildFrom`](../casting/can_build_from.md) carries.

**Every field the source does not supply needs `Default`.** Building a `Server` from a `Host` that
lacks its `port`, where `Port` has no `Default`:

```rust
pub struct Port(pub u16);

#[derive(CgpData)]
pub struct Host {
    pub host: String,
}

#[derive(CgpData)]
pub struct Server {
    pub host: String,
    pub port: Port,
}

let _ = Server::build_with_default(Host {
    host: "localhost".to_owned(),
});
```

names the type and the conversion that needs it:

```text
error[E0277]: the trait bound `Port: Default` is not satisfied
...
   = note: required for `TransformMapDefault` to implement `TransformMap<IsNothing, IsPresent, Port>`
```

The later notes climb the chain from there, through `TransformMapFields` and
`CanFinalizeWithDefault`, to ``required for `Server` to implement `CanBuildWithDefault<Host>` ``.

**It is an associated function on the target.** `Point3d::build_with_default(source)`, not a method on
the source.

**There is no place to set a field explicitly.** If one field needs a real value, use the three-call form.

**A source field the target lacks is an error, not dropped.** The merge walks the *source's* fields
and builds each into the target, so a `label` that `Point3d` does not declare:

```rust
#[derive(CgpData)]
pub struct LabeledPoint2d {
    pub x: u64,
    pub y: u64,
    pub label: String,
}

let _ = Point3d::build_with_default(LabeledPoint2d {
    x: 1,
    y: 2,
    label: "origin".to_owned(),
});
```

has no slot to go into:

```text
error[E0277]: the trait bound `__PartialPoint3d<IsPresent, IsPresent, IsNothing>: UpdateField<Symbol<5, cgp::prelude::Chars<'l', cgp::prelude::Chars<'a', cgp::prelude::Chars<'b', cgp::prelude::Chars<'e', cgp::prelude::Chars<'l', Nil>>>>>>, IsPresent>` is not satisfied
```

The widening runs one way only: the target may have fields the source lacks, never the reverse.

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

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
