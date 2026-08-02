---
sidebar_label: 'TransformMapDefault'
---

# `TransformMapDefault`

The transform marker that makes every field present, defaulting whatever is not.

## What it's for

[`CanFinalizeWithDefault`](./can_finalize_with_default.md) works by re-marking every field of a partial
record to `IsPresent` and then calling the ordinary [`finalize_build`](./finalize_build.md).
`TransformMapDefault` is the marker that carries the per-field conversions making that possible.

It is a zero-sized type implementing [`TransformMap`](./transform_map.md) three times, once per state a
field might currently be in — and reading those three impls is the shortest description of what the
defaulting layer does:

| a field currently | becomes |
|---|---|
| `IsPresent` | itself, unchanged |
| `IsNothing` | `Default::default()` |
| `IsOptional` | its contents, or the default if `None` |

All three target `IsPresent`, which is exactly the configuration
[`FinalizeBuild`](./finalize_build.md) accepts.

**You name it only when extending the layer.** Using the defaulting workflow means calling
[`finalize_with_default`](./can_finalize_with_default.md) or
[`build_with_default`](./can_build_with_default.md), with no marker in sight.

## Using it

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::TransformMapDefault;
```

There is nothing to call. The marker exists to be named in a
[`TransformMapFields`](./transform_map_fields.md) bound, which is where a capability says *which*
conversion it drives:

```rust
Builder: TransformMapFields<TransformMapDefault, IsPresent>
```

**Every field's type must implement `Default`** for the two filling impls to apply — which is where the
defaulting layer's one real requirement comes from.

## Examples

You meet it in a bound rather than at a call site. This is
[`CanFinalizeWithDefault`](./can_finalize_with_default.md)'s whole impl:

```rust
use cgp::core::field::traits::TransformMapFields;
use cgp::extra::field::impls::TransformMapDefault;
use cgp::prelude::*;

impl<Builder, Output> CanFinalizeWithDefault for Builder
where
    Builder: TransformMapFields<TransformMapDefault, IsPresent>,
    Builder::Output: FinalizeBuild<Target = Output>,
{
    type Output = Output;

    fn finalize_with_default(self) -> Output {
        self.transform_map_fields().finalize_build()
    }
}
```

The marker is the first type argument, and swapping it for
[`TransformOptional`](./transform_optional.md) — with `IsOptional` as the target — is exactly how
[`ToOptional`](./to_optional.md) is written. **Same recursion, different marker.**

Writing a marker of your own follows the same three-impl shape; the
[`TransformMap`](./transform_map.md#examples) page shows one.

## When to reach for it, and when not

**Name it only when writing a capability that drives the defaulting conversion.** Everything else is
served by the two capabilities already built on it.

- **[`CanFinalizeWithDefault`](./can_finalize_with_default.md)** to finalize a partial record, defaulting
  gaps.
- **[`CanBuildWithDefault`](./can_build_with_default.md)** to merge from a source and default the rest, in
  one call.
- **Write your own [`TransformMap`](./transform_map.md) marker** for a conversion these do not cover — one
  that validates, logs, or fills from something other than `Default`.
- **[`TransformOptional`](./transform_optional.md)** is its counterpart, targeting `IsOptional` instead.

## Under the hood

:::note

### Advanced

This section shows the three impls, which are the layer's actual content.

:::

The marker is zero-sized and carries no data. What it carries is three
[`TransformMap`](./transform_map.md) impls, distinguished by their source marker:

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

Note that each argument type is the *source* marker's projection — `T`, then `()`, then `Option<T>` —
which is what keeps the three from overlapping. And note that only two carry the `T: Default` bound: a
field already present needs no default, which is why a fully-set builder can be finalized this way
regardless of its field types.

Three impls are needed rather than one because
[`TransformMapFields`](./transform_map_fields.md#under-the-hood) resolves a conversion per field from
whatever state that field is in, and a record may hold fields in all three states at once.

## Gotchas

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**Two of its three impls require `Default`.** A field whose type has none makes the walk unresolvable the
moment that field is unset, and the error names the missing `TransformMap` impl rather than the field.

**It always targets `IsPresent`.** It cannot be used to reach any other configuration; that is
[`TransformOptional`](./transform_optional.md)'s job.

**It is a marker, not a capability.** There is no method to call and nothing to wire — it is named in a
bound.

**A defaulted field is indistinguishable from one set to the default value** in the result, which is the
trade against [`FinalizeOptional`](./finalize_optional.md).

## Related constructs

- [`CanFinalizeWithDefault`](./can_finalize_with_default.md) — the capability built directly on it.
- [`CanBuildWithDefault`](./can_build_with_default.md) — merge plus that finalize.
- [`TransformOptional`](./transform_optional.md) — the counterpart marker, targeting `IsOptional`.
- [`TransformMap`](./transform_map.md) — the trait it implements three times.
- [`TransformMapFields`](./transform_map_fields.md) — the walk that applies it.
- [`MapType`](./map_type.md) — the markers it converts between.
- [`FinalizeBuild`](./finalize_build.md) — what an all-`IsPresent` result is accepted by.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records and how their states change.

## Source

- [`build_default.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/build_default.rs)
  — `TransformMapDefault` and the capabilities built on it

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
