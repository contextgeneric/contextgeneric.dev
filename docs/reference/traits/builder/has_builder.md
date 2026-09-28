---
title: 'HasBuilder — start an empty builder'
sidebar_label: 'HasBuilder'
sidebar_position: 1
description: 'Start an empty builder for a record, as the entry point to assembling a struct field by field rather than through a struct literal that names the type.'
---

# `HasBuilder`

Starting an empty builder for a record.

## Overview

A struct literal supplies every field at once, in one place that names the type. `HasBuilder` is the
entry point for the case where that is impossible: the fields come from several independent places, none
of which should know the whole struct, and the result must still be checked at compile time.

`builder()` hands back a partial value with **every field absent**. Filling it is
[`BuildField`](./build_field.md), and turning it back into the concrete struct is
[`FinalizeBuild`](./finalize_build.md), which is implemented **only** for the fully-present
configuration, so finalizing early is a missing impl rather than a runtime panic:

```rust
let person = Person::builder()                                      // every field absent
    .build_field(PhantomData::<Symbol!("first_name")>, first)
    .build_field(PhantomData::<Symbol!("last_name")>, last)
    .finalize_build();                                              // all present: this resolves
```

Delete a middle line and this does not compile. **Field presence lives in the type**, which is the
defining idea of the whole family.

## Definition

`HasBuilder` is a small trait: one associated type and one associated function.

```rust
pub trait HasBuilder {
    type Builder;

    fn builder() -> Self::Builder;
}
```

`Self` is the concrete record being built. `Builder` is the *partial companion type* the derive
generates for it, the record carrying one [`MapType`](../type-level/map_type.md) presence marker per
field. `builder()` is an associated function, so it does not take a receiver, and it returns that
companion at its all-absent configuration, ready to fill.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

`builder()` is called without a receiver, as `Person::builder()` or `T::builder()` in generic code.
You rarely name the `Builder` type yourself, but it appears in an error when a build is incomplete.

The impls come from [`#[derive(BuildField)]`](../../derives/derive_build_field.md), also available through
[`#[derive(CgpRecord)]`](../../derives/derive_cgp_record.md) and
[`#[derive(CgpData)]`](../../derives/derive_cgp_data.md).

**Its counterpart is [`IntoBuilder`](./into_builder.md)**, which starts from an existing value with every
field *present* rather than from nothing. The two differ only in where they begin, and the choice is
about whether you are assembling a record or redistributing one.

## Examples

A record extended from a narrower one, and a field read back off a still-incomplete builder:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::CanBuildFrom;

// `build_from` walks the *source's* field list, so the source needs `HasFields` too.
#[derive(HasFields, BuildField)]
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
}

#[derive(Debug, PartialEq, BuildField)]
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
}

pub fn extend(foo_bar: FooBar) -> FooBarBaz {
    FooBarBaz::builder()
        .build_from(foo_bar)
        .build_field(PhantomData::<Symbol!("baz")>, true)
        .finalize_build()
}

pub fn demo() {
    // A set field can be read back off a still-incomplete builder.
    let partial = FooBarBaz::builder().build_field(PhantomData::<Symbol!("baz")>, true);
    assert!(*partial.get_field(PhantomData::<Symbol!("baz")>));

    let foo_bar_baz = extend(FooBar {
        foo: 1,
        bar: "bar".to_owned(),
    });
    assert_eq!(
        foo_bar_baz,
        FooBarBaz {
            foo: 1,
            bar: "bar".to_owned(),
            baz: true,
        }
    );
}
```

Every step changes the partial *type*, and `finalize_build` type-checks only because every marker
has reached `IsPresent`; moving it before the `baz` step is a compile error. A set field can be read
back mid-build because the derive emits a [`HasField`](../field-access/has_field.md) impl on the
partial type for each field once it is present.

## When to use it

**Call `builder()` freely in concrete code; bound on the trait only in generic code.** In concrete code
you call it and never name a trait. The bound matters when you write code that is generic over the
record being built, which is the extensible builder pattern, and the reason the family exists.

- **Bound on `HasBuilder` + [`BuildField`](./build_field.md) + [`FinalizeBuild`](./finalize_build.md)**
  to write a routine that assembles some record it does not name.
- **Use [`IntoBuilder`](./into_builder.md)** when you start from an existing value rather than from
  nothing.
- **Do not reach for any of it for a struct you build with a literal.** A literal is already checked for
  completeness, reads better, and generates nothing. This family buys *decoupling*, and with nothing to
  decouple it is pure cost.

Two boundaries are worth stating plainly. This is **not a conventional builder**: it tracks presence
and nothing else, without defaults, validation at finalize, or optional fields unless the field's
own type is optional. The [optional-field extensions](../optional/has_optional_builder.md) cover the
defaulted and optional cases, and a hand-written builder remains better when the *logic* is the
point. And the enum counterparts are a different family:
[`ExtractField`](../variant/extract_field.md) for taking a value apart and
[`FromVariant`](../variant/from_variant.md) for constructing one.

## Under the hood

`#[derive(BuildField)]` generates a companion struct, `__Partial{Name}`, with one
[`MapType`](../type-level/map_type.md) parameter per field and each field's type wrapped in that
parameter's projection. `cargo cgp expand` on a two-field `Person` shows it:

```rust
pub struct __PartialPerson<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub last_name: <__F1__ as MapType>::Map<String>,
}
```

`HasBuilder` fixes the starting configuration to all-absent, where each field is stored as `()`:

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
```

and [`FinalizeBuild`](./finalize_build.md) exists only at the opposite end:

```rust
impl FinalizeBuild for __PartialPerson<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Person {
            first_name: self.first_name,
            last_name: self.last_name,
        }
    }
}
```

That pair is the whole safety argument. `builder()` starts at all-absent, each
[`build_field`](./build_field.md) flips one marker, and nothing is checked at the end: the
impl is absent for an incomplete value. Everything between the two ends reduces to one primitive,
[`UpdateField`](./update_field.md), which is what the derive writes per field;
[`BuildField`](./build_field.md) and [`TakeField`](./take_field.md) are library blanket impls over
it in opposite directions.

## Common Mistakes

**"No method named `finalize_build`" is the expected error for an incomplete build.** A missing
field means the all-present impl does not apply, so the compiler reports a missing method rather
than a missing field. Finalizing a `Person` with only `first_name` set:

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

The marker still `IsNothing` is the missing field, here `last_name`.

**`build_from` needs [`HasFields`](../shape/has_fields.md) on the source.** It walks the source's
field list, so a source with only `#[derive(BuildField)]`:

```rust
#[derive(BuildField)]
pub struct FooBar {
    pub foo: u64,
}

#[derive(BuildField)]
pub struct FooBaz {
    pub foo: u64,
    pub baz: bool,
}

let _ = FooBaz::builder()
    .build_from(FooBar { foo: 1 })
    .build_field(PhantomData::<Symbol!("baz")>, true)
    .finalize_build();
```

fails on the source rather than on the builder:

```text
error[E0277]: the trait bound `FooBar: HasFields` is not satisfied
...
   = note: required for `__PartialFooBaz<IsNothing, IsNothing>` to implement `CanBuildFrom<FooBar>`
```

The [example](#examples) derives both on its `FooBar`.

**The partial type cannot be printed or cloned.** The derive drops the original's struct-level
attributes, so it lacks `Debug`, `Clone`, and every other derive the record carries. Read a set
field through the partial type's [`HasField`](../field-access/has_field.md) impl instead.
Field-level attributes, by contrast, are copied onto the partial type, so a field helper attribute
such as `#[serde(rename = "...")]` breaks the build; see
[`#[derive(BuildField)]`](../../derives/derive_build_field.md#common-mistakes).

**It neither defaults nor validates.** Presence is all that is tracked. A field with a sensible
default still has to be set, unless you reach for the
[optional-field extensions](../optional/can_finalize_with_default.md).

**`builder()` is an associated function.** It does not take a receiver, so it is `Person::builder()`
rather than anything called on a value. Starting from a value is [`IntoBuilder`](./into_builder.md).

**A fieldless struct's builder is immediately finalizable**, since there is nothing to track.
`Empty::builder().finalize_build()` compiles for a `struct Empty {}`, which is legal and useless.

## Related constructs

- [`IntoBuilder`](./into_builder.md): the other entry point, starting from a complete value.
- [`BuildField`](./build_field.md): setting one absent field.
- [`TakeField`](./take_field.md): the reverse, removing one present field.
- [`UpdateField`](./update_field.md): the primitive both are built from.
- [`PartialData`](./partial_data.md) and [`FinalizeBuild`](./finalize_build.md): naming the destination,
  and reaching it.
- [`CanBuildFrom`](../casting/can_build_from.md): merging every field of another record into the
  builder.
- [`#[derive(BuildField)]`](../../derives/derive_build_field.md): generates the partial type and every impl
  in the family.
- [`MapType`](../type-level/map_type.md): the `IsPresent`/`IsNothing` markers presence is encoded in.
- [`HasField`](../field-access/has_field.md): how a set field is read back off a partial value.
- [Optional fields](../optional/has_optional_builder.md): defaulted and optional finalization.
- [`ExtractField`](../variant/extract_field.md) and [`FromVariant`](../variant/from_variant.md): the enum counterparts.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and the extensible builder
  pattern.

## Source

- [`has_builder.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_builder.rs):
  `HasBuilder` and `IntoBuilder`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
