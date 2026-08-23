---
sidebar_label: 'HasOptionalBuilder'
---

# `HasOptionalBuilder`

Starting a builder in which every field is already optional.

## Overview

The core [builder family](./has_builder.md) is deliberately strict: a field is set exactly once, and a
partial record becomes its concrete struct only when **every** field is present. That strictness is what
catches a missing field at compile time, and it is too rigid for a record whose fields arrive in an
unpredictable order, or more than once.

`HasOptionalBuilder` is the entry point that relaxes it:

```rust
pub trait HasOptionalBuilder {
    type Builder;

    fn optional_builder() -> Self::Builder;
}
```

It hands back a partial value with every field marked `IsOptional` rather than `IsNothing` — so each
slot holds an `Option` and **absence becomes a runtime condition instead of a type-level one.** Fields
can then be set freely with [`SetOptional`](./set_optional.md), in any order and as many times as you
like, and the choice of how to end is deferred to the finalize call.

It stands to [`HasBuilder`](./has_builder.md) as the optional layer stands to the core one: same
machinery, different starting marker.

## Usage

**It is not in the prelude.** Nothing in the optional-field layer is — import it from
`cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::HasOptionalBuilder;
```

`optional_builder()` is an associated function with no receiver: `Context::optional_builder()`.

The impls are blanket ones over the core builder machinery, so any record deriving
[`#[derive(BuildField)]`](../derives/derive_build_field.md) — or
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md) — gets this for free. There is no separate derive.

**Two endings are available**, and picking between them at the call site is the layer's real payoff:
[`FinalizeOptional`](./finalize_optional.md) requires every field and reports the first missing one, while
[`CanFinalizeWithDefault`](./can_finalize_with_default.md) fills unset fields from `Default`.

## Examples

An all-optional builder, set in any order and finalized strictly:

```rust
use cgp::extra::field::impls::{FinalizeOptional, HasOptionalBuilder, SetOptional};
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

let builder = Context::optional_builder()
    .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
    .set(PhantomData::<Symbol!("bar")>, 42);

let context = builder.finalize_optional().unwrap();

assert_eq!(context.foo, "foo");
```

Leave a field unset and the two endings diverge, which is the point of deferring the choice:

```rust
use cgp::extra::field::impls::CanFinalizeWithDefault;

let context = Context::optional_builder()
    .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
    .finalize_with_default();

assert_eq!(context.bar, 0);   // defaulted
```

Had that used `finalize_optional`, it would have returned `Err("bar")` instead.

## When to use it

**Reach for it when fields must be settable in any order and more than once**, which the core
`build_field` cannot do — and stay on the core builder otherwise.

- **[`HasBuilder`](./has_builder.md)** when every field is genuinely required and set once. You lose the
  compile-time completeness check by moving here, and that check is the reason the builder family
  exists.
- **`HasOptionalBuilder`** when the fields arrive unpredictably — parsed configuration, accumulated
  defaults, a value assembled across several passes.
- **[`ToOptional`](./to_optional.md)** when you already hold a core builder and want to convert it rather
  than start fresh.
- **A hand-written builder** when the *logic* is the point — validation at finalize, interdependent
  fields, computed defaults that are not `Default::default()`. This layer models presence and defaulting,
  and nothing else.

## Under the hood

`optional_builder()` is [`HasBuilder`](./has_builder.md)'s `builder()` followed by
[`ToOptional`](./to_optional.md) — start at all-`IsNothing`, then re-mark every field to `IsOptional`
with a [`TransformMapFields`](./transform_map_fields.md) walk carrying the
[`TransformOptional`](./transform_optional.md) marker.

So the resulting type is the same partial companion the core builder uses, at a configuration the core
builder never reaches on its own:

```rust
// __PartialContext<IsOptional, IsOptional>
```

Nothing about the companion type is special to this layer. What changes is the marker, and with it which
operations apply: [`SetOptional`](./set_optional.md) resolves where
[`BuildField`](./build_field.md) does not, and the strict
[`FinalizeBuild`](./finalize_build.md) does not resolve at all until a finalize has re-marked the fields
back to `IsPresent`.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`, along with whatever else in the
layer you use.

**Moving here gives up the compile-time completeness check.** With the core builder a missing field is a
compile error; here it is an `Err` or a silently substituted default. That is the trade, and it is worth
making consciously.

**The builder's type does not record what you have set.** Every field stays `IsOptional` throughout,
which is what allows re-setting and is why finalizing has to check at run time.

**`optional_builder()` is an associated function**, so it is `Context::optional_builder()` with no
receiver.

**A field whose type is already `Option<T>` is not the same as an `IsOptional` field.** The former is a
value that happens to be optional and must still be set; the latter is a builder slot that may be empty.
Conflating them produces an `Option<Option<T>>`.

## Related constructs

- [`SetOptional`](./set_optional.md) — setting a field on the builder this produces.
- [`FinalizeOptional`](./finalize_optional.md) — the strict ending, reporting the first missing field.
- [`CanFinalizeWithDefault`](./can_finalize_with_default.md) — the defaulting ending.
- [`ToOptional`](./to_optional.md) — converting an existing core builder instead of starting fresh.
- [`HasBuilder`](./has_builder.md) — the core entry point this relaxes.
- [`MapType`](./map_type.md) — the `IsOptional` marker the layer runs on.
- [`TransformMapFields`](./transform_map_fields.md) — the walk that re-marks every field.
- [`#[derive(BuildField)]`](../derives/derive_build_field.md) — generates the machinery underneath.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records, and where relaxing presence
  fits.

## Source

- [`to_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/to_optional.rs)
  — `HasOptionalBuilder`, `ToOptional`, and `TransformOptional`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
