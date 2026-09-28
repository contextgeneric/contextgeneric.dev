---
title: 'ToOptional — relax an existing builder'
sidebar_label: 'ToOptional'
sidebar_position: 2
description: 'Re-mark every field of a core builder as optional, keeping set fields as Some, so the rest can arrive in any order and the build can end either way.'
---

# `ToOptional`

Re-marking every field of an existing builder as optional.

## Overview

[`HasOptionalBuilder`](./has_optional_builder.md) starts an all-optional builder from nothing.
`ToOptional` is the conversion for the case where you already hold a partial value and want to relax it.
Every field is re-marked to `IsOptional`, whatever state it was in (a set field becomes `Some(value)`,
an absent one becomes `None`), and from there the value behaves like any other optional builder:
[`SetOptional`](./set_optional.md) applies, and either
[`FinalizeOptional`](./finalize_optional.md) or [`CanFinalizeWithDefault`](./can_finalize_with_default.md)
ends it.

**It is the conversion `optional_builder()` is built from**, which is the shortest way to describe both:
one starts fresh and converts, the other converts something you already have.

## Definition

`ToOptional` carries the re-marked builder type as an associated `Output` and converts a partial value
into it:

```rust
pub trait ToOptional {
    type Output;

    fn to_optional(self) -> Self::Output;
}
```

`Self` is any partial builder, and `Output` is the same partial companion type with every marker set to
`IsOptional`. `to_optional` takes `self`, consuming the builder and returning the re-marked one. The
trait has no [supertrait](/docs/reference/glossary#supertrait); it is a blanket impl over
[`TransformMapFields`](../type-level/transform_map_fields.md), so any partial value whose fields can be
re-marked gains it, with nothing to implement by hand. It is not in the prelude; the optional-field
layer lives in `cgp-field-extra`.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::ToOptional;
```

You call it on a partial value, as `builder.to_optional()`.

## Examples

Relaxing a partly filled core builder, and then a complete value:

```rust
use cgp::prelude::*;
use cgp::extra::field::impls::{FinalizeOptional, SetOptional, ToOptional};

#[derive(Debug, PartialEq, CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

pub fn demo() {
    // `foo` is set before the conversion and survives it as `Some`.
    let context = Context::builder()
        .build_field(PhantomData::<Symbol!("foo")>, "foo".to_owned())
        .to_optional()
        .set(PhantomData::<Symbol!("bar")>, 42)
        .finalize_optional()
        .unwrap();
    assert_eq!(context.foo, "foo");

    // Every field of a complete value becomes `Some`, so one can be overwritten.
    let context = context
        .into_builder()
        .to_optional()
        .set(PhantomData::<Symbol!("bar")>, 7)
        .finalize_optional()
        .unwrap();
    assert_eq!(context.bar, 7);
}
```

The already-set `foo` survives the first conversion as `Some("foo")`, which distinguishes this from
starting over with [`optional_builder()`](./has_optional_builder.md). The second conversion starts
from [`into_builder`](../builder/into_builder.md), so every field is `Some` and `set` overwrites
`bar`.

## When to use it

**Reach for it when a builder already exists and its strictness has become the wrong shape.**

- **[`HasOptionalBuilder`](./has_optional_builder.md)** when nothing has been built yet.
  `optional_builder()` is `builder().to_optional()` and reads better.
- **`ToOptional`** when a core builder is partly filled, because a routine handed it to you, or because
  the first few fields were known and the rest are not.
- **[`IntoBuilder`](../builder/into_builder.md) then `to_optional`** to relax a complete value, typically before
  overwriting some of its fields.
- **Stay on the [core builder](../builder/has_builder.md)** if the remaining fields are known and set once. The
  conversion costs the compile-time completeness check.

## Under the hood

`ToOptional` is a blanket impl over a [`TransformMapFields`](../type-level/transform_map_fields.md)
walk carrying the [`TransformOptional`](./transform_optional.md) marker, targeting `IsOptional`:

```rust
impl<Context> ToOptional for Context
where
    Context: TransformMapFields<TransformOptional, IsOptional>,
{
    type Output = Context::Output;

    fn to_optional(self) -> Self::Output {
        self.transform_map_fields()
    }
}
```

That walk visits each field of the target's [`HasFields`](../shape/has_fields.md) shape, uses
[`UpdateField`](../builder/update_field.md) to take the field out and learn its current marker,
applies the transform, and writes it back under `IsOptional`. The transform's impls decide the
value: `IsPresent` becomes `Some(value)` and `IsNothing` becomes `None`. **There is no impl from
`IsOptional`**, so a builder that is already optional cannot be converted again, as [Common
Mistakes](#common-mistakes) shows.

Its mirror image is [`CanFinalizeWithDefault`](./can_finalize_with_default.md), which runs the same walk
with [`TransformMapDefault`](./transform_map_default.md) toward `IsPresent`. **Both operations are the
same recursion with a different marker**, which is why the defaulted and optional workflows behave so
symmetrically.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**It consumes the builder** and returns a differently-typed one, so the original binding is gone.

**A field already set survives as `Some`.** It is not reset, which is the difference from
[`optional_builder()`](./has_optional_builder.md) and is easy to assume the other way round.

**Converting gives up the compile-time completeness check** for the fields that were still absent. From
here a missing field is an `Err` or a default rather than a compile error.

**It does not apply to a builder that is already optional.** Converting the result of
`optional_builder()` again:

```rust
let _ = Context::optional_builder().to_optional();
```

finds no [`TransformOptional`](./transform_optional.md) conversion from `IsOptional`, and fails
with:

```text
error[E0599]: the method `to_optional` exists for struct `__PartialContext<IsOptional, IsOptional>`, but its trait bounds were not satisfied
...
   = note: the following trait bounds were not satisfied:
           `__PartialContext<IsOptional, IsOptional>: TransformMapFields<TransformOptional, IsOptional>`
```

**It is not reversible.** There is no `from_optional`; getting back to a strict configuration means
finalizing, through [`FinalizeOptional`](./finalize_optional.md) or
[`CanFinalizeWithDefault`](./can_finalize_with_default.md).

**A field whose type is already `Option<T>` becomes `Option<Option<T>>`** in the slot. That is correct
(the outer layer is the builder's presence tracking) and reads confusingly in an error.

## Related constructs

- [`HasOptionalBuilder`](./has_optional_builder.md): the entry point built from this conversion.
- [`SetOptional`](./set_optional.md): setting a field once every marker is `IsOptional`.
- [`FinalizeOptional`](./finalize_optional.md) and
  [`CanFinalizeWithDefault`](./can_finalize_with_default.md): the two endings.
- [`TransformOptional`](./transform_optional.md): the marker this walk carries.
- [`TransformMapFields`](../type-level/transform_map_fields.md): the walk itself.
- [`IntoBuilder`](../builder/into_builder.md): how a complete value becomes a builder to convert.
- [`HasBuilder`](../builder/has_builder.md): the core family this relaxes.
- [`MapType`](../type-level/map_type.md): the `IsOptional` marker.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records, and where relaxing presence
  fits.

## Source

- [`to_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/to_optional.rs):
  `ToOptional`, `HasOptionalBuilder`, and `TransformOptional`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
