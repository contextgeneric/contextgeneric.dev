---
sidebar_label: 'CanFinalizeWithDefault'
---

# `CanFinalizeWithDefault`

Finalizing a partial record, filling whatever is unset from `Default`.

## Overview

The core [builder family](./has_builder.md) refuses to finalize an incomplete record, which is what
catches a missing field at compile time. For a record where some fields have sensible defaults, that
strictness is the wrong shape. `CanFinalizeWithDefault` relaxes it:

```rust
pub trait CanFinalizeWithDefault {
    type Output;

    fn finalize_with_default(self) -> Self::Output;
}
```

Every field that is not set becomes `Default::default()`, and the result is the concrete struct.

**The strict presence check still runs — it just always passes**, because the fields are re-marked to
`IsPresent` first. That is the layer's whole design in one sentence, and it is why the core
[`FinalizeBuild`](./finalize_build.md) remains the only route from a partial value to a struct.

It is one of two endings for a partial builder, and the choice is made at the call site:

| | a field left unset |
|---|---|
| `CanFinalizeWithDefault` | filled from `Default` |
| [`FinalizeOptional`](./finalize_optional.md) | `Err("field_name")` |

## Usage

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::CanFinalizeWithDefault;
```

`Output` is the concrete struct. It applies to any partial value whose fields can be transformed to
all-present, which covers a core builder from [`builder()`](./has_builder.md) as well as an optional one
from [`optional_builder()`](./has_optional_builder.md).

**Every unset field's type needs `Default`.** A field whose type has none makes the transform
unresolvable, and the error names the missing
[`TransformMap`](./transform_map.md) impl rather than the field.

## Examples

Filling a gap rather than reporting it:

```rust
use cgp::extra::field::impls::{CanFinalizeWithDefault, HasOptionalBuilder, SetOptional};
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

let context = Context::optional_builder()
    .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
    .finalize_with_default();

assert_eq!(context.foo, "foo");
assert_eq!(context.bar, 0);        // defaulted
```

Had that used [`finalize_optional`](./finalize_optional.md), it would have returned `Err("bar")` instead
— which is the point of deferring the choice to the finalize call rather than to the builder.

The one-call form that also copies from a source is
[`CanBuildWithDefault`](./can_build_with_default.md).

## When to use it

**Reach for it when unset fields have meaningful defaults and silence is the right outcome.**

- **[`FinalizeOptional`](./finalize_optional.md)** when absence is an error worth reporting. The two are
  the same builder finalized differently.
- **[`FinalizeBuild`](./finalize_build.md)** — the core builder — when every field is genuinely
  required. A missing field is then a compile error, which is strictly stronger.
- **[`CanBuildWithDefault`](./can_build_with_default.md)** when the set fields come from another record
  rather than from individual calls. It chains the merge and this finalize into one call.
- **A hand-written builder** when the defaults are computed rather than `Default::default()`, or when
  finalizing should validate. This layer models presence and defaulting, and nothing else.

## Under the hood

The whole impl is a transform followed by the ordinary finalize:

```rust
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

Read the bounds together: the first says every field can be re-marked to `IsPresent` by
[`TransformMapDefault`](./transform_map_default.md), and the second says the result is then finalizable.

Its mirror image is [`ToOptional`](./to_optional.md), which runs the same
[`TransformMapFields`](./transform_map_fields.md) walk with
[`TransformOptional`](./transform_optional.md) toward `IsOptional`. **Both capabilities are the same
recursion with a different marker**, which is why the defaulted and optional workflows behave so
symmetrically.

Because the transform targets `IsPresent`, the value handed to `finalize_build` is at exactly the
configuration its single impl requires — so the strict check runs, and cannot fail.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**Every field's type needs `Default`.** A field whose type has none makes the transform unresolvable, and
the error names the missing [`TransformMap`](./transform_map.md) impl rather than the field — which is
the most confusing failure in this layer.

**A defaulted field is silent.** Nothing distinguishes "set to zero" from "left unset" in the result, which
is the trade against [`FinalizeOptional`](./finalize_optional.md).

**It gives up the compile-time completeness check** that the core builder provides.

**It applies to a core builder too**, not only an optional one — an absent `IsNothing` field is
transformed as readily as a `None`.

**It consumes the builder.**

## Related constructs

- [`FinalizeOptional`](./finalize_optional.md) — the other ending, reporting rather than filling.
- [`CanBuildWithDefault`](./can_build_with_default.md) — merge and default in one call.
- [`TransformMapDefault`](./transform_map_default.md) — the marker this drives.
- [`TransformMapFields`](./transform_map_fields.md) — the walk it runs.
- [`FinalizeBuild`](./finalize_build.md) — the strict impl it ultimately calls.
- [`HasOptionalBuilder`](./has_optional_builder.md) and [`ToOptional`](./to_optional.md) — the optional
  entry points.
- [`HasBuilder`](./has_builder.md) — the core family this relaxes.
- [`MapType`](./map_type.md) — the markers the transform moves between.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records, and where relaxing presence
  fits.

## Source

- [`build_default.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/build_default.rs)
  — `CanFinalizeWithDefault`, `CanBuildWithDefault`, and `TransformMapDefault`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
