---
title: 'FinalizeBuild — end a build'
sidebar_label: 'FinalizeBuild'
sidebar_position: 6
description: 'Turn a partial record with every field present back into the concrete struct; the impl exists only at that configuration.'
---

# `FinalizeBuild`

Turning a fully-built [partial record](/docs/reference/glossary#partial-record) back into the concrete struct.

## Overview

`FinalizeBuild` is where a build ends. What makes it the family's safety argument is not the method but
**where the impl exists**. It is
implemented for exactly one configuration of a partial record, the one in which every field is
`IsPresent`, so calling it on an incomplete value is a *missing impl*, not a runtime check that fails:

```rust
let person = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, first)
    .build_field(PhantomData::<Symbol!("last_name")>, last)
    .finalize_build();                                       // resolves only here
```

Delete a middle line and this does not compile. There is nothing to run and nothing to panic: the method
is not in scope for a value with a field still absent.

The destination type comes from its [supertrait](/docs/reference/glossary#supertrait) [`PartialData`](./partial_data.md), which every
configuration implements. That division is deliberate: **one names where you are going, the other says
you have arrived.**

## Definition

`FinalizeBuild` has one method and a supertrait:

```rust
pub trait FinalizeBuild: PartialData {
    fn finalize_build(self) -> Self::Target;
}
```

`finalize_build` takes `self` by value, so it consumes the partial record, and returns
`Self::Target`, the concrete struct being built. `Target` is not declared here: it comes from the
supertrait [`PartialData`](./partial_data.md), which every configuration of a partial record
implements, so the destination is nameable at any point in a build. `FinalizeBuild` adds the method,
and its impl exists for only one configuration (every field `IsPresent`), which is the safety
argument the rest of this page works out.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

`finalize_build` consumes the partial value and returns `Self::Target`. It takes only `self`:
nothing is left to decide by the time it applies.

Bounding on it is how generic builder code says it will produce a finished value, as the
`assemble` function in the [example](#examples) does with `FinalizeBuild<Target = Target>`.

The impls come from [`#[derive(BuildField)]`](../../derives/derive_build_field.md), which emits exactly one
per record.

## Examples

A generic finalize over any complete builder, including a fieldless record:

```rust
use cgp::prelude::*;

#[derive(Debug, PartialEq, BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, PartialEq, BuildField)]
pub struct Empty {}

pub fn assemble<Partial, Target>(partial: Partial) -> Target
where
    Partial: FinalizeBuild<Target = Target>,
{
    partial.finalize_build()
}

pub fn demo() {
    let person: Person = assemble(
        Person::builder()
            .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
            .build_field(PhantomData::<Symbol!("last_name")>, "Chen".to_owned()),
    );
    assert_eq!(person.last_name, "Chen");

    // A fieldless record's builder is complete from the start.
    let empty: Empty = assemble(Empty::builder());
    assert_eq!(empty, Empty {});
}
```

`assemble` names only `FinalizeBuild`, yet it can bind `Target`, because the projection comes from
the supertrait [`PartialData`](./partial_data.md). `Empty` lacks markers to fill, so its builder is
already at the one configuration the impl covers.

## When to use it

**Call it at the end of every build; bound on it when the code is generic over the record.**

- **Bound on [`HasBuilder`](./has_builder.md) + [`BuildField`](./build_field.md) + `FinalizeBuild`** to
  write a routine that assembles a record it does not name.
- **Use [`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md)** when unset fields should be filled
  from `Default` rather than rejected. It re-marks every field to `IsPresent` and then calls this, so the
  strict check still runs and always passes.
- **Use [`FinalizeOptional`](../optional/finalize_optional.md)** when absence should be *reported* at run time
  rather than caught at compile time. It returns a `Result` naming a missing field.
- **Stay with `FinalizeBuild`** when every field is genuinely required. Moving to either of the other two
  gives up the compile-time completeness check, which is the reason the family exists.

## Under the hood

The derive emits one impl, with every marker fixed. `cargo cgp expand` on the example's `Person`
shows it beside its supertrait's impl, which leaves every marker generic:

```rust
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialPerson<__F0__, __F1__> {
    type Target = Person;
}
impl FinalizeBuild for __PartialPerson<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Person {
            first_name: self.first_name,
            last_name: self.last_name,
        }
    }
}
```

So a partial value always knows its destination and only sometimes has a way to reach it. Because
`IsPresent::Map<T>` is `T`, the body is a field-by-field move with nothing to unwrap. **This is why
the error for an incomplete build is a missing method rather than a missing field**: method
resolution looks for `finalize_build` on `__PartialPerson<IsPresent, IsNothing>`, does not find an
impl, and reports that, so the marker list in the type is the diagnostic.

The optional layer reaches this same impl rather than replacing it:
[`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) runs a
[`TransformMapFields`](../type-level/transform_map_fields.md) walk to `IsPresent` first, so **the
strict, all-present impl remains the only way a partial value becomes a concrete struct.**

## Common Mistakes

**"No method named `finalize_build`" is the expected error for an incomplete build.** Setting only
`first_name` on the example's `Person` and finalizing:

```rust
let person: Person = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
    .finalize_build();
```

fails with:

```text
error[E0599]: no method named `finalize_build` found for struct `__PartialPerson<__F0__, __F1__>` in the current scope
...
   | |         -^^^^^^^^^^^^^^ method not found in `__PartialPerson<IsPresent, IsNothing>`
```

The field whose marker is still `IsNothing` is the one missing.

**It consumes the partial value**, so the builder cannot be reused afterwards.

**It supertraits [`PartialData`](./partial_data.md)**, so naming both in a bound is redundant, and
`Target` is projected from the supertrait.

**It lacks a fallible variant.** Reporting absence at run time is
[`FinalizeOptional`](../optional/finalize_optional.md), a different trait in a different crate.

**A fieldless struct finalizes immediately.** Its companion lacks markers, so
`builder().finalize_build()` compiles, which is legal and useless.

**The enum side has its own ending.** [`FinalizeExtract`](../variant/finalize_extract.md) discharges an exhausted
extractor, and it is sound for the opposite reason: the value cannot exist, rather than being complete.

## Related constructs

- [`PartialData`](./partial_data.md): the supertrait naming the destination.
- [`HasBuilder`](./has_builder.md) and [`IntoBuilder`](./into_builder.md): where a partial value comes
  from.
- [`BuildField`](./build_field.md) and [`TakeField`](./take_field.md): what moves it between
  configurations.
- [`UpdateField`](./update_field.md): the primitive underneath both.
- [`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) and
  [`FinalizeOptional`](../optional/finalize_optional.md): the two relaxed endings.
- [`MapType`](../type-level/map_type.md): the `IsPresent` marker every field must reach.
- [`FinalizeExtract`](../variant/finalize_extract.md): the enum family's ending, sound for the opposite reason.
- [`#[derive(BuildField)]`](../../derives/derive_build_field.md): generates this impl.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and the extensible builder
  pattern.

## Source

- [`build_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/build_field.rs):
  `FinalizeBuild` and `BuildField`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
