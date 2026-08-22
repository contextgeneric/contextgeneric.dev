---
sidebar_label: 'ToOptional'
---

# `ToOptional`

Re-marking every field of an existing builder as optional.

## Overview

[`HasOptionalBuilder`](./has_optional_builder.md) starts an all-optional builder from nothing.
`ToOptional` is the conversion for the case where you already hold a partial value and want to relax it:

```rust
pub trait ToOptional {
    type Output;

    fn to_optional(self) -> Self::Output;
}
```

Every field is re-marked to `IsOptional`, whatever state it was in — a set field becomes `Some(value)`,
an absent one becomes `None` — and from there the value behaves like any other optional builder:
[`SetOptional`](./set_optional.md) applies, and either
[`FinalizeOptional`](./finalize_optional.md) or [`CanFinalizeWithDefault`](./can_finalize_with_default.md)
ends it.

**It is the conversion `optional_builder()` is built from**, which is the shortest way to describe both:
one starts fresh and converts, the other converts something you already have.

## Using it

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::ToOptional;
```

`Output` is the same partial companion type with every marker set to `IsOptional`. There is nothing to
implement — the impl is a blanket one over
[`TransformMapFields`](./transform_map_fields.md), so any partial value whose fields can be re-marked
gets it.

## Examples

Relaxing a partly-filled core builder so the remaining fields can arrive in any order:

```rust
use cgp::extra::field::impls::{FinalizeOptional, SetOptional, ToOptional};
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

let builder = Context::builder()
    .build_field(PhantomData::<Symbol!("foo")>, "foo".to_owned())
    .to_optional();                                    // foo is Some, bar is None

let context = builder
    .set(PhantomData::<Symbol!("bar")>, 42)
    .finalize_optional()
    .unwrap();

assert_eq!(context.bar, 42);
```

The already-set `foo` survives the conversion as `Some("foo")`, which is what distinguishes this from
starting over with [`optional_builder()`](./has_optional_builder.md).

Converting a *complete* value works too, by way of
[`into_builder`](./into_builder.md):

```rust
let builder = context.into_builder().to_optional();   // every field Some
```

## When to reach for it, and when not

**Reach for it when a builder already exists and its strictness has become the wrong shape.**

- **[`HasOptionalBuilder`](./has_optional_builder.md)** when nothing has been built yet.
  `optional_builder()` is `builder().to_optional()` and reads better.
- **`ToOptional`** when a core builder is partly filled — because a routine handed it to you, or because
  the first few fields were known and the rest are not.
- **[`IntoBuilder`](./into_builder.md) then `to_optional`** to relax a complete value, typically before
  overwriting some of its fields.
- **Stay on the [core builder](./has_builder.md)** if the remaining fields are known and set once. The
  conversion costs the compile-time completeness check.

## Under the hood

:::note

### Advanced

This section shows the transform it drives.

:::

`ToOptional` is a [`TransformMapFields`](./transform_map_fields.md) walk carrying the
[`TransformOptional`](./transform_optional.md) marker, targeting `IsOptional`:

```rust
// conceptually:
//   self.transform_map_fields::<TransformOptional, IsOptional>()
```

That walk visits each field of the target's [`HasFields`](./has_fields.md) shape, uses
[`UpdateField`](./update_field.md) to take the field out and learn its current marker, applies the
transform, and writes it back under `IsOptional`. The transform's own impls are what decide the value:
`IsPresent` becomes `Some(value)`, `IsNothing` becomes `None`, and `IsOptional` passes through.

Its mirror image is [`CanFinalizeWithDefault`](./can_finalize_with_default.md), which runs the same walk
with [`TransformMapDefault`](./transform_map_default.md) toward `IsPresent`. **Both capabilities are the
same recursion with a different marker**, which is why the defaulted and optional workflows behave so
symmetrically.

## Gotchas

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**It consumes the builder** and returns a differently-typed one, so the original binding is gone.

**A field already set survives as `Some`.** It is not reset, which is the difference from
[`optional_builder()`](./has_optional_builder.md) and is easy to assume the other way round.

**Converting gives up the compile-time completeness check** for the fields that were still absent — from
here a missing field is an `Err` or a default rather than a compile error.

**It is not reversible.** There is no `from_optional`; getting back to a strict configuration means
finalizing, through [`FinalizeOptional`](./finalize_optional.md) or
[`CanFinalizeWithDefault`](./can_finalize_with_default.md).

**A field whose type is already `Option<T>` becomes `Option<Option<T>>`** in the slot. That is correct —
the outer layer is the builder's presence tracking — and reads confusingly in an error.

## Related constructs

- [`HasOptionalBuilder`](./has_optional_builder.md) — the entry point built from this conversion.
- [`SetOptional`](./set_optional.md) — setting a field once every marker is `IsOptional`.
- [`FinalizeOptional`](./finalize_optional.md) and
  [`CanFinalizeWithDefault`](./can_finalize_with_default.md) — the two endings.
- [`TransformOptional`](./transform_optional.md) — the marker this walk carries.
- [`TransformMapFields`](./transform_map_fields.md) — the walk itself.
- [`IntoBuilder`](./into_builder.md) — how a complete value becomes a builder to convert.
- [`HasBuilder`](./has_builder.md) — the core family this relaxes.
- [`MapType`](./map_type.md) — the `IsOptional` marker.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records, and where relaxing presence
  fits.

## Source

- [`to_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/to_optional.rs)
  — `ToOptional`, `HasOptionalBuilder`, and `TransformOptional`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
