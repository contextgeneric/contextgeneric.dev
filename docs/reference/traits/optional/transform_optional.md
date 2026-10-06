---
title: 'TransformOptional — make every field optional'
sidebar_label: 'TransformOptional'
sidebar_position: 8
description: 'The transform marker behind ToOptional: two per-field conversions that wrap a present field in Some and turn an absent one into None.'
---

# `TransformOptional`

The transform marker that makes every field optional.

:::info

### Generated machinery

**You are not expected to name `TransformOptional` directly.** It is the marker
[`ToOptional`](./to_optional.md) and [`HasOptionalBuilder`](./has_optional_builder.md) drive, and
you write a call to one of those. This page explains the conversion behind them, and why it needs
less of a field's type than its defaulting counterpart does. The one case for naming it is writing
an operation of your own that drives the optional conversion.

:::

## Overview

[`ToOptional`](./to_optional.md) re-marks every field of a [partial record](/docs/reference/glossary#partial-record) to `IsOptional`, so it can then
be set freely and finalized either way. `TransformOptional` is the marker carrying the per-field
conversions that make that possible.

**It is the exact mirror of [`TransformMapDefault`](./transform_map_default.md)**, which targets
`IsPresent` instead: same walk, different destination. That symmetry is why the defaulted and optional
workflows behave so alike.

**You name it only when extending the layer.** Using the optional workflow means calling
[`optional_builder()`](./has_optional_builder.md) or [`to_optional()`](./to_optional.md), without
naming the marker.

## Definition

`TransformOptional` is a zero-sized marker type:

```rust
pub struct TransformOptional;
```

It stores nothing. It becomes a transform by implementing
[`TransformMap`](../type-level/transform_map.md) once for each state a core builder's field can be
in, both impls targeting `IsOptional`:

| a field currently | becomes |
|---|---|
| `IsPresent` | `Some(value)` |
| `IsNothing` | `None` |

Because both impls target `IsOptional`, applying the marker across a record leaves every field
optional, the configuration [`SetOptional`](./set_optional.md) requires. Unlike its defaulting
counterpart it needs nothing of the field types, since wrapping a value in `Some` and producing
`None` do not require `Default`. The impl bodies are in [*Under the hood*](#under-the-hood). The
marker is not in the prelude; import it from `cgp-field-extra`.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::TransformOptional;
```

There is nothing to call. The marker is named in a
[`TransformMapFields`](../type-level/transform_map_fields.md) bound, which is where an operation says which
conversion it drives:

```rust
Builder: TransformMapFields<TransformOptional, IsOptional>
```

Unlike its defaulting counterpart, **it requires nothing of the field types**: wrapping a value in
`Some` and producing `None` need neither `Default` nor any other bound, which is why the optional
path applies to records the defaulting path does not.

## Examples

A generic function that drives the optional conversion across any builder, and a partly built core
builder passed through it:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::IsOptional;
use cgp::core::field::traits::TransformMapFields;
use cgp::extra::field::impls::{FinalizeOptional, SetOptional, TransformOptional};

#[derive(Debug, PartialEq, CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

pub fn relax<Builder>(builder: Builder) -> Builder::Output
where
    Builder: TransformMapFields<TransformOptional, IsOptional>,
{
    builder.transform_map_fields()
}

pub fn demo() {
    let optional =
        relax(Context::builder().build_field(PhantomData::<Symbol!("foo")>, "foo".to_owned()));
    let context = optional
        .set(PhantomData::<Symbol!("bar")>, 42)
        .finalize_optional()
        .unwrap();
    assert_eq!(context.foo, "foo");
}
```

`relax` is [`ToOptional`](./to_optional.md)'s whole impl written as a function. Swap the marker for
[`TransformMapDefault`](./transform_map_default.md) and the target for `IsPresent`, add a
[`finalize_build`](../builder/finalize_build.md), and you have
[`CanFinalizeWithDefault`](./can_finalize_with_default.md).

## When to use it

**Name it only when writing an operation that drives the optional conversion.**

- **[`ToOptional`](./to_optional.md)** to relax an existing partial value.
- **[`HasOptionalBuilder`](./has_optional_builder.md)** to start an all-optional builder, which is
  `builder().to_optional()`.
- **[`TransformMapDefault`](./transform_map_default.md)** when the destination is `IsPresent` rather than
  `IsOptional`.
- **Write your own [`TransformMap`](../type-level/transform_map.md) marker** for a conversion neither covers.

## Under the hood

The marker is zero-sized and carries two [`TransformMap`](../type-level/transform_map.md) impls,
distinguished by their source marker, each targeting `IsOptional`:

```rust
impl<T> TransformMap<IsPresent, IsOptional, T> for TransformOptional {
    fn transform_mapped(value: T) -> Option<T> {
        Some(value)
    }
}

impl<T> TransformMap<IsNothing, IsOptional, T> for TransformOptional {
    fn transform_mapped(_value: ()) -> Option<T> {
        None
    }
}
```

Note what is absent: **a `Default` bound.** Its counterpart
[`TransformMapDefault`](./transform_map_default.md) needs one on two of its three impls, because
filling an absent field means producing a value from nothing. Producing `None` does not, so this
conversion applies to every field type. Also absent is an impl from `IsOptional`, which
[`TransformMapDefault`](./transform_map_default.md) has: a field that is already optional has no
conversion here, so the marker cannot be applied to an optional builder.

[`TransformMapFields`](../type-level/transform_map_fields.md#under-the-hood) applies it, visiting each field
of the target's [`HasFields`](../shape/has_fields.md) [shape](/docs/reference/glossary#shape) and using
[`UpdateField`](../builder/update_field.md) twice per field: once to take the value out and learn its marker,
once to write it back under `IsOptional`.

The result is a partial value at a configuration the core builder never reaches on its own, which is what
makes [`SetOptional`](./set_optional.md) resolve and the strict
[`FinalizeBuild`](../builder/finalize_build.md) not.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**It always targets `IsOptional`.** Reaching `IsPresent` is
[`TransformMapDefault`](./transform_map_default.md)'s job.

**It lacks a conversion from `IsOptional`**, so a builder that is already optional cannot be relaxed
again; [`ToOptional`](./to_optional.md#common-mistakes) shows the error.

**It is a marker, not an operation.** It lacks a method and needs nothing wired. It is named in a
bound.

**It requires nothing of the field types**, which is the one place the two markers genuinely differ in
what they can be applied to.

**Converting gives up the compile-time completeness check**, since every field becomes optional whether
it was set or not.

**A field whose type is already `Option<T>` becomes `Option<Option<T>>`.** The outer layer is the
builder's presence tracking, and the two are easy to conflate in an error.

## Related constructs

- [`ToOptional`](./to_optional.md): the conversion built directly on it.
- [`HasOptionalBuilder`](./has_optional_builder.md): the entry point built on that.
- [`TransformMapDefault`](./transform_map_default.md): the counterpart marker, targeting `IsPresent`.
- [`TransformMap`](../type-level/transform_map.md): the trait it implements.
- [`TransformMapFields`](../type-level/transform_map_fields.md): the walk that applies it.
- [`SetOptional`](./set_optional.md): what the resulting configuration makes available.
- [`MapType`](../type-level/map_type.md): the `IsOptional` marker it targets.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and how their states change.

## Source

- [`to_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/to_optional.rs):
  `TransformOptional`, `ToOptional`, and `HasOptionalBuilder`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
