---
sidebar_label: 'CanBuildFrom'
---

# `CanBuildFrom`

Filling a builder with every field it shares with another record.

## Overview

Assembling one struct out of several smaller ones means copying each shared field across by hand — a
line per field, repeated for every pair of types. `CanBuildFrom` derives that copying from the field
names instead.

It is the record counterpart of the [variant casts](./can_upcast.md), and it differs from them in one
way that shapes how it is written: **it is implemented for a builder rather than for the target type**.
Calling it fills a partial value with whatever fields it shares with the source and hands the builder
back, so several sources can be absorbed in sequence before the result is finalized.

That is the merge step of the extensible builder pattern: independent parts of a program each produce a
record, and the target absorbs all of them without any of them naming it.

## Using it

**It is not in the prelude.** Import it from `cgp::core::field::impls`:

```rust
use cgp::core::field::impls::CanBuildFrom;
```

```rust
pub trait CanBuildFrom<Source> {
    type Output;

    fn build_from(self, source: Source) -> Self::Output;
}
```

`Self` is the builder, which is why the call reads `Target::builder().build_from(source)`. `Output` is
the updated builder — **not** the finished struct, which is what lets a second `build_from` follow.

**The source needs [`HasFields`](./has_fields.md) as well as a builder**, because the recursion walks the
source's field list to know what to copy. The target needs only its builder. Deriving the same thing on
both looks symmetric and does not compile, which is the mistake this trait causes most often.

## Examples

Two independent records merged into a third, which names neither of them:

```rust
use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

#[derive(CgpData)] pub struct FooBar { pub foo: u64, pub bar: String }
#[derive(CgpData)] pub struct Baz { pub baz: bool }
#[derive(CgpData)] pub struct FooBarBaz { pub foo: u64, pub bar: String, pub baz: bool }

let combined: FooBarBaz = FooBarBaz::builder()
    .build_from(FooBar { foo: 1, bar: "bar".to_owned() })
    .build_from(Baz { baz: true })
    .finalize_build();
```

Neither source names the target and the target names neither source. They share field *names*, matched
at the type level, and that is the whole coupling.

Mixing a merge with an explicit field is the usual shape, since a source rarely covers everything:

```rust
let employee = Employee::builder()
    .build_from(person)                                      // the shared fields
    .build_field(PhantomData::<Symbol!("employee_id")>, 7)   // and the one it did not carry
    .finalize_build();
```

## When to reach for it, and when not

**Reach for it when a record is assembled from independent pieces.** That is the extensible builder
pattern's merge step, and it is what the trait exists for.

- **Prefer a struct literal** when one place knows every field. A literal is already checked for
  completeness, reads better, and generates nothing.
- **Prefer a plain `From` impl** when the two types are yours and the conversion is one you would write
  once. This trait earns its keep when the conversion must be *derived* from names rather than written.
- **Reach for [`CanBuildWithDefault`](./can_build_with_default.md)** when the target has fields the source
  does not cover and their defaults are acceptable — it chains `builder()`, `build_from`, and a defaulted
  finalize into one call.
- **Reach for [`CanUpcast`](./can_upcast.md)** for the enum analogue.

One boundary worth stating: this is **compile-time, name-driven, and opt-in**. Both types must derive the
machinery, the names must match exactly, and nothing is inspected at run time.

## Under the hood

:::note

### Advanced

This section shows the recursion, which is the record mirror of the variant casts'.

:::

`CanBuildFrom` **recurses over the source's field product.** For each field the source exposes, it uses
[`TakeField`](./take_field.md) to remove that value from the source and [`BuildField`](./build_field.md)
to write it into the target builder, threading both the shrinking source and the growing builder through
the walk.

When the source's fields run out, the builder is returned — **not finalized**, which is exactly what
lets a second `build_from` follow, and why `Output` is a builder type rather than the target struct.

One asymmetry in the source is worth knowing if you read it: this recursion, `FieldsBuilder`, is
**private**, while the variant casts' `FieldsExtractor` is public. So the extractor recursion can appear
by name in a diagnostic and be named in a bound; this one cannot.

## Gotchas

**It is not in the prelude.** Import from `cgp::core::field::impls`.

**The source needs [`HasFields`](./has_fields.md), not just a builder.** Deriving only
[`BuildField`](../derives/derive_build_field.md) on both structs looks symmetric and fails; the error is
an unsatisfied `HasFields` bound on the source type.

**It is called on the builder, not the target.** `Target::builder().build_from(source)`, and the result
is a builder you still have to [finalize](./finalize_build.md).

**Field names are the whole interface.** Renaming a field in one struct silently stops it being copied,
and the failure surfaces at `finalize_build` — a missing-method error — rather than at the rename.

**A field the target lacks is not an error and not copied.** The walk matches on the target's slots, so
an extra field on the source is simply dropped.

**It consumes the source.** There is no borrowing form.

## Related constructs

- [`HasBuilder`](./has_builder.md) — where a builder comes from, and the family this belongs to.
- [`BuildField`](./build_field.md) and [`TakeField`](./take_field.md) — the two primitives the recursion
  routes through.
- [`FinalizeBuild`](./finalize_build.md) — how the resulting builder becomes a struct.
- [`HasFields`](./has_fields.md) — the shape the walk reads, and what the source must derive.
- [`CanBuildWithDefault`](./can_build_with_default.md) — merge plus a defaulted finalize, in one call.
- [`CanUpcast`](./can_upcast.md) — the enum counterpart.
- [`#[derive(CgpRecord)]`](../derives/derive_cgp_record.md) — what makes a struct eligible.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — merging records through a builder.

## Source

- [`build_from.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/build_from.rs)
  — `CanBuildFrom` and its `FieldsBuilder` recursion

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
