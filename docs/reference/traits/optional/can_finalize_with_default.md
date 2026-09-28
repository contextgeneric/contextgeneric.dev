---
title: 'CanFinalizeWithDefault — fill gaps, then build'
sidebar_label: 'CanFinalizeWithDefault'
sidebar_position: 5
description: 'End a build by filling every unset field from Default, for an optional builder or a core one, then finalizing through the strict impl.'
---

# `CanFinalizeWithDefault`

Finalizing a [partial record](/docs/reference/glossary#partial-record), filling whatever is unset from `Default`.

## Overview

The core [builder family](../builder/has_builder.md) refuses to finalize an incomplete record, which is what
catches a missing field at compile time. For a record where some fields have sensible defaults, that
strictness is the wrong shape. `CanFinalizeWithDefault` relaxes it: every field that is not set becomes
`Default::default()`, and the result is the concrete struct.

**The strict presence check still runs, and it always passes**, because the fields are re-marked to
`IsPresent` first. That is the layer's whole design in one sentence, and it is why the core
[`FinalizeBuild`](../builder/finalize_build.md) remains the only route from a partial value to a struct.

It is one of two endings for a partial builder, and the choice is made at the call site:

| | a field left unset |
|---|---|
| `CanFinalizeWithDefault` | filled from `Default` |
| [`FinalizeOptional`](./finalize_optional.md) | `Err("field_name")` |

## Definition

`CanFinalizeWithDefault` carries the finished struct as an associated `Output` and finalizes a partial
record into it:

```rust
pub trait CanFinalizeWithDefault {
    type Output;

    fn finalize_with_default(self) -> Self::Output;
}
```

`Self` is any partial builder whose fields can be re-marked to all-present, which covers a core builder
from [`builder()`](../builder/has_builder.md) as well as an optional one from
[`optional_builder()`](./has_optional_builder.md). `Output` is the concrete struct being built.
`finalize_with_default` takes `self`, consuming the builder, and returns that struct with every unset
field filled from `Default`. The trait has no [supertrait](/docs/reference/glossary#supertrait); it is a blanket impl over the
transform-then-finalize shown in [*Under the hood*](#under-the-hood). It is not in the prelude; import it
from `cgp-field-extra`.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::CanFinalizeWithDefault;
```

**A field's type needs `Default` unless the field is `IsPresent`.** On a core builder that means
every unset field. On an optional builder it means every field, set or not, because a set field is
still `IsOptional` and only [`TransformMapDefault`](./transform_map_default.md)'s `IsPresent` impl
is free of the bound. [Common Mistakes](#common-mistakes) shows what rustc reports.

## Examples

Filling a gap rather than reporting it, from an optional builder and from a core one:

```rust
use cgp::prelude::*;
use cgp::extra::field::impls::{CanFinalizeWithDefault, HasOptionalBuilder, SetOptional};

#[derive(Debug, PartialEq, CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

pub fn demo() {
    let from_optional = Context::optional_builder()
        .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
        .finalize_with_default();
    assert_eq!(from_optional.bar, 0);

    // A core builder works too: its absent field is `IsNothing` rather than `None`.
    let from_core = Context::builder()
        .build_field(PhantomData::<Symbol!("foo")>, "foo".to_owned())
        .finalize_with_default();
    assert_eq!(from_core, from_optional);
}
```

Had the first used [`finalize_optional`](./finalize_optional.md), it would have returned
`Err("bar")` instead, which is the point of deferring the choice to the finalize call. The core
builder reaches the same value, because [`TransformMapDefault`](./transform_map_default.md) converts
an `IsNothing` field as readily as a `None`. The one-call form that also copies from a source is
[`CanBuildWithDefault`](./can_build_with_default.md).

## When to use it

**Reach for it when unset fields have meaningful defaults and silence is the right outcome.**

- **[`FinalizeOptional`](./finalize_optional.md)** when absence is an error worth reporting. The two are
  the same builder finalized differently.
- **[`FinalizeBuild`](../builder/finalize_build.md)** (the core builder) when every field is genuinely
  required. A missing field is then a compile error, which is strictly stronger.
- **[`CanBuildWithDefault`](./can_build_with_default.md)** when the set fields come from another record
  rather than from individual calls. It chains the merge and this finalize into one call.
- **A hand-written builder** when the defaults are computed rather than `Default::default()`, or when
  finalizing should validate. This layer models presence and defaulting, and nothing else.

## Under the hood

The trait's one impl is a transform followed by the ordinary finalize:

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
[`TransformMapDefault`](./transform_map_default.md), and the second says the result is then
finalizable. The `Default` requirement lives inside the first, on the marker's conversions from
`IsNothing` and `IsOptional`.

Its mirror image is [`ToOptional`](./to_optional.md), which runs the same
[`TransformMapFields`](../type-level/transform_map_fields.md) walk with
[`TransformOptional`](./transform_optional.md) toward `IsOptional`. **Both operations are the same
recursion with a different marker**, which is why the defaulted and optional workflows behave so
symmetrically.

Because the transform targets `IsPresent`, the value handed to `finalize_build` is at exactly the
configuration its single impl requires, so the strict check runs, and cannot fail.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**A field whose type has no `Default` is reported without being named.** On a core builder with
`port` left unset, where `Port` has no `Default`:

```rust
pub struct Port(pub u16);

#[derive(CgpData)]
pub struct Server {
    pub host: String,
    pub port: Port,
}

let _ = Server::builder()
    .build_field(PhantomData::<Symbol!("host")>, "localhost".to_owned())
    .finalize_with_default();
```

method resolution fails, and the notes stop at the walk rather than reaching `Port`:

```text
error[E0599]: the method `finalize_with_default` exists for struct `__PartialServer<IsPresent, IsNothing>`, but its trait bounds were not satisfied
...
   = note: the following trait bounds were not satisfied:
           `__PartialServer<IsPresent, IsNothing>: TransformMapFields<TransformMapDefault, IsPresent>`
           which is required by `__PartialServer<IsPresent, IsNothing>: CanFinalizeWithDefault`
```

Neither `port` nor `Default` appears, which makes this the most confusing failure in the layer. The
same bound reached through a `where` clause rather than a method call, as in
[`build_with_default`](./can_build_with_default.md#common-mistakes), does name `Port: Default`.

**On an optional builder, setting the field does not help.** Setting both fields of the same
`Server` on an optional builder:

```rust
let _ = Server::optional_builder()
    .set(PhantomData::<Symbol!("host")>, "localhost".to_owned())
    .set(PhantomData::<Symbol!("port")>, Port(8080))
    .finalize_with_default();
```

fails the same way, on `__PartialServer<IsOptional, IsOptional>`, because a set field is still
`IsOptional` and its conversion requires `Default`. A core builder with both fields set finalizes
this way regardless, and [`finalize_optional`](./finalize_optional.md) needs no `Default` at all.

**A defaulted field is silent.** Nothing distinguishes "set to zero" from "left unset" in the result, which
is the trade against [`FinalizeOptional`](./finalize_optional.md).

**It gives up the compile-time completeness check** that the core builder provides.

**It applies to a core builder too**, not only an optional one: an absent `IsNothing` field is
transformed as readily as a `None`, as the [example](#examples) shows.

**It consumes the builder.**

## Related constructs

- [`FinalizeOptional`](./finalize_optional.md): the other ending, reporting rather than filling.
- [`CanBuildWithDefault`](./can_build_with_default.md): merge and default in one call.
- [`TransformMapDefault`](./transform_map_default.md): the marker this drives.
- [`TransformMapFields`](../type-level/transform_map_fields.md): the walk it runs.
- [`FinalizeBuild`](../builder/finalize_build.md): the strict impl it ultimately calls.
- [`HasOptionalBuilder`](./has_optional_builder.md) and [`ToOptional`](./to_optional.md): the optional
  entry points.
- [`HasBuilder`](../builder/has_builder.md): the core family this relaxes.
- [`MapType`](../type-level/map_type.md): the markers the transform moves between.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records, and where relaxing presence
  fits.

## Source

- [`build_default.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/build_default.rs):
  `CanFinalizeWithDefault`, `CanBuildWithDefault`, and `TransformMapDefault`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
