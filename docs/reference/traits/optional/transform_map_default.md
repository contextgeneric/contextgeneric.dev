---
title: 'TransformMapDefault — fill the unset fields'
sidebar_label: 'TransformMapDefault'
sidebar_position: 7
description: 'The transform marker behind the defaulting finalize: three per-field conversions that leave every field present, filling gaps from Default.'
---

# `TransformMapDefault`

The transform marker that makes every field present, defaulting whatever is not.

:::info

### Generated machinery

**You are not expected to name `TransformMapDefault` directly.** It is the
marker [`CanFinalizeWithDefault`](./can_finalize_with_default.md) and
[`CanBuildWithDefault`](./can_build_with_default.md) drive, and you write a call to one of those.
This page explains the three conversions behind them, which is also the model to copy if you write a
transform of your own.

:::

## Overview

[`CanFinalizeWithDefault`](./can_finalize_with_default.md) works by re-marking every field of a partial
record to `IsPresent` and then calling the ordinary [`finalize_build`](../builder/finalize_build.md).
`TransformMapDefault` is the marker that carries the per-field conversions making that possible.

**You name it only when extending the layer.** Using the defaulting workflow means calling
[`finalize_with_default`](./can_finalize_with_default.md) or
[`build_with_default`](./can_build_with_default.md), without naming the marker.

## Definition

`TransformMapDefault` is a zero-sized marker type:

```rust
pub struct TransformMapDefault;
```

It stores nothing. It becomes a transform by implementing
[`TransformMap`](../type-level/transform_map.md) once for each state a field might currently be in,
every impl targeting `IsPresent`:

| a field currently | becomes |
|---|---|
| `IsPresent` | itself, unchanged |
| `IsNothing` | `Default::default()` |
| `IsOptional` | its contents, or the default if `None` |

Because all three impls target `IsPresent`, applying the marker across a record leaves every field
present, the configuration [`FinalizeBuild`](../builder/finalize_build.md) accepts. The impl bodies are
in [*Under the hood*](#under-the-hood). The marker is not in the prelude; import it from
`cgp-field-extra`.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::TransformMapDefault;
```

There is nothing to call. The marker exists to be named in a
[`TransformMapFields`](../type-level/transform_map_fields.md) bound, which is where an operation says *which*
conversion it drives:

```rust
Builder: TransformMapFields<TransformMapDefault, IsPresent>
```

**A field's type must implement `Default` unless the field is already `IsPresent`**, since two of
the three impls carry the bound. That is where the defaulting layer's one real requirement comes
from.

## Examples

A generic function that drives the defaulting conversion across any builder, and a core builder
passed through it:

```rust
use cgp::prelude::*;
use cgp::core::field::traits::TransformMapFields;
use cgp::extra::field::impls::TransformMapDefault;

#[derive(Debug, PartialEq, CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

pub fn fill_defaults<Builder>(builder: Builder) -> Builder::Output
where
    Builder: TransformMapFields<TransformMapDefault, IsPresent>,
{
    builder.transform_map_fields()
}

pub fn demo() {
    let full = fill_defaults(
        Context::builder().build_field(PhantomData::<Symbol!("foo")>, "foo".to_owned()),
    );
    let context = full.finalize_build();
    assert_eq!(context.bar, 0);
}
```

`fill_defaults` is [`CanFinalizeWithDefault`](./can_finalize_with_default.md) without its last step:
the bound names the marker and the target, and the result is at the all-present configuration, so
`finalize_build` resolves on it. Swapping the marker for
[`TransformOptional`](./transform_optional.md), with `IsOptional` as the target, is how
[`ToOptional`](./to_optional.md) is written.

## When to use it

**Name it only when writing an operation that drives the defaulting conversion.** Everything else is
served by the two operations already built on it.

- **[`CanFinalizeWithDefault`](./can_finalize_with_default.md)** to finalize a [partial record](/docs/reference/glossary#partial-record), defaulting
  gaps.
- **[`CanBuildWithDefault`](./can_build_with_default.md)** to merge from a source and default the rest, in
  one call.
- **Write your own [`TransformMap`](../type-level/transform_map.md) marker** for a conversion these do not cover: one
  that validates, logs, or fills from something other than `Default`.
- **[`TransformOptional`](./transform_optional.md)** is its counterpart, targeting `IsOptional` instead.

## Under the hood

The marker is zero-sized, so it stores nothing. What it carries is three
[`TransformMap`](../type-level/transform_map.md) impls, distinguished by their source marker:

```rust
impl<T> TransformMap<IsPresent, IsPresent, T> for TransformMapDefault {
    fn transform_mapped(value: T) -> T {
        value
    }
}

impl<T: Default> TransformMap<IsNothing, IsPresent, T> for TransformMapDefault {
    fn transform_mapped(_value: ()) -> T {
        T::default()
    }
}

impl<T: Default> TransformMap<IsOptional, IsPresent, T> for TransformMapDefault {
    fn transform_mapped(value: Option<T>) -> T {
        value.unwrap_or_default()
    }
}
```

Note that each argument type is the *source* marker's projection (`T`, then `()`, then `Option<T>`),
which keeps the three from overlapping. And note that only two carry the `T: Default` bound: a field
already present does not need a default, which is why a fully-set *core* builder can be finalized
this way regardless of its field types. An optional builder cannot: its set fields are still
`IsOptional`, so the third impl, and its bound, applies to every field.

Three impls are needed rather than one because
[`TransformMapFields`](../type-level/transform_map_fields.md#under-the-hood) resolves a conversion per field from
whatever state that field is in, and a record may hold fields in all three states at once.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**Two of its three impls require `Default`.** A field whose type has none makes the walk
unresolvable whenever that field is not `IsPresent`: unset on a core builder, and always on an
optional one. Through a method call such as `finalize_with_default`, the error names the unmet
`TransformMapFields<TransformMapDefault, IsPresent>` bound rather than the field;
[`CanFinalizeWithDefault`](./can_finalize_with_default.md#common-mistakes) shows it.

**It always targets `IsPresent`.** It cannot be used to reach any other configuration; that is
[`TransformOptional`](./transform_optional.md)'s job.

**It is a marker, not an operation.** It lacks a method to call and needs nothing wired. It is named
in a bound.

**A defaulted field is indistinguishable from one set to the default value** in the result, which is the
trade against [`FinalizeOptional`](./finalize_optional.md).

## Related constructs

- [`CanFinalizeWithDefault`](./can_finalize_with_default.md): the operation built directly on it.
- [`CanBuildWithDefault`](./can_build_with_default.md): merge plus that finalize.
- [`TransformOptional`](./transform_optional.md): the counterpart marker, targeting `IsOptional`.
- [`TransformMap`](../type-level/transform_map.md): the trait it implements three times.
- [`TransformMapFields`](../type-level/transform_map_fields.md): the walk that applies it.
- [`MapType`](../type-level/map_type.md): the markers it converts between.
- [`FinalizeBuild`](../builder/finalize_build.md): what an all-`IsPresent` result is accepted by.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and how their states change.

## Source

- [`build_default.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/build_default.rs):
  `TransformMapDefault` and the operations built on it

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
