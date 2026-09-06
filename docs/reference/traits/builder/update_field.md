---
sidebar_label: 'UpdateField'
sidebar_position: 5
---

# `UpdateField`

The primitive that moves one field of a partial record between states.

:::info

### Generated machinery

**You are not expected to call `update_field` directly.** It is the per-field
primitive [`#[derive(BuildField)]`](../../derives/derive_build_field.md) emits, and
[`BuildField`](./build_field.md) and [`TakeField`](./take_field.md) are the two directions you actually
call. This page explains the primitive, because it is what makes build order free and what the
optional-field layer reaches for when neither direction fits.

:::

## Overview

Everything in the [builder family](./has_builder.md) reduces to one operation: change one field's storage
from one [`MapType`](../type-level/map_type.md) marker to another, and hand back what was there before.
`UpdateField` is that operation. [`BuildField`](./build_field.md) and [`TakeField`](./take_field.md) are
library blanket impls over it, each pinning one transition, which is why the derive writes this general
form once per field.

## Definition

`UpdateField` is parameterized by the field's `Tag` and a target marker `M`, and carries three associated
types and one method:

```rust
pub trait UpdateField<Tag, M: MapType> {
    type Value;
    type Mapper: MapType;   // the field's marker before the update
    type Output;            // the partial value with the field now in state M

    fn update_field(
        self,
        _tag: PhantomData<Tag>,
        value: M::Map<Self::Value>,
    ) -> (<Self::Mapper as MapType>::Map<Self::Value>, Self::Output);
}
```

`Tag` names the field, and `M` is the [`MapType`](../type-level/map_type.md) marker the field should end
in. `Value` is the field's declared type, stripped of any marker. `Mapper` is the marker the field was in
*before* the call, which the impl reports rather than requires. `Output` is the partial type with that one
field now in state `M`.

Both the `value` argument and the first component the method returns are *marker projections* rather than
bare values. `M::Map<Self::Value>` is the field's value as stored under the target marker, and
`<Self::Mapper as MapType>::Map<Self::Value>` is how it was stored before. `IsPresent` stores the value
itself and `IsNothing` stores `()`, so setting an absent field takes the real value in and hands `()`
back, and removing a present one takes `()` in and hands the real value back. One signature covers both
directions.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

The `M` parameter is the target marker, and it is usually inferred from the value you pass: supplying a
`String` where the field's type is `String` means `M = IsPresent`, and supplying `()` means
`M = IsNothing`.

The impls come from [`#[derive(BuildField)]`](../../derives/derive_build_field.md), one per field.

## Examples

You almost always reach it through the two directions rather than directly:

```rust
use cgp::prelude::*;

// BuildField: IsNothing -> IsPresent
let partial = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());
```

Naming `UpdateField` yourself is for a transition neither direction covers, which in practice means one
involving `IsOptional`:

```rust
// the shape SetOptional uses: IsOptional -> IsOptional, so the type does not change
let (previous, builder) = builder.update_field(
    PhantomData::<Symbol!("first_name")>,
    Some("Bob".to_owned()),
);
```

Because both markers are `IsOptional` here, `Output` is the same type as `Self`, which is exactly what
lets an optional field be set repeatedly, unlike the core `build_field` that consumes an absent slot once.

## When to use it

**Bound on it directly only for an operation neither [`BuildField`](./build_field.md) nor
[`TakeField`](./take_field.md) covers.** Those two are the common transitions and read far better at a
call site.

- **[`BuildField`](./build_field.md)** for `IsNothing → IsPresent`, which is setting a field.
- **[`TakeField`](./take_field.md)** for `IsPresent → IsNothing`, which is removing one.
- **`UpdateField`** for anything else: a transition to or from `IsOptional`, which the
  [optional-field extensions](../optional/set_optional.md) do, or a transform's intermediate steps.
- **[`TransformMapFields`](../type-level/transform_map_fields.md)** when *every* field changes state rather than one.
  It calls this twice per field internally, and reaching for `UpdateField` in a loop is re-implementing
  it.

## Under the hood

The derive emits one impl per field, and the key property is visible in the generics: **only the named
field's marker moves, while every other stays generic.**

```rust
impl<__M1__: MapType, __M2__: MapType, __F1__: MapType>
    UpdateField<Symbol!("first_name"), __M2__> for __PartialPerson<__M1__, __F1__>
{
    type Value = String;
    type Mapper = __M1__;                          // the marker before
    type Output = __PartialPerson<__M2__, __F1__>; // and after
    // ...
}
```

`__M1__` is unconstrained, so the impl applies whatever state the field is currently in, and `__F1__`
passes through untouched. That is precisely why fields can be built in any order: no impl cares about the
other fields' states.

Because `Mapper` is an *output* rather than an input, the two directional traits can pin a transition
by constraining it. [`BuildField`](./build_field.md) is a blanket impl over
`UpdateField<Tag, IsPresent, Mapper = IsNothing>` and [`TakeField`](./take_field.md) over
`UpdateField<Tag, IsNothing, Mapper = IsPresent>`: same primitive, opposite constraints.

Nothing here changes a value's runtime layout beyond moving it in or out of the slot; the markers are
zero-sized and the wrapping is a type-level fiction.

## Common Mistakes

**`Mapper` is the state *before* the call, not after.** The name reads either way, and getting it
backwards makes a bound resolve to the wrong impl or none at all.

**Both the argument and the returned value are marker projections**, so their concrete types depend on
`M` and `Mapper`. An impl from `IsNothing` returns `()`, one from `IsPresent` returns the value.

**It returns a tuple**, old value first and new partial second. Ignoring the first with `_` is common and
fine, but the shape catches people expecting the builder alone.

**It changes one field.** For a whole-record transition, use
[`TransformMapFields`](../type-level/transform_map_fields.md).

**Reaching for it where [`BuildField`](./build_field.md) would do makes the code harder to read** for no
gain. The directional traits exist because they say what is happening.

**A field not declared on the record has no impl**, and the error names the missing `UpdateField` bound
with the tag fully expanded rather than saying the field does not exist.

## Related constructs

- [`BuildField`](./build_field.md) and [`TakeField`](./take_field.md): the two directions, as blanket
  impls over this.
- [`HasBuilder`](./has_builder.md) and [`IntoBuilder`](./into_builder.md): where a partial value comes
  from.
- [`FinalizeBuild`](./finalize_build.md): where one ends up.
- [`MapType`](../type-level/map_type.md): the markers a field moves between.
- [`TransformMapFields`](../type-level/transform_map_fields.md): the whole-record form, which calls this twice per
  field.
- [`SetOptional`](../optional/set_optional.md): a transition this covers and the directional traits do not.
- [`#[derive(BuildField)]`](../../derives/derive_build_field.md): generates the per-field impls.
- [`Symbol!`](../../macros/symbol.md) and [`Index`](../../types/index_type.md): the tags that name a field.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and the extensible builder
  pattern.

## Source

- [`update_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/update_field.rs):
  `UpdateField`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
