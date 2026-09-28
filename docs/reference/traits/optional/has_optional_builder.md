---
title: 'HasOptionalBuilder — an all-optional builder'
sidebar_label: 'HasOptionalBuilder'
sidebar_position: 1
description: 'Start a builder whose every field is an Option, set in any order and as often as you like, with the choice of ending deferred to the finalize call.'
---

# `HasOptionalBuilder`

Starting a builder in which every field is already optional.

## Overview

The core [builder family](../builder/has_builder.md) is deliberately strict: a field is set exactly once, and a
[partial record](/docs/reference/glossary#partial-record) becomes its concrete struct only when **every** field is present. That strictness is what
catches a missing field at compile time, and it is too rigid for a record whose fields arrive in an
unpredictable order, or more than once.

`HasOptionalBuilder` is the entry point that relaxes it. It hands back a partial value with every field
marked `IsOptional` rather than `IsNothing`, so each slot holds an `Option` and **absence becomes a
runtime condition instead of a type-level one.** Fields can then be set freely with
[`SetOptional`](./set_optional.md), in any order and as many times as you like, and the choice of how to
end is deferred to the finalize call.

It stands to [`HasBuilder`](../builder/has_builder.md) as the optional layer stands to the core one: same
machinery, different starting marker.

## Definition

`HasOptionalBuilder` carries the partial builder type as an associated `Builder` and hands back an empty
one:

```rust
pub trait HasOptionalBuilder {
    type Builder;

    fn optional_builder() -> Self::Builder;
}
```

`Builder` names the partial companion type the derive generated, with every field marked `IsOptional`.
It is the same companion the core builder produces, at a configuration the core builder never starts
from. `optional_builder` is an associated function with no receiver, so a caller writes
`Context::optional_builder()`. The trait carries no [supertrait](/docs/reference/glossary#supertrait), and it is implemented as a blanket impl
over the core builder machinery, so any record that derives the builder gains it. It is not in the
prelude; the whole optional-field layer lives in `cgp-field-extra`.

## Usage

**It is not in the prelude.** Nothing in the optional-field layer is. Import it from
`cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::HasOptionalBuilder;
```

The impls are blanket ones over the core builder machinery, so any record deriving
[`#[derive(BuildField)]`](../../derives/derive_build_field.md) (or
[`#[derive(CgpData)]`](../../derives/derive_cgp_data.md)) gets this for free. There is no separate derive.

**Two endings are available**, and picking between them at the call site is the layer's real payoff:
[`FinalizeOptional`](./finalize_optional.md) requires every field and reports a missing one, while
[`CanFinalizeWithDefault`](./can_finalize_with_default.md) fills unset fields from `Default`.

## Examples

An all-optional builder, set in any order and finished by each of the two endings:

```rust
use cgp::prelude::*;
use cgp::extra::field::impls::{
    CanFinalizeWithDefault, FinalizeOptional, HasOptionalBuilder, SetOptional,
};

#[derive(Debug, PartialEq, CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

pub fn demo() {
    let context = Context::optional_builder()
        .set(PhantomData::<Symbol!("bar")>, 42)
        .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
        .finalize_optional();
    assert_eq!(
        context,
        Ok(Context {
            foo: "foo".to_owned(),
            bar: 42,
        })
    );

    // The same kind of builder, with `bar` left unset, finalized each way.
    let defaulted = Context::optional_builder()
        .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
        .finalize_with_default();
    assert_eq!(defaulted.bar, 0);

    let missing = Context::optional_builder()
        .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
        .finalize_optional();
    assert_eq!(missing, Err("bar"));
}
```

The fields are set in reverse declaration order, which the core builder also allows; what it does
not allow is ending an incomplete build any way but a compile error. Here the same unfinished
builder either fills `bar` with `0` or reports it by name, chosen at the finalize call.

## When to use it

**Reach for it when fields must be settable in any order and more than once**, which the core
`build_field` cannot do, and stay on the core builder otherwise.

- **[`HasBuilder`](../builder/has_builder.md)** when every field is genuinely required and set once. You lose the
  compile-time completeness check by moving here, and that check is the reason the builder family
  exists.
- **`HasOptionalBuilder`** when the fields arrive unpredictably: parsed configuration, accumulated
  defaults, or a value assembled across several passes.
- **[`ToOptional`](./to_optional.md)** when you already hold a core builder and want to convert it rather
  than start fresh.
- **A hand-written builder** when the *logic* is the point: validation at finalize, interdependent
  fields, or computed defaults that are not `Default::default()`. This layer models presence and defaulting,
  and nothing else.

## Under the hood

The trait's one impl is a blanket impl over the core builder and [`ToOptional`](./to_optional.md):

```rust
impl<Context, Builder> HasOptionalBuilder for Context
where
    Context: HasBuilder,
    Context::Builder: ToOptional<Output = Builder>,
{
    type Builder = Builder;

    fn optional_builder() -> Self::Builder {
        Self::builder().to_optional()
    }
}
```

So `optional_builder()` starts at the all-`IsNothing` configuration and re-marks every field to
`IsOptional` with a [`TransformMapFields`](../type-level/transform_map_fields.md) walk carrying the
[`TransformOptional`](./transform_optional.md) marker. For the example's `Context`, the result is
`__PartialContext<IsOptional, IsOptional>`, the same partial companion the core builder uses, at a
configuration the core builder never reaches on its own.

Nothing about the companion type is special to this layer. What changes is the marker, and with it which
operations apply: [`SetOptional`](./set_optional.md) resolves where
[`BuildField`](../builder/build_field.md) does not, and the strict
[`FinalizeBuild`](../builder/finalize_build.md) does not resolve at all until a finalize has re-marked the fields
back to `IsPresent`.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`, along with whatever else in the
layer you use.

**Moving here gives up the compile-time completeness check.** With the core builder a missing field is a
compile error; here it is an `Err` or a silently substituted default. That is the trade, and it is worth
making consciously.

**The builder's type does not record what you have set.** Every field stays `IsOptional` throughout,
which allows re-setting and is why finalizing has to check at run time. It also means
[`finalize_with_default`](./can_finalize_with_default.md#common-mistakes) needs `Default` on every
field's type, set or not.

**`optional_builder()` is an associated function**, so it is `Context::optional_builder()` with no
receiver.

**A field whose type is already `Option<T>` is not the same as an `IsOptional` field.** The former is a
value that happens to be optional and must still be set; the latter is a builder slot that may be empty.
Conflating them produces an `Option<Option<T>>`.

## Related constructs

- [`SetOptional`](./set_optional.md): setting a field on the builder this produces.
- [`FinalizeOptional`](./finalize_optional.md): the strict ending, reporting the first missing field.
- [`CanFinalizeWithDefault`](./can_finalize_with_default.md): the defaulting ending.
- [`ToOptional`](./to_optional.md): converting an existing core builder instead of starting fresh.
- [`HasBuilder`](../builder/has_builder.md): the core entry point this relaxes.
- [`MapType`](../type-level/map_type.md): the `IsOptional` marker the layer runs on.
- [`TransformMapFields`](../type-level/transform_map_fields.md): the walk that re-marks every field.
- [`#[derive(BuildField)]`](../../derives/derive_build_field.md): generates the machinery underneath.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records, and where relaxing presence
  fits.

## Source

- [`to_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/to_optional.rs):
  `HasOptionalBuilder`, `ToOptional`, and `TransformOptional`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
