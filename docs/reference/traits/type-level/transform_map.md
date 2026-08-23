---
sidebar_label: 'TransformMap'
sidebar_position: 3
---

# `TransformMap`

The per-field function that converts a value from one marker's storage to another's.

## Overview

[`MapType`](./map_type.md) only names storage *types*: it says that an `IsNothing` field holds `()` and
an `IsPresent` field holds its value, and nothing about how you get from one to the other.
`TransformMap` carries that missing half: the function that performs the conversion.

**This is the extension point of the whole partial-type scheme**, and it is genuinely usable. Defining a
new way to re-mark a record means writing one impl per source state and then applying it with
[`TransformMapFields`](./transform_map_fields.md).

## Definition

`TransformMap` is parameterized by a source marker, a target marker, and a value type, and carries one
method:

```rust
pub trait TransformMap<M1: MapType, M2: MapType, T> {
    fn transform_mapped(value: M1::Map<T>) -> M2::Map<T>;
}
```

`Self` is a transform marker of your own naming. `M1` is the field's current state and `M2` the state it
should end in, both [`MapType`](./map_type.md) markers, and `T` is the field's value type.
`transform_mapped` takes the field's value as stored under `M1` (`M1::Map<T>`) and returns it as stored
under `M2` (`M2::Map<T>`). So one impl answers one question: given a field currently in state `M1`,
produce the same field in state `M2`.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::traits`, along with any non-prelude
marker you name — `IsOptional` comes from `cgp::core::field::impls`:

```rust
use cgp::core::field::impls::IsOptional;
use cgp::core::field::traits::TransformMap;
```

The trait is implemented on a **transform marker**, a zero-sized type you declare. It carries no data
and exists only to name the set of conversions, exactly as a provider does elsewhere in CGP.

**A transform needs an impl for every source marker a field might currently be in.** That is the rule
that decides how many impls you write, and getting it wrong is the usual failure — the walk simply does
not resolve, and the error names the missing `TransformMap` impl rather than the field.

## Examples

A marker that fills absent fields from `Default` is one impl per source state:

```rust
use cgp::core::field::impls::IsOptional;
use cgp::core::field::traits::TransformMap;
use cgp::prelude::*;

pub struct FillDefaults;

impl<T> TransformMap<IsPresent, IsPresent, T> for FillDefaults {
    fn transform_mapped(value: T) -> T {
        value
    }
}

impl<T: Default> TransformMap<IsNothing, IsPresent, T> for FillDefaults {
    fn transform_mapped(_value: ()) -> T {
        T::default()
    }
}

impl<T: Default> TransformMap<IsOptional, IsPresent, T> for FillDefaults {
    fn transform_mapped(value: Option<T>) -> T {
        value.unwrap_or_default()
    }
}
```

Read that as three cases of one rule: a present field passes through, an absent one becomes its default,
and an optional one becomes its contents or the default. Note how each signature's argument type is the
source marker's `Map<T>` — `T`, then `()`, then `Option<T>` — which makes the three impls
non-overlapping.

Because all three target `IsPresent`, applying `FillDefaults` through
[`transform_map_fields`](./transform_map_fields.md) leaves every field present, which is precisely the
configuration [`FinalizeBuild`](../builder/finalize_build.md) accepts.

**That is not hypothetical: it is how [`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) works**,
and reading it here is the shortest route to understanding that layer.

## When to use it

**Implement it to define a new per-field conversion**, and reach for the existing layer otherwise.

- **Use the [optional-field layer](../optional/can_finalize_with_default.md)** if what you want is defaulted or
  optional finalization. Both already exist, built on exactly this, and
  [`TransformMapDefault`](../optional/transform_map_default.md) and
  [`TransformOptional`](../optional/transform_optional.md) are the two markers it ships.
- **Implement `TransformMap`** for a conversion those do not cover — a validating transform, one that
  logs, one that fills from something other than `Default`.
- **Use [`TransformMapFields`](./transform_map_fields.md)** to apply what you have written. This trait
  converts one field; that one walks a whole record.
- **Do not implement [`MapType`](./map_type.md) for a new marker** and expect the derives to use it. A
  fifth state marker has no machinery behind it; a new *transform* marker, by contrast, works
  immediately.

## Under the hood

The signature's argument and return types are **projections through the two markers**, so the concrete
types differ per impl even though the trait is the same. `TransformMap<IsNothing, IsPresent, T>` has an
argument type of `<IsNothing as MapType>::Map<T>`, which normalizes to `()`, and a return type of `T`.
That lets three impls for one marker coexist without overlapping: they differ in `M1`.

[`TransformMapFields`](./transform_map_fields.md) calls this. For each field of the target's
[`HasFields`](../shape/has_fields.md) shape, it uses [`UpdateField`](../builder/update_field.md) to take the field out —
learning the marker it was in — applies `Transform::transform_mapped`, and writes the result back under
the target marker.

Two consequences follow from that construction, and both explain limits you might otherwise trip over.
The transform must have an impl for **every** source marker a field might currently be in, which is why
`FillDefaults` above needs three rather than one. And the walk is driven by the *target's* shape, so it
re-marks exactly the fields the concrete struct declares.

Nothing here has a runtime representation beyond the field values themselves: the markers are zero-sized,
and `transform_mapped` is an associated function with no receiver.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::traits`.

**It needs an impl per source state.** Writing only the `IsNothing` case leaves the recursion
unresolvable for a record with any field already present, and the error names the missing
`TransformMap` impl rather than the field that caused it.

**The argument type is the source marker's projection**, not `T`. An impl from `IsNothing` takes `()`,
one from `IsOptional` takes `Option<T>`. Writing `T` there is the most common mistake.

**It converts one field.** Applying it to a record is
[`TransformMapFields`](./transform_map_fields.md)'s job.

**A transform whose impls do not all target the same marker will not compose into a finalize.** The
layer's payoff comes from every field landing in one state, usually `IsPresent`.

**It is an associated function, not a method.** There is no receiver; the marker exists only as a name.

## Related constructs

- [`TransformMapFields`](./transform_map_fields.md) — lifts this across a whole partial record.
- [`MapType`](./map_type.md) — the markers naming each state, and the trait this supplies the missing
  half of.
- [`TransformMapDefault`](../optional/transform_map_default.md) and [`TransformOptional`](../optional/transform_optional.md)
  — the two markers CGP ships, and the models to copy.
- [`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) — the capability built from the first of
  them.
- [`UpdateField`](../builder/update_field.md) — the primitive the walk uses to take a field out and write it back.
- [`FinalizeBuild`](../builder/finalize_build.md) — what an all-`IsPresent` result is accepted by.
- [`HasFields`](../shape/has_fields.md) — the shape the walk follows.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records and how their states change.

## Source

- [`transform_map.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/transform_map.rs)
  — `TransformMap` and `TransformMapFields`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
