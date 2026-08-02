---
sidebar_label: 'CanUpcast, CanDowncast & CanBuildFrom'
---

# `CanUpcast`, `CanDowncast` & `CanBuildFrom`

Structural casts between records and between variants.

## What it's for

Two types that share a subset of named fields or variants can be converted into one another with no
hand-written `From` or `TryFrom` impl. Once a type's shape is a type-level list — which
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md) makes it — conversion becomes a matter of routing each
named entry to the matching slot in the target, and the routing is derived from the names.

Four traits cover the directions that routing can take.

**`CanUpcast`** widens an enum: a value of a narrow enum whose variants are a subset of a wider one's is lifted
into the wider one. It **always succeeds**, because every source variant has a home in the target.

**`CanDowncast`** narrows: it tries to convert a wide enum into a smaller one, succeeding only if the value's
current variant exists in the target, and otherwise handing back a *remainder* so the caller can try another
target.

**`CanDowncastFields`** is that same narrowing one level down, called on a remainder rather than on an enum — so
several candidate targets can be tried in sequence.

**`CanBuildFrom`** is the record counterpart: it fills a target's builder with whatever fields it shares with a
source, leaving the rest of the target to be supplied separately.

## Using it

**None of the four is in the prelude.** Import them from `cgp::core::field::impls`:

```rust
use cgp::core::field::impls::{CanBuildFrom, CanDowncast, CanDowncastFields, CanUpcast};
```

You will also want `core::marker::PhantomData` in scope, since every one of them names its target with a
`PhantomData` argument rather than a turbofish on the method.

### The variant casts

```rust
pub trait CanUpcast<Target> {
    fn upcast(self, _tag: PhantomData<Target>) -> Target;
}

pub trait CanDowncast<Target> {
    type Remainder;
    fn downcast(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder>;
}

pub trait CanDowncastFields<Target> {
    type Remainder;
    fn downcast_fields(self, _tag: PhantomData<Target>) -> Result<Target, Self::Remainder>;
}
```

`upcast` returns the target directly because it cannot fail. `downcast` returns a `Result` whose error is the
source's extractor with the attempted variants removed — which is what `downcast_fields` is then called on. The
difference between the two downcasts is only the starting point: `downcast` converts the enum to an extractor
first, while `downcast_fields` takes one it is already given.

### The record cast

```rust
pub trait CanBuildFrom<Source> {
    type Output;
    fn build_from(self, source: Source) -> Self::Output;
}
```

It is implemented for a **builder**, not for the target type, which is why it is written
`Target::builder().build_from(source)`. The result is the updated builder, so several sources can be absorbed in
sequence before finalizing.

**The source of a `build_from` needs [`HasFields`](./has_fields.md)** as well as a builder, because the recursion
walks the source's field list to know what to copy. The target needs only its builder.

## Examples

Two independently-defined enums that share variant names interconvert with no manual impl:

```rust
use cgp::core::field::impls::{CanDowncast, CanUpcast};
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

// upcast always succeeds: every FooBar variant exists in FooBarBaz
let wide = FooBar::Foo(1).upcast(PhantomData::<FooBarBaz>);
assert_eq!(wide, FooBarBaz::Foo(1));

// downcast succeeds for a shared variant
assert_eq!(
    FooBarBaz::Bar("hi".to_owned()).downcast(PhantomData::<FooBar>).ok(),
    Some(FooBar::Bar("hi".to_owned())),
);

// and fails for one the target lacks
assert_eq!(FooBarBaz::Baz(true).downcast(PhantomData::<FooBar>).ok(), None);
```

`CanBuildFrom` assembles one struct from several smaller ones:

```rust
use cgp::core::field::impls::CanBuildFrom;

#[derive(CgpData)] pub struct FooBar { pub foo: u64, pub bar: String }
#[derive(CgpData)] pub struct Baz { pub baz: bool }
#[derive(CgpData)] pub struct FooBarBaz { pub foo: u64, pub bar: String, pub baz: bool }

let combined: FooBarBaz = FooBarBaz::builder()
    .build_from(FooBar { foo: 1, bar: "bar".to_owned() })
    .build_from(Baz { baz: true })
    .finalize_build();
```

Neither source names the target and the target names neither source. They share field *names*, matched at the
type level, and that is the whole coupling.

## When to reach for it, and when not

**Reach for a cast when two shapes genuinely overlap and you would otherwise write the conversion by hand.**
That is the case these exist for, and it is narrower than "I have two similar types".

- **`upcast`** when an implementation works in a small local enum and the result must be widened. This is the
  common use, and it is the construction-side counterpart of reading one field through a getter: name only what
  you need, and let the widening be checked.
- **`downcast` then `downcast_fields`** when a value must be tried against several candidate targets in turn.
  Reach for the pair together — the first starts the chain and the second continues it on each remainder.
- **`build_from`** when a record is assembled from independent pieces. This is the extensible builder pattern's
  merge step.
- **Prefer a plain `From` impl** when the two types are yours, the conversion is one you would write once, and
  nothing about it needs to be generic. A hand-written `From` is clearer, has no requirements on either type, and
  generates nothing — these casts earn their keep when the conversion must be *derived* from names rather than
  written.
- **Prefer a `match`** for a one-off narrowing of a concrete enum. `downcast` pays off when the target is a type
  parameter or when the chain of candidates is long.

One boundary worth stating: these are **compile-time, name-driven, and opt-in**. Both types must derive the
shape, the names must match exactly, and nothing is inspected at run time. A type from a crate that has not
derived the machinery cannot participate at all.

## Under the hood

:::note

### Advanced

This section shows the three recursions. You do not need it to call a cast, but it explains why a downcast hands
back a remainder rather than an `Option`, and why the two downcast traits both exist.

:::

All three are driven by a recursion over a [`HasFields`](./has_fields.md) shape, and *which* shape is what
distinguishes them.

**`CanUpcast` recurses over the source's variants.** It converts the source to its extractor, then walks the
source's own field list, pulling each variant out with [`ExtractField`](./extract_field.md) and rebuilding it into
the target with [`FromVariant`](./from_variant.md). Because every source variant is guaranteed to exist in a wider
target, the walk is total — and the extractor left at the end is uninhabited, discharged with
[`FinalizeExtract`](./extract_field.md). That totality is why `upcast` returns the target directly instead of a
`Result`.

**The downcasts recurse over the *target's* variants instead.** For each `Field<Tag, Value>` in
`Target::Fields`, the implementation tries pulling that variant out of the source extractor: on success it rebuilds
the target and returns `Ok`; on failure it threads the shrunken remainder into the next attempt. If no target
variant matches, the terminal `Void` impl returns the whole remainder as `Err`.

That threading is the reason for two traits rather than one. `CanDowncast` starts by calling `to_extractor` on the
enum, while `CanDowncastFields` operates on an extractor it is handed — which is exactly the `Remainder` a prior
`downcast` returned. So chaining candidates is `downcast` followed by `downcast_fields` on each remainder, and the
remainder type narrows at every step just as it does in a hand-written
[extraction chain](./extract_field.md).

**`CanBuildFrom` recurses over the source's field product.** For each field the source exposes, it uses
[`TakeField`](./has_builder.md) to remove that value from the source and [`BuildField`](./has_builder.md) to write
it into the target builder, threading both the shrinking source and the growing builder through. When the source's
fields run out, the builder is returned — not finalized, which is what lets a second `build_from` follow.

One asymmetry in the source is worth knowing if you read it: the extractor recursion, `FieldsExtractor`, is
public, while the builder recursion `FieldsBuilder` is private. So the former can appear by name in a diagnostic
and be named in a bound; the latter cannot.

## Gotchas

**None of the four is in the prelude.** Import from `cgp::core::field::impls`. This is the first thing that goes
wrong.

**`build_from` needs [`HasFields`](./has_fields.md) on the source, not just a builder.** Deriving only
`BuildField` on both looks symmetric and does not compile; the error is an unsatisfied `HasFields` bound on the
source type.

**`build_from` is called on the builder, not the target.** `Target::builder().build_from(source)`, and the result
is a builder you still have to finalize.

**A downcast returns a remainder, not an `Option`.** `.ok()` discards it, which is fine for a single attempt and
wrong if you meant to try another target — that remainder is the input to `downcast_fields`.

**Names must match exactly, and a mismatch is a compile error rather than a runtime miss.** Renaming a variant or
field in one type silently removes it from the overlap, and the failure surfaces as an unsatisfied bound
wherever the cast is written.

**A remainder carries none of the enum's attributes**, so a `Result<Target, Remainder>` is neither `Debug` nor
`PartialEq` — comparing one whole does not compile. Reach for `.ok()`, `.is_ok()`, or a `match`.

**An upcast is total only if the target really is wider.** If the target lacks one of the source's variants there
is no impl, and the error is an unsatisfied bound rather than a runtime failure — which is the guarantee, but it
reads as a puzzling missing-impl message until you check the variant lists.

## Related constructs

- [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) — generates the shapes and machinery all four consume.
- [`HasFields`](./has_fields.md) — the shape each recursion walks, and what a `build_from` source needs.
- [`ExtractField`](./extract_field.md) and [`FromVariant`](./from_variant.md) — the primitives the variant casts
  route through.
- [`HasBuilder`](./has_builder.md) — the `TakeField`/`BuildField` pair the record cast routes through.
- [Optional fields](./optional_fields.md) — `CanBuildWithDefault`, which chains `build_from` into a defaulted
  finalize.
- [Type-level spines](../types/type_level_spines.md) — the `Either`/`Void` and `Cons`/`Nil` chains being walked.
- [`Symbol!`](../macros/symbol.md) — the tags the matching is done on.
- [Dispatch combinators](../providers/dispatch_combinators.md) — where casting meets per-variant routing.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — upcasting and downcasting between enums.
- [Extensible records](/docs/concepts/extensible-records) — merging records through a builder.

## Source

- [`cast.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/cast.rs) —
  `CanUpcast`, `CanDowncast`, `CanDowncastFields`, and the `FieldsExtractor` recursion
- [`build_from.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/build_from.rs)
  — `CanBuildFrom` and its `FieldsBuilder` recursion

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
