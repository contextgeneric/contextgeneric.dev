---
title: 'CanUpcast — widen an enum by variant name'
sidebar_label: 'CanUpcast'
sidebar_position: 1
description: 'Convert a narrow enum into a wider one that has every one of its variants, routing each by name at the type level; the cast cannot fail.'
---

# `CanUpcast`

Widening a narrow enum into a wider one that shares its variants.

## Overview

Two enums defined independently can share variant names, say a small `FooBar` and a larger `FooBarBaz`.
Converting the narrow one into the wide one is a conversion you could write by hand, and it is
pure boilerplate: one `match` arm per variant, rewrapping each payload under the same name.

`CanUpcast` derives that conversion from the names instead. Once the source exposes its variants as
a type-level list and the target can be built one variant at a time, widening is a matter of routing
each source variant to the target's slot of the same name;
[`#[derive(CgpData)]`](../../derives/derive_cgp_data.md) on both provides everything.

**It always succeeds.** Every variant of the source has a home in the target, or the conversion does not
type-check at all, so `upcast` returns the target directly rather than a `Result`. That totality is the
difference between this trait and its narrowing counterpart,
[`CanDowncast`](./can_downcast.md).

## Definition

`CanUpcast` is a one-method trait parameterized by the target enum:

```rust
pub trait CanUpcast<Target> {
    fn upcast(self, _tag: PhantomData<Target>) -> Target;
}
```

`Self` is the narrow enum and `Target` the wider one. The method takes `self` by value and consumes
it, returning the `Target` directly rather than a `Result`, because a widening cannot fail. The
`PhantomData<Target>` argument names the target for inference and carries nothing but the type. The
target's variant set must include every one of the source's, matched by name and by payload type.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::impls`, and bring
`core::marker::PhantomData` into scope too, since the target is named with a `PhantomData` argument
rather than a turbofish on the method:

```rust
use cgp::core::field::impls::CanUpcast;
use core::marker::PhantomData;
```

**Each side derives what its role needs.** The source is walked and taken apart, so it needs
[`#[derive(HasFields)]`](../../derives/derive_has_fields.md) and
[`#[derive(ExtractField)]`](../../derives/derive_extract_field.md); the target is only built into,
so it needs [`#[derive(FromVariant)]`](../../derives/derive_from_variant.md).
[`#[derive(CgpData)]`](../../derives/derive_cgp_data.md) on both covers every direction at once.


## Examples

A narrow enum widened into a wider one, with each side deriving only what the cast needs:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::CanUpcast;

// The source is walked and taken apart, so it needs its shape and its extractor.
#[derive(Debug, Eq, PartialEq, HasFields, ExtractField)]
pub enum FooBar {
    Foo(u64),
    Bar(String),
}

// The target is only built into, one variant at a time.
#[derive(Debug, Eq, PartialEq, FromVariant)]
pub enum FooBarBaz {
    Foo(u64),
    Bar(String),
    Baz(bool),
}

pub fn demo() {
    let wide = FooBar::Foo(1).upcast(PhantomData::<FooBarBaz>);
    assert_eq!(wide, FooBarBaz::Foo(1));

    let wide = FooBar::Bar("hi".to_owned()).upcast(PhantomData::<FooBarBaz>);
    assert_eq!(wide, FooBarBaz::Bar("hi".to_owned()));
}
```

Neither enum names the other. They share variant *names*, matched at the type level, and that is the
whole coupling. Each derives only its side of the cast, as [Usage](#usage) explains: the source is
walked and taken apart, and the target is only built into.

## When to use it

**Reach for it when an implementation works in a small local enum and its result must be widened.** That
is the common use, and it is the construction-side counterpart of reading one field through a getter:
name only the variants you need, and let the widening be checked.

- **Prefer a plain `From` impl** when both types are yours, the conversion is one you would write once,
  and nothing about it needs to be generic. A hand-written `From` is clearer, requires nothing of either
  type, and generates nothing.
- **Reach for [`CanDowncast`](./can_downcast.md)** for the other direction, which can fail and therefore
  reads quite differently.
- **Reach for [`CanBuildFrom`](./can_build_from.md)** for the record analogue: merging a struct's fields
  into another struct's builder.

One boundary worth stating: this is **compile-time, name-driven, and opt-in**. Each enum must derive
the machinery its side of the cast needs, the names must match exactly, and nothing is inspected at
run time. An enum from a crate that has not derived the machinery cannot participate at all.

## Under the hood

`CanUpcast` **recurses over the source's variants**. Its one impl turns the source into its
extractor with [`HasExtractor`](../variant/has_extractor.md), walks the source's own
[`HasFields`](../shape/has_fields.md) list, and discharges what is left:

```rust
impl<Context, Source, Target, Remainder> CanUpcast<Target> for Context
where
    Context: HasFields + HasExtractor<Extractor = Source>,
    Context::Fields: FieldsExtractor<Source, Target, Remainder = Remainder>,
    Remainder: FinalizeExtract,
{
    fn upcast(self, _tag: PhantomData<Target>) -> Target {
        Context::Fields::extract_from(self.to_extractor()).finalize_extract_result()
    }
}
```

The walk is `FieldsExtractor`, implemented for each cell of the variant list. A cell pulls its
variant out with [`ExtractField`](../variant/extract_field.md), rebuilds it into the target with
[`FromVariant`](../variant/from_variant.md), or threads the remainder into the rest of the list:

```rust
impl<Source, Target, Tag, Value, RestFields, Remainder> FieldsExtractor<Source, Target>
    for Either<Field<Tag, Value>, RestFields>
where
    Source: ExtractField<Tag, Value = Value>,
    Target: FromVariant<Tag, Value = Value>,
    RestFields: FieldsExtractor<Source::Remainder, Target, Remainder = Remainder>,
{
    type Remainder = Remainder;

    fn extract_from(source: Source) -> Result<Target, Remainder> {
        match source.extract_field(PhantomData) {
            Ok(field) => Ok(Target::from_variant(PhantomData, field)),
            Err(remainder) => RestFields::extract_from(remainder),
        }
    }
}
```

and the terminal `Void` cell returns whatever remains as `Err`. Because the list walked is the
source's own, every source variant is tried, so the final remainder is uninhabited and
`Remainder: FinalizeExtract` holds;
[`finalize_extract_result`](../variant/finalize_extract_result.md) then discharges it. That is why
`upcast` returns the target directly instead of a `Result`, and why the bounds put `HasFields` and
`HasExtractor` on the source and only `FromVariant` on the target.

`FieldsExtractor` is **public**, so it can be named in a bound, and it appears by name in a failed
cast's notes. Its record-side analogue `FieldsBuilder`, behind
[`CanBuildFrom`](./can_build_from.md), is private, so it cannot be named in a bound, though it still
appears in diagnostics.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::impls`. This is the first thing that goes
wrong.

**An upcast is total only if the target really is wider.** If the target lacks one of the source's
variants, the cast fails to compile rather than at run time. With a target
`FooBaz { Foo(u64), Baz(bool) }` that lacks `FooBar`'s `Bar`:

```rust
let _ = FooBar::Foo(1).upcast(PhantomData::<FooBaz>);
```

the error names the missing constructor, then the walk that needed it:

```text
error[E0277]: the trait bound `FooBaz: FromVariant<Symbol<3, cgp::prelude::Chars<'B', cgp::prelude::Chars<'a', cgp::prelude::Chars<'r', Nil>>>>>` is not satisfied
...
   = note: required for `FooBar` to implement `CanUpcast<FooBaz>`
```

Read the `Symbol` in the headline to find the variant the target is missing.

**Names must match exactly.** Renaming a variant in one enum removes it from the overlap, and the
failure surfaces where the cast is written rather than at the rename.

**The payload types must match too.** A `Foo(u64)` does not upcast into a `Foo(u32)`; the tag and
the value type are both part of the entry, and the mismatch is reported on the target's constructor:

```text
error[E0271]: type mismatch resolving `<Foo32Bar as FromVariant<Symbol<3, Chars<'F', Chars<'o', Chars<'o', Nil>>>>>>::Value == u64`
```

**It consumes the source.** The trait lacks a borrowing form.

## Related constructs

- [`CanDowncast`](./can_downcast.md): the narrowing direction, which can fail.
- [`CanDowncastFields`](./can_downcast_fields.md): narrowing continued on a remainder.
- [`CanBuildFrom`](./can_build_from.md): the record counterpart.
- [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md) and
  [`#[derive(CgpVariant)]`](../../derives/derive_cgp_variant.md): what makes an enum eligible.
- [`ExtractField`](../variant/extract_field.md) and [`FromVariant`](../variant/from_variant.md): the two primitives the
  recursion routes through.
- [`HasFields`](../shape/has_fields.md): the variant shape being walked.
- [Type-level lists](../../types/index.md): the `Either`/`Void` chain underneath.
- [Dispatch combinators](../../providers/dispatch/index.md): where casting meets per-variant routing.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants): upcasting and downcasting between enums.

## Source

- [`cast.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/cast.rs):
  `CanUpcast` and the `FieldsExtractor` recursion

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
