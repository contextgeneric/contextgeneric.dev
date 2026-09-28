---
title: 'FinalizeOptional — finalize or name the gap'
sidebar_label: 'FinalizeOptional'
sidebar_position: 4
description: 'End an optional build at run time: the concrete struct if every field holds a value, or the name of a missing field as an Err.'
---

# `FinalizeOptional`

Finalizing an optional builder, reporting a missing field.

## Overview

An [optional builder](./has_optional_builder.md) has given up the compile-time completeness check: every
field is `IsOptional`, so the type no longer records what has been set. Something has to check at run
time instead, and `FinalizeOptional` is the strict way to do it: it succeeds only if every field holds a
value, and otherwise reports a missing field by name.

It is one of two endings for an optional builder, and the choice between them is made at the call site
rather than when the builder is created:

| | a field left unset |
|---|---|
| `FinalizeOptional` | `Err("field_name")` |
| [`CanFinalizeWithDefault`](./can_finalize_with_default.md) | filled from `Default` |

## Definition

`FinalizeOptional` [supertraits](/docs/reference/glossary#supertrait) [`PartialData`](../builder/partial_data.md) and finalizes an optional
builder into its concrete struct:

```rust
pub trait FinalizeOptional: PartialData {
    fn finalize_optional(self) -> Result<Self::Target, &'static str>;
}
```

The `PartialData` [supertrait](/docs/reference/glossary#supertrait) supplies `Target`, the concrete
struct being built, so the method projects its return type through it and does not declare an
associated type of its own. `finalize_optional` takes `self`, consuming the builder, and returns
`Result<Self::Target, &'static str>`: the built struct on success, or, on failure, a missing field's
own name, recovered from its type-level tag as a `&'static str` without allocating. The trait is not
generic. It is not in the prelude; import it from `cgp-field-extra`.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::FinalizeOptional;
```

`Self` must be an optional builder, from [`optional_builder()`](./has_optional_builder.md) or
[`to_optional()`](./to_optional.md). The record also needs [`HasFields`](../shape/has_fields.md),
since the walk runs over its field list;
[`#[derive(CgpData)]`](../../derives/derive_cgp_data.md) supplies it along with the builder.

## Examples

The strict ending, succeeding and failing:

```rust
use cgp::prelude::*;
use cgp::extra::field::impls::{FinalizeOptional, HasOptionalBuilder, SetOptional};

#[derive(Debug, PartialEq, CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

pub fn demo() {
    let complete = Context::optional_builder()
        .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
        .set(PhantomData::<Symbol!("bar")>, 42)
        .finalize_optional();
    assert_eq!(
        complete,
        Ok(Context {
            foo: "foo".to_owned(),
            bar: 42,
        })
    );

    let missing_bar = Context::optional_builder()
        .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
        .finalize_optional();
    assert_eq!(missing_bar, Err("bar"));

    // With both unset, the walk from the last field back stops at `bar`.
    assert_eq!(Context::optional_builder().finalize_optional(), Err("bar"));
}
```

With both fields unset the error is still `"bar"`: the walk checks the last declared field first and
stops at the first `None` it meets.

## When to use it

**Reach for it when absence is an error you want to *report* rather than hide.**

- **[`CanFinalizeWithDefault`](./can_finalize_with_default.md)** when unset fields have meaningful
  defaults and silence is the right outcome. The two are the same builder finalized differently, so the
  same value can go either way.
- **[`FinalizeBuild`](../builder/finalize_build.md)** (that is, the core builder) when every field is genuinely
  required and set once. A missing field is then a *compile* error, which is strictly better than a
  `Result`, and moving to this layer gives that check up.
- **A hand-written check** when the validation is more than presence: interdependent fields, ranges,
  anything needing a structured error.

One thing to notice about the trade: this moves a missing field from a compile error to a `Result`, and
the error type is a `&'static str` field name rather than a structured value. That is enough to report
which field is missing and not enough to match on programmatically.

## Under the hood

Unlike its defaulting sibling, `FinalizeOptional` does **not** go through
[`TransformMapFields`](../type-level/transform_map_fields.md). It walks the target's
[`HasFields`](../shape/has_fields.md) list directly, and the reason is that it has to be able to
*stop*. The trait's one impl hands the walk the builder and finalizes whatever comes back:

```rust
impl<ContextA, ContextB, Target> FinalizeOptional for ContextA
where
    ContextA: PartialData<Target = Target>,
    Target: HasFields,
    Target::Fields: FinalizeOptionalImpl<ContextA, Output = ContextB>,
    ContextB: FinalizeBuild<Target = Target>,
{
    fn finalize_optional(self) -> Result<Self::Target, &'static str> {
        let context = Target::Fields::finalize_optional(self)?;
        Ok(context.finalize_build())
    }
}
```

The walk is a private helper trait, implemented for `Nil` as the identity and for each `Cons` cell
as:

```rust
fn finalize_optional(context: ContextA) -> Result<Self::Output, &'static str> {
    let context = Rest::finalize_optional(context)?;
    let (m_value, context) = context.update_field(PhantomData, ());

    let value = m_value.ok_or(Tag::VALUE)?;
    let context = context.build_field(PhantomData, value);

    Ok(context)
}
```

It checks the rest of the list before the current field, so the fields are checked from last to
first. For each field, [`UpdateField`](../builder/update_field.md) moves it from `IsOptional` to
`IsNothing` and hands back the `Option`. A `Some` is written back as `IsPresent` with
[`BuildField`](../builder/build_field.md); a `None` returns `Err(Tag::VALUE)`, the field's name
recovered as a static string through [`StaticString`](../formatting/static_string.md), and the `?`
stops the walk.

Only if every field yields a value does the walk reach the all-present configuration and call
[`finalize_build`](../builder/finalize_build.md).

**The strict, all-present [`FinalizeBuild`](../builder/finalize_build.md) remains the only way a partial value
becomes a concrete struct.** Everything in this layer guarantees that configuration is reached
before it is invoked, or reports why it could not be.

That short-circuiting is also why only one missing field is named: the walk stops at the first
`None` it meets rather than collecting. Because the walk runs from the last field back, that is the
last unset field in declaration order: with nothing set on `struct Context { foo: String, bar: u64 }`,
the error is `"bar"`.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**The error is a `&'static str`.** It names one missing field and is not a structured error, so it
cannot be matched on beyond string comparison.

**It reports only one.** A builder missing three fields yields one name, the last of them in
declaration order.

**It does not apply to a core builder.** Calling it on a builder from `builder()`:

```rust
let _ = Context::builder().finalize_optional();
```

fails method resolution, and the notes list only the unmet bounds for the autoref'd receivers, with
nothing about `IsOptional`:

```text
error[E0599]: the method `finalize_optional` exists for struct `__PartialContext<IsNothing, IsNothing>`, but its trait bounds were not satisfied
```

The fields must be `IsOptional`; a core builder either finalizes at compile time or does not.

**It supertraits [`PartialData`](../builder/partial_data.md)**, so `Target` is projected from there and naming
both in a bound is redundant.

**Field order decides which name you get.** Reordering a struct's fields changes which missing
field is reported, so a test asserting on the name depends on the declaration order.

## Related constructs

- [`CanFinalizeWithDefault`](./can_finalize_with_default.md): the other ending, filling gaps from
  `Default`.
- [`HasOptionalBuilder`](./has_optional_builder.md) and [`ToOptional`](./to_optional.md): where an
  optional builder comes from.
- [`SetOptional`](./set_optional.md): filling one.
- [`FinalizeBuild`](../builder/finalize_build.md): the strict impl this ultimately calls.
- [`PartialData`](../builder/partial_data.md): the supertrait naming the destination.
- [`StaticString`](../formatting/static_string.md): how the missing field's name is recovered.
- [`UpdateField`](../builder/update_field.md) and [`BuildField`](../builder/build_field.md): the primitives the walk uses.
- [`HasFields`](../shape/has_fields.md): the shape it walks.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records, and where relaxing presence
  fits.

## Source

- [`finalize_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/finalize_optional.rs):
  `FinalizeOptional`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
