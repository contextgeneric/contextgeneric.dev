---
title: 'IntoBuilder — a full builder from a value'
sidebar_label: 'IntoBuilder'
sidebar_position: 2
description: 'Turn a complete record into a partial one with every field present, to take fields out and put them back without naming the concrete type.'
---

# `IntoBuilder`

Turning a complete record into a partial one, with every field present.

## Overview

[`HasBuilder`](./has_builder.md) starts from nothing: a partial value with every field absent, to be
filled. `IntoBuilder` starts from the other end: `into_builder(self)` consumes an existing value and
hands back a partial one with **every field present**. You reach for it when *redistributing* a struct's
fields rather than assembling one: taking a value apart, moving some fields elsewhere, and rebuilding.

The two entry points differ only in where they begin, and they produce the same kind of value:

| | starting configuration | what it is for |
|---|---|---|
| [`HasBuilder`](./has_builder.md) | every field `IsNothing` | assembling a record from pieces |
| `IntoBuilder` | every field `IsPresent` | taking one apart and putting it back |

## Definition

`IntoBuilder` mirrors [`HasBuilder`](./has_builder.md), with one associated type and one method:

```rust
pub trait IntoBuilder {
    type Builder;

    fn into_builder(self) -> Self::Builder;
}
```

`Self` is the complete record. `Builder` is the same partial companion type
[`HasBuilder`](./has_builder.md) names, but at the all-present configuration rather than the all-absent
one. `into_builder` takes `self` by value, so it consumes the record and returns the companion with every
field already set.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

Unlike `builder()`, this is a method taking `self`, so it is called on a value: `person.into_builder()`.

The impls come from [`#[derive(BuildField)]`](../../derives/derive_build_field.md), also available through
[`#[derive(CgpRecord)]`](../../derives/derive_cgp_record.md) and
[`#[derive(CgpData)]`](../../derives/derive_cgp_data.md).

## Examples

Swapping one field of a complete record, the shape redistributing code has:

```rust
use cgp::prelude::*;
use cgp::core::field::traits::TakeField;

#[derive(Debug, PartialEq, BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

pub fn with_last_name(person: Person, last_name: String) -> Person {
    let (_old, remainder) = person
        .into_builder()
        .take_field(PhantomData::<Symbol!("last_name")>);

    remainder
        .build_field(PhantomData::<Symbol!("last_name")>, last_name)
        .finalize_build()
}

pub fn demo() {
    let person = Person {
        first_name: "Alice".to_owned(),
        last_name: "Anderson".to_owned(),
    };

    assert_eq!(
        with_last_name(person, "Chen".to_owned()),
        Person {
            first_name: "Alice".to_owned(),
            last_name: "Chen".to_owned(),
        }
    );
}
```

`into_builder` starts with every field present, [`take_field`](./take_field.md) removes `last_name`,
and [`build_field`](./build_field.md) puts a new one in its place. A value straight out of
`into_builder` is already finalizable, so the round trip earns its keep only from what happens in
between.

## When to use it

**Reach for it when the starting point is a value rather than nothing**, which is the narrower of the two
entry points and the one that shows up in generic code more than in application code.

- **Use [`HasBuilder`](./has_builder.md)** when assembling a record from independent pieces. That is the
  common case and the one the extensible builder pattern is about.
- **Use `IntoBuilder`** when a complete value must be decomposed: to swap one field, or to hand its
  fields to several destinations, including the builder of a different record.
- **Use [`ToFields`](../shape/to_fields.md)** instead when what you want is the value's *shape* as a flat list
  rather than a partial type you can fill. A partial value tracks presence; a `Fields` product does not.
- **Prefer a struct update expression** in concrete code. `Person { first_name, ..person }` does the
  common case with no machinery, and this family is for code that cannot name the type.

## Under the hood

The derive generates one partial companion per record, and the two entry points differ only in the
configuration they produce. `cargo cgp expand` on the example's `Person` shows both:

```rust
impl HasBuilder for Person {
    type Builder = __PartialPerson<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialPerson {
            first_name: (),
            last_name: (),
        }
    }
}
impl IntoBuilder for Person {
    type Builder = __PartialPerson<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialPerson {
            first_name: self.first_name,
            last_name: self.last_name,
        }
    }
}
```

Under `IsPresent` a slot holds the value itself, so `into_builder` moves each field across and
nothing about the runtime representation changes. The result is at the all-present configuration,
so it satisfies [`FinalizeBuild`](./finalize_build.md) immediately and
[`TakeField`](./take_field.md) for every field, the pair a redistribution needs.

## Common Mistakes

**It consumes the value.** There is no borrowing form; use [`ToFieldsRef`](../shape/to_fields_ref.md) if the
original must survive, bearing in mind that gives you a shape rather than a builder.

**The result is already finalizable.** Calling `into_builder().finalize_build()` is the identity, which
is legal and pointless. The trait earns its keep only if something happens in between.

**The partial type cannot be printed or cloned.** The derive drops the original's struct-level
attributes, so a value mid-redistribution has none of the record's own derives.

**It is a method, where [`HasBuilder`](./has_builder.md)'s `builder()` is an associated function.** The
asymmetry is deliberate (one starts from a value and the other from nothing), and it catches people
writing `Person::into_builder()`.

**Taking a field out makes the value unfinalizable until it is put back.** That is the guarantee, and it
means a redistribution that drops a field fails at `finalize_build` rather than where the field was
dropped.

## Related constructs

- [`HasBuilder`](./has_builder.md): the other entry point, and where the family is explained in full.
- [`TakeField`](./take_field.md): removing a present field, which this enables.
- [`BuildField`](./build_field.md): putting one back.
- [`FinalizeBuild`](./finalize_build.md): returning to the concrete struct.
- [`UpdateField`](./update_field.md): the primitive underneath both directions.
- [`PartialData`](./partial_data.md): what names the destination type mid-build.
- [`ToFields`](../shape/to_fields.md): the flat-shape alternative to a partial value.
- [`#[derive(BuildField)]`](../../derives/derive_build_field.md): generates this impl.
- [`MapType`](../type-level/map_type.md): the markers the configuration is written in.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and the extensible builder
  pattern.

## Source

- [`has_builder.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_builder.rs):
  `IntoBuilder` and `HasBuilder`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
