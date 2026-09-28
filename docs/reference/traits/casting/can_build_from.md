---
title: 'CanBuildFrom — merge a record into a builder'
sidebar_label: 'CanBuildFrom'
sidebar_position: 4
description: 'Move every field of one record into another record''s builder by field name, so several sources can be merged before the target is finalized.'
---

# `CanBuildFrom`

Moving every field of one record into another record's builder.

## Overview

Assembling one struct out of several smaller ones means copying each field across by hand: a
line per field, repeated for every pair of types. `CanBuildFrom` derives that copying from the field
names instead.

It is the record counterpart of the [variant casts](./can_upcast.md), and it differs from them in
one way that shapes how it is written: **it is implemented for a builder rather than for the target
type**. Calling it moves every field of the source into a partial value and hands the builder back,
so several sources can be absorbed in sequence before the result is finalized.

That is the merge step of the extensible builder pattern: independent parts of a program each produce a
record, and the target absorbs all of them without any of them naming it.

## Definition

`CanBuildFrom` is parameterized by the source record, with an `Output` type and one method:

```rust
pub trait CanBuildFrom<Source> {
    type Output;

    fn build_from(self, source: Source) -> Self::Output;
}
```

`Self` is the builder, not the target type, which is why the call reads
`Target::builder().build_from(source)`. `Source` is the record being copied from, taken by value.
`Output` is the updated builder rather than the finished struct, which lets a second `build_from` follow
before a finalize.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::impls`:

```rust
use cgp::core::field::impls::CanBuildFrom;
```

**The source needs [`HasFields`](../shape/has_fields.md) as well as a builder**, because the
recursion walks the source's field list to know what to copy. The target needs only its builder, and
must declare every field the source carries. Deriving only
[`BuildField`](../../derives/derive_build_field.md) on both looks symmetric and does not compile,
which is the mistake this trait causes most often.

## Examples

Two independent records merged into a third, and a merge followed by the one field its source did
not carry:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::CanBuildFrom;

#[derive(CgpData)]
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
}

#[derive(CgpData)]
pub struct Baz {
    pub baz: bool,
}

#[derive(Debug, Eq, PartialEq, CgpData)]
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
}

// A source rarely covers everything, so a merge is often followed by an explicit field.
#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Eq, PartialEq, CgpData)]
pub struct Employee {
    pub employee_id: u64,
    pub first_name: String,
    pub last_name: String,
}

pub fn hire(person: Person, employee_id: u64) -> Employee {
    Employee::builder()
        .build_from(person)
        .build_field(PhantomData::<Symbol!("employee_id")>, employee_id)
        .finalize_build()
}

pub fn demo() {
    let combined: FooBarBaz = FooBarBaz::builder()
        .build_from(FooBar {
            foo: 1,
            bar: "bar".to_owned(),
        })
        .build_from(Baz { baz: true })
        .finalize_build();
    assert_eq!(
        combined,
        FooBarBaz {
            foo: 1,
            bar: "bar".to_owned(),
            baz: true,
        }
    );

    let person = Person {
        first_name: "Alice".to_owned(),
        last_name: "Anderson".to_owned(),
    };
    assert_eq!(hire(person, 7).employee_id, 7);
}
```

Neither source names the target and the target names neither source. They share field *names*,
matched at the type level, and that is the whole coupling. `hire` shows the usual shape, since a
source rarely covers everything: `build_from` returns a builder, so an explicit
[`build_field`](../builder/build_field.md) can follow it before `finalize_build`.

## When to use it

**Reach for it when a record is assembled from independent pieces.** That is the extensible builder
pattern's merge step, and the reason the trait exists.

- **Prefer a struct literal** when one place knows every field. A literal is already checked for
  completeness, reads better, and generates nothing.
- **Prefer a plain `From` impl** when the two types are yours and the conversion is one you would
  write once. This trait pays off when the conversion must be *derived* from names rather than
  written.
- **Reach for [`CanBuildWithDefault`](../optional/can_build_with_default.md)** when the target has fields the source
  does not cover and their defaults are acceptable: it chains `builder()`, `build_from`, and a defaulted
  finalize into one call.
- **Reach for [`CanUpcast`](./can_upcast.md)** for the enum analogue.

One boundary worth stating: this is **compile-time, name-driven, and opt-in**. Both types must
derive their part of the machinery, the names must match exactly, and nothing is inspected at run
time.

## Under the hood

`CanBuildFrom` **recurses over the source's field product.** Its one impl turns the source into a
full builder with [`IntoBuilder`](../builder/into_builder.md) and walks the source's
[`HasFields`](../shape/has_fields.md) list:

```rust
impl<Builder, Source, Output> CanBuildFrom<Source> for Builder
where
    Source: HasFields + IntoBuilder,
    Source::Fields: FieldsBuilder<Source::Builder, Builder, Output = Output>,
{
    type Output = Output;

    fn build_from(self, source: Source) -> Output {
        Source::Fields::build_fields(source.into_builder(), self)
    }
}
```

For each field, `FieldsBuilder` takes the value out of the source with
[`TakeField`](../builder/take_field.md) and writes it into the target with
[`BuildField`](../builder/build_field.md), threading both the shrinking source and the growing
builder:

```rust
impl<Source, Target, RestFields, Tag, Value> FieldsBuilder<Source, Target>
    for Cons<Field<Tag, Value>, RestFields>
where
    Source: TakeField<Tag, Value = Value>,
    Target: BuildField<Tag, Value = Value>,
    RestFields: FieldsBuilder<Source::Remainder, Target::Output>,
{
    type Output = RestFields::Output;

    fn build_fields(source: Source, target: Target) -> Self::Output {
        let (value, next_source) = source.take_field(PhantomData);
        let next_target = target.build_field(PhantomData, value);

        RestFields::build_fields(next_source, next_target)
    }
}
```

The `Target: BuildField<Tag>` bound is what makes every source field mandatory in the target, and
absent there before the merge. When the source's fields run out, the `Nil` impl returns the builder,
**not finalized**, which lets a second `build_from` follow and is why `Output` is a builder type
rather than the target struct.

`FieldsBuilder` is **private**, while the variant casts' `FieldsExtractor` is public. So only the
extractor recursion can be named in a bound; both still appear by name in a failed call's notes,
this one as `cgp::cgp_core::cgp_field::impls::build_from::FieldsBuilder`.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::impls`.

**The source needs [`HasFields`](../shape/has_fields.md), not only a builder.** Deriving only
[`BuildField`](../../derives/derive_build_field.md) on both structs looks symmetric and fails; the
error is an unsatisfied `HasFields` bound on the source type.

**It is called on the builder, not the target.** `Target::builder().build_from(source)`, and the result
is a builder you still have to [finalize](../builder/finalize_build.md).

**Every source field must exist in the target, so a renamed field fails at the merge.** A source
spelling `surname` where the target says `last_name`:

```rust
#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub surname: String,
}

#[derive(CgpData)]
pub struct Employee {
    pub first_name: String,
    pub last_name: String,
}

let _ = Employee::builder()
    .build_from(Person {
        first_name: "Alice".to_owned(),
        surname: "Anderson".to_owned(),
    })
    .finalize_build();
```

has nowhere to put `surname`, and the error is on `build_from`:

```text
error[E0277]: the trait bound `__PartialEmployee<IsPresent, IsNothing>: UpdateField<Symbol<7, cgp::prelude::Chars<'s', cgp::prelude::Chars<'u', cgp::prelude::Chars<'r', cgp::prelude::Chars<'n', cgp::prelude::Chars<'a', cgp::prelude::Chars<'m', cgp::prelude::Chars<'e', Nil>>>>>>>>, IsPresent>` is not satisfied
...
   = note: required for `__PartialEmployee<IsNothing, IsNothing>` to implement `CanBuildFrom<Person>`
```

A source field the target lacks is never silently dropped. The reverse, a target field no source
carries, is legal and is left for a [`build_field`](../builder/build_field.md) or a defaulted
finalize.

**Two sources may not carry the same field.** Merging `FooBar { foo, bar }` and then
`FooBaz { foo, baz }` into one builder sets `foo` twice, which fails on the second merge as
[`BuildField`](../builder/build_field.md#common-mistakes)'s build-twice mismatch:

```text
error[E0271]: type mismatch resolving `<__PartialFooBarBaz<IsPresent, IsPresent, IsNothing> as UpdateField<Symbol<3, Chars<'f', Chars<'o', Chars<'o', Nil>>>>, IsPresent>>::Mapper == IsNothing`
```

**It consumes the source.** The trait lacks a borrowing form.

## Related constructs

- [`HasBuilder`](../builder/has_builder.md): where a builder comes from, and the family this belongs to.
- [`BuildField`](../builder/build_field.md) and [`TakeField`](../builder/take_field.md): the two primitives the recursion
  routes through.
- [`FinalizeBuild`](../builder/finalize_build.md): how the resulting builder becomes a struct.
- [`HasFields`](../shape/has_fields.md): the shape the walk reads, and what the source must derive.
- [`CanBuildWithDefault`](../optional/can_build_with_default.md): merge plus a defaulted finalize, in one call.
- [`CanUpcast`](./can_upcast.md): the enum counterpart.
- [`#[derive(CgpRecord)]`](../../derives/derive_cgp_record.md): what makes a struct eligible.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): merging records through a builder.

## Source

- [`build_from.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/build_from.rs):
  `CanBuildFrom` and its `FieldsBuilder` recursion

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
