---
sidebar_label: 'MapType'
---

# `MapType`

The present, absent, and void type-mapping markers and their transforms.

## What it's for

CGP represents a half-built struct or a partly-extracted enum as a single type whose every field is
independently wrapped: a field that is set carries its value, a field that is not carries nothing, a variant
that has been ruled out carries an uninhabited type. Rather than hard-code those shapes,
each field is parameterized by a **marker**, and the field's actual storage type is read out of the marker.

`MapType` is that marker's trait, and it has one member:

```rust
pub trait MapType {
    type Map<T>;
}
```

`Map<T>` names the storage type the marker assigns to a field whose value type is `T`. So `IsPresent::Map<String>`
is `String`, and `IsNothing::Map<String>` is `()`. **The marker is the field's state, expressed as a type.**

That is what lets the [builder](./has_builder.md) and [extractor](./extract_field.md) families move a value
through intermediate states without changing its runtime representation. A builder starts every field
`IsNothing` and flips each to `IsPresent` as it is filled; an extractor starts every variant `IsPresent` and
flips each to `IsVoid` as it is ruled out. Because a flip is a change of *type parameter*, the compiler tracks
exactly which fields are filled — and refuses to finalize an incomplete one.

Three companions complete the picture. `MapTypeRef` does the same job for borrowed views, where the wrapping
also introduces a lifetime. `TransformMap` and `TransformMapFields` are the value-level side: they carry the
functions that actually convert a field from one wrapping to another.

## Using it

The four `MapType` markers and two of the three `MapTypeRef` markers are in the prelude. **`IsOptional` and
`IsOwned` are not** — import them from `cgp::core::field::impls`. `TransformMap` and `TransformMapFields` are
not in the prelude either; they come from `cgp::core::field::traits`.

### The `MapType` markers

Four markers cover the states a field passes through:

```rust
impl MapType for IsPresent  { type Map<T> = T; }         // the field holds its value
impl MapType for IsNothing  { type Map<T> = (); }        // the field is absent
impl MapType for IsVoid     { type Map<T> = Void; }      // the case is impossible
impl MapType for IsOptional { type Map<T> = Option<T>; } // the field may or may not be there
```

The distinction between `IsNothing` and `IsVoid` is the one that matters and the one most easily blurred.
`IsNothing` maps to `()`, which is **inhabited** — an absent field is a real state a value can be in, which is
why a builder needs an explicit all-present impl to finalize. `IsVoid` maps to the uninhabited `Void`, so a
value with every marker `IsVoid` **cannot exist** — which is exactly how an extractor discharges its remainder
with an empty `match`. Records use the first, variants the second.

### The `MapTypeRef` markers

`MapTypeRef` threads a lifetime, so the marker decides not just presence but how the value is held:

```rust
pub trait MapTypeRef {
    type Map<'a, T: 'a>: 'a;
}

impl MapTypeRef for IsRef   { type Map<'a, T: 'a> = &'a T; }
impl MapTypeRef for IsMut   { type Map<'a, T: 'a> = &'a mut T; }
impl MapTypeRef for IsOwned { type Map<'a, T: 'a> = T; }
```

A borrowed extractor combines both families: one outer `MapTypeRef` marker shared across all fields, and one
`MapType` marker per field, so a field's storage is `MapType::Map<MapTypeRef::Map<'a, T>>`. `HasExtractorRef`
fixes the outer marker to `IsRef` and `HasExtractorMut` to `IsMut`.

**`IsOwned` has no consumer in CGP.** No derive emits it and no provider resolves against it — it is available
for a borrowed view that holds its payloads by value, and at present that view exists only if you write it
yourself.

### The transforms

`MapType` only names storage types. `TransformMap` carries the *function* that converts a value from one
wrapping to another:

```rust
pub trait TransformMap<M1: MapType, M2: MapType, T> {
    fn transform_mapped(value: M1::Map<T>) -> M2::Map<T>;
}
```

`TransformMapFields` lifts such a transform across a whole partial record, walking the target's
[`HasFields`](./has_fields.md) shape field by field:

```rust
pub trait TransformMapFields<Transform, TargetMap> {
    type Output;

    fn transform_map_fields(self) -> Self::Output;
}
```

This is the pair to reach for when you need to re-mark every field at once — which is what the
[optional-field extensions](./optional_fields.md) are built from.

## Examples

You mostly meet the markers inside the partial types that
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md) generates, where a builder's type reads
`__PartialPerson<IsNothing, IsPresent>` and tells you at a glance which fields are set.

The transforms, though, are directly useful. A marker that fills absent fields from `Default` is one impl per
source state:

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

Read that as three cases of one rule: a present field passes through, an absent one becomes its default, and an
optional one becomes its contents or the default. Because all three target `IsPresent`, applying it through
`transform_map_fields` leaves every field present — which is precisely the configuration
[`FinalizeBuild`](./has_builder.md) accepts, turning a partially-built record into a finalizable one.

That is not hypothetical: it is how `CanFinalizeWithDefault` in the
[optional-field extensions](./optional_fields.md) works, and reading it here is the shortest route to
understanding that layer.

## When to reach for it, and when not

**You will name the markers far more often than the traits.** `IsPresent` and `IsNothing` appear whenever you
read a builder's type in an error; `IsVoid` whenever you read an extractor's. Recognizing them is the common
case, and it is most of what this page is for.

The traits themselves have narrower uses.

- **Bound on `MapType`** when you write generic code over a partial type's marker parameter — which is what the
  builder and extractor families do internally, and what you would do writing your own partial-type machinery.
- **Implement `TransformMap`** to define a new per-field conversion, as `FillDefaults` above does. This is the
  extension point of the whole scheme, and it is genuinely usable: one impl per source marker, and
  `transform_map_fields` applies it everywhere.
- **Do not implement `MapType` for a new marker** expecting the derives to use it. The generated partial types
  are parameterized over markers, but only the four standard ones have transforms and finalize impls behind
  them; a fifth marker is a type with no machinery attached.
- **Reach for the [optional-field extensions](./optional_fields.md) rather than writing the transform yourself**
  if what you want is defaulted or optional finalization. Both already exist, built on exactly this.

`MapFields` — the other trait with a similar name — is a different thing: it applies one marker uniformly to
every entry of a type-level list, and lives with the [product operations](./product_ops.md).

## Under the hood

:::note

### Advanced

This section shows how the transform recursion works. You do not need it to read a marker in an error, but it is
what the optional-field layer is made of.

:::

`TransformMapFields` is implemented for any [`PartialData`](./has_builder.md) and walks the target's
[`HasFields`](./has_fields.md) product one entry at a time. For each `Field<Tag, Value>` it uses
[`UpdateField`](./has_builder.md) **twice**: it takes the field out — replacing its marker with `IsNothing` and
reading what the marker was — applies `Transform::transform_mapped` to convert the value into the
`TargetMap` wrapping, then writes it back under `TargetMap`.

So the result type has every field re-marked to `TargetMap`, with the values converted accordingly. Two things
follow from that construction, and both explain limits you might otherwise trip over. The transform must have an
impl for **every** source marker a field might currently be in, or the walk does not resolve — which is why
`FillDefaults` above needs three impls rather than one. And the recursion is driven by the *target's* shape, so
it re-marks exactly the fields the concrete struct declares.

The markers themselves are zero-sized and carry no data. Nothing here has a runtime representation: a `MapType`
impl is a type-level function, and the only values that exist are the field values being wrapped or unwrapped.

## Gotchas

**`IsNothing` and `IsVoid` are not interchangeable.** `IsNothing` is inhabited (`()`), `IsVoid` is not (`Void`).
Records use the first, variants the second, and an error naming the wrong one usually means the two families
have been crossed.

**`IsOptional` and `IsOwned` are not in the prelude.** Import them from `cgp::core::field::impls`. The other
five markers are.

**`IsOwned` is unused by CGP.** It is a legal `MapTypeRef` marker with no consumer, so selecting it means
writing the machinery that uses it yourself.

**`MapType` and `MapFields` are different traits.** `MapType` is one marker naming one field's storage;
[`MapFields`](./product_ops.md) applies a marker across a whole list. The names are close and the jobs are not.

**A new marker gets you a type and nothing else.** Implementing `MapType` is easy; the derives' finalize and
transform impls are written against the standard markers, so a custom one has no machinery behind it.

**`TransformMap` needs an impl per source state.** Writing only the `IsNothing` case leaves the recursion
unresolvable for a record with any field already present, and the error names the missing `TransformMap` impl
rather than the field.

## Related constructs

- [`HasBuilder`](./has_builder.md) — uses `IsPresent`/`IsNothing` to track which fields are filled.
- [`ExtractField`](./extract_field.md) — uses `IsPresent`/`IsVoid` to track which variants remain, and
  `MapTypeRef` for its borrowed forms.
- [Optional fields](./optional_fields.md) — the layer built from `TransformMap` and `IsOptional`.
- [`MapFields`](./product_ops.md) — applies one marker across a whole type-level list.
- [`HasFields`](./has_fields.md) — the shape `TransformMapFields` walks.
- [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) — generates the partial types these markers
  parameterize.
- [Type-level spines](../types/type_level_spines.md) — where the uninhabited `Void` comes from.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — presence tracking on a partial record.
- [Extensible variants](/docs/concepts/extensible-variants) — possibility tracking on a partial variant.

## Source

- [`map_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_type.rs)
  and [`map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_type_ref.rs)
  — the two traits
- [`impls/map_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/map_type.rs)
  — `IsPresent`, `IsNothing`, `IsVoid`, `IsOptional`
- [`impls/map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/map_type_ref.rs)
  — `IsRef`, `IsMut`, `IsOwned`
- [`transform_map.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/transform_map.rs)
  — `TransformMap`, `TransformMapFields`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
