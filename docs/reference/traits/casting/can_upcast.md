---
sidebar_label: 'CanUpcast'
sidebar_position: 1
---

# `CanUpcast`

Widening a narrow enum into a wider one that shares its variants.

## Overview

Two enums defined independently can share variant names — a small `FooBar` and a larger `FooBarBaz`,
say. Converting the narrow one into the wide one is a conversion you could write by hand, and it is
pure boilerplate: one `match` arm per variant, rewrapping each payload under the same name.

`CanUpcast` derives that conversion from the names instead. Once both enums expose their shape as a
type-level list — which [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md) makes them — widening is a
matter of routing each source variant to the target's slot of the same name.

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

`Self` is the narrow enum and `Target` the wider one. The method takes `self` by value and consumes it,
returning the `Target` directly rather than a `Result`, because a widening cannot fail. The
`PhantomData<Target>` argument names the target for inference and carries no data. Both enums must derive
the extensible-data machinery, and the target's variant set must include every one of the source's,
matched by name and by payload type.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::impls`, and bring
`core::marker::PhantomData` into scope too, since the target is named with a `PhantomData` argument
rather than a turbofish on the method:

```rust
use cgp::core::field::impls::CanUpcast;
use core::marker::PhantomData;
```

## Examples

Two independently-defined enums that share variant names interconvert with no manual impl:

```rust
use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;
use core::marker::PhantomData;

#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum FooBar {
    Foo(u64),
    Bar(String),
}

#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum FooBarBaz {
    Foo(u64),
    Bar(String),
    Baz(bool),
}

let wide = FooBar::Foo(1).upcast(PhantomData::<FooBarBaz>);

assert_eq!(wide, FooBarBaz::Foo(1));
```

Neither enum names the other. They share variant *names*, matched at the type level, and that is the
whole coupling.

## When to use it

**Reach for it when an implementation works in a small local enum and its result must be widened.** That
is the common use, and it is the construction-side counterpart of reading one field through a getter:
name only the variants you need, and let the widening be checked.

- **Prefer a plain `From` impl** when both types are yours, the conversion is one you would write once,
  and nothing about it needs to be generic. A hand-written `From` is clearer, requires nothing of either
  type, and generates nothing.
- **Reach for [`CanDowncast`](./can_downcast.md)** for the other direction, which can fail and therefore
  reads quite differently.
- **Reach for [`CanBuildFrom`](./can_build_from.md)** for the record analogue — merging a struct's fields
  into another struct's builder.

One boundary worth stating: this is **compile-time, name-driven, and opt-in**. Both enums must derive
the shape, the names must match exactly, and nothing is inspected at run time. An enum from a crate that
has not derived the machinery cannot participate at all.

## Under the hood

`CanUpcast` **recurses over the source's variants**. It converts the source to its extractor with
[`HasExtractor`](../variant/has_extractor.md), then walks the source's own field list, pulling each variant out
with [`ExtractField`](../variant/extract_field.md) and rebuilding it into the target with
[`FromVariant`](../variant/from_variant.md).

Because every source variant is guaranteed to exist in a wider target, the walk is total, and the
extractor left at the end is uninhabited, discharged with [`FinalizeExtract`](../variant/finalize_extract.md).
That is precisely why `upcast` returns the target directly instead of a `Result`.

One detail from the source is worth knowing if you read it: the recursion driving this,
`FieldsExtractor`, is **public**, so it can appear by name in a diagnostic and be named in a bound. Its
record-side analogue `FieldsBuilder`, behind [`CanBuildFrom`](./can_build_from.md), is private.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::impls`. This is the first thing that goes
wrong.

**An upcast is total only if the target really is wider.** If the target lacks one of the source's
variants there is no impl, and the failure is an unsatisfied bound rather than a runtime error. That is
the guarantee, but it reads as a puzzling missing-impl message until you check the variant lists.

**Names must match exactly.** Renaming a variant in one enum silently removes it from the overlap, and
the failure surfaces wherever the cast is written rather than at the rename.

**The payload types must match too.** A `Foo(u64)` does not upcast into a `Foo(u32)`; the tag and the
value type are both part of the entry.

**It consumes the source.** There is no borrowing form.

## Related constructs

- [`CanDowncast`](./can_downcast.md) — the narrowing direction, which can fail.
- [`CanDowncastFields`](./can_downcast_fields.md) — narrowing continued on a remainder.
- [`CanBuildFrom`](./can_build_from.md) — the record counterpart.
- [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md) and
  [`#[derive(CgpVariant)]`](../../derives/derive_cgp_variant.md) — what makes an enum eligible.
- [`ExtractField`](../variant/extract_field.md) and [`FromVariant`](../variant/from_variant.md) — the two primitives the
  recursion routes through.
- [`HasFields`](../shape/has_fields.md) — the variant shape being walked.
- [Type-level spines](../../types/type_level_spines.md) — the `Either`/`Void` chain underneath.
- [Dispatch combinators](../../providers/dispatch/index.md) — where casting meets per-variant routing.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — upcasting and downcasting between enums.

## Source

- [`cast.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/cast.rs) —
  `CanUpcast` and the `FieldsExtractor` recursion

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
