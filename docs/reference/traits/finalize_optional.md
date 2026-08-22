---
sidebar_label: 'FinalizeOptional'
---

# `FinalizeOptional`

Finalizing an optional builder, reporting the first missing field.

## Overview

An [optional builder](./has_optional_builder.md) has given up the compile-time completeness check: every
field is `IsOptional`, so the type no longer records what has been set. Something has to check at run
time instead, and `FinalizeOptional` is the strict way to do it:

```rust
pub trait FinalizeOptional: PartialData {
    fn finalize_optional(self) -> Result<Self::Target, &'static str>;
}
```

It succeeds only if every field holds a value. The error is **the first missing field's own name**, as a
`&'static str` recovered from the field's type-level tag — so a caller learns which field was left unset
without any allocation.

It is one of two endings for an optional builder, and the choice between them is made at the call site
rather than when the builder is created:

| | a field left unset |
|---|---|
| `FinalizeOptional` | `Err("field_name")` |
| [`CanFinalizeWithDefault`](./can_finalize_with_default.md) | filled from `Default` |

## Usage

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::FinalizeOptional;
```

It supertraits [`PartialData`](./partial_data.md), which is where `Target` — the concrete struct being
built — comes from.

`Self` must be an optional builder, from
[`optional_builder()`](./has_optional_builder.md) or [`to_optional()`](./to_optional.md).

## Examples

The strict ending, succeeding and failing:

```rust
use cgp::extra::field::impls::{FinalizeOptional, HasOptionalBuilder, SetOptional};
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

let context = Context::optional_builder()
    .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
    .set(PhantomData::<Symbol!("bar")>, 42)
    .finalize_optional()
    .unwrap();

assert_eq!(context.foo, "foo");
```

Leave `bar` unset and it reports the name:

```rust
let result = Context::optional_builder()
    .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
    .finalize_optional();

assert_eq!(result.err(), Some("bar"));
```

## When to reach for it, and when not

**Reach for it when absence is an error you want to *report* rather than paper over.**

- **[`CanFinalizeWithDefault`](./can_finalize_with_default.md)** when unset fields have meaningful
  defaults and silence is the right outcome. The two are the same builder finalized differently, so the
  same value can go either way.
- **[`FinalizeBuild`](./finalize_build.md)** — that is, the core builder — when every field is genuinely
  required and set once. A missing field is then a *compile* error, which is strictly better than a
  `Result` and is what you give up by moving to this layer.
- **A hand-written check** when the validation is more than presence: interdependent fields, ranges,
  anything needing a structured error.

One thing to notice about the trade: this moves a missing field from a compile error to a `Result`, and
the error type is a `&'static str` field name rather than a structured value. That is enough to report
which field is missing and not enough to match on programmatically.

## Under the hood

Unlike its defaulting sibling, `FinalizeOptional` does **not** go through
[`TransformMapFields`](./transform_map_fields.md). It walks the target's
[`HasFields`](./has_fields.md) spine directly, and the reason is that it has to be able to *stop*.

For each field it pulls the `Option` out with [`UpdateField`](./update_field.md), and:

- if it is `Some`, writes the value back as `IsPresent` with [`BuildField`](./build_field.md) and
  continues;
- if it is `None`, returns `Err(Tag::VALUE)` immediately — the field's name recovered as a static string
  through [`StaticString`](./static_string.md).

Only if every field yields a value does the walk reach the all-present configuration and call
[`finalize_build`](./finalize_build.md).

**The strict, all-present [`FinalizeBuild`](./finalize_build.md) remains the only way a partial value
becomes a concrete struct.** Everything in this layer simply guarantees that configuration is reached
before it is invoked — or reports why it could not be.

That short-circuiting is also why only the *first* missing field is named: the walk stops at it rather
than collecting.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**The error is a `&'static str`.** It names the first missing field and is not a structured error, so it
cannot be matched on beyond string comparison.

**It reports only the first.** A builder missing three fields yields one name.

**It does not apply to a core builder.** The fields must be `IsOptional`; a core builder either
finalizes at compile time or does not.

**It supertraits [`PartialData`](./partial_data.md)**, so `Target` is projected from there and naming
both in a bound is redundant.

**Field order decides which name you get**, since the walk follows declaration order and stops at the
first `None`.

## Related constructs

- [`CanFinalizeWithDefault`](./can_finalize_with_default.md) — the other ending, filling gaps from
  `Default`.
- [`HasOptionalBuilder`](./has_optional_builder.md) and [`ToOptional`](./to_optional.md) — where an
  optional builder comes from.
- [`SetOptional`](./set_optional.md) — filling one.
- [`FinalizeBuild`](./finalize_build.md) — the strict impl this ultimately calls.
- [`PartialData`](./partial_data.md) — the supertrait naming the destination.
- [`StaticString`](./static_string.md) — how the missing field's name is recovered.
- [`UpdateField`](./update_field.md) and [`BuildField`](./build_field.md) — the primitives the walk uses.
- [`HasFields`](./has_fields.md) — the shape it walks.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records, and where relaxing presence
  fits.

## Source

- [`finalize_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/finalize_optional.rs)
  — `FinalizeOptional`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
