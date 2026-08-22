---
sidebar_label: 'SetOptional'
---

# `SetOptional`

Setting one field of an optional builder, as many times as you like.

## Overview

[`BuildField`](./build_field.md) sets an absent field and consumes the slot: the builder's type changes,
and setting the same field twice is a compile error. `SetOptional` is the optional layer's answer, and its
defining property is the opposite:

```rust
pub trait SetOptional<Tag> {
    type Value;

    fn set(self, _tag: PhantomData<Tag>, value: Self::Value) -> Self;

    fn set_optional(
        self,
        _tag: PhantomData<Tag>,
        value: Self::Value,
    ) -> (Option<Self::Value>, Self);
}
```

**Both methods return `Self`.** The field's marker is `IsOptional` before *and* after, so the builder's
type does not change — which is exactly what makes an optional field settable repeatedly.

The two methods differ only in what they do with whatever was there: `set` discards it, and
`set_optional` hands it back.

## Usage

**It is not in the prelude.** Import it from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::SetOptional;
```

`Self` must be an optional builder — one from
[`optional_builder()`](./has_optional_builder.md) or [`to_optional()`](./to_optional.md). Calling it on a
core builder does not resolve, because the field's marker is `IsNothing` rather than `IsOptional`.

`Value` is the field's declared type, so what you pass is an ordinary value rather than an `Option`.
Setting a field to "absent" is not something this trait does; a field starts absent and stays so unless
set.

## Examples

Setting fields in any order, and re-setting one:

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

// Setting an already-set field hands back what it replaced.
let (replaced, builder) = builder.set_optional(PhantomData::<Symbol!("foo")>, "bar".to_owned());

assert_eq!(replaced, Some("foo".to_owned()));

let context = builder.finalize_optional().unwrap();

assert_eq!(context.foo, "bar");
```

Because `set` returns `Self`, the calls chain exactly as core `build_field` calls do — the difference is
invisible at the call site and lives entirely in the type.

## When to reach for it, and when not

**Reach for it when a field may be set more than once, or when the order is not known.** That is the one
thing the core builder cannot express.

- **[`BuildField`](./build_field.md)** when each field is set exactly once. The compile-time
  completeness check it buys is the reason the core family exists.
- **`set`** when whatever was there should simply be replaced.
- **`set_optional`** when the previous value matters — merging, accumulating, warning on a duplicate.
- **[`CanBuildFrom`](./can_build_from.md)** when the values come from another record wholesale rather
  than one at a time.

## Under the hood

:::note

### Advanced

This section shows why the type does not change, which is the trait's whole trick.

:::

`SetOptional` is a single [`UpdateField`](./update_field.md) call writing `Some(value)` into an
`IsOptional` slot, with both the source and target markers pinned to `IsOptional`:

```rust
// conceptually:
//   Partial: UpdateField<Tag, IsOptional, Mapper = IsOptional, Output = Partial>
```

`Mapper = IsOptional` requires the field to already be in the optional state, and `Output = Partial`
pins the result to the same type on both sides. **That pinning is exactly what allows repeated sets** —
there is no marker to consume, so no impl becomes unavailable after the first call.

Contrast [`BuildField`](./build_field.md#under-the-hood), which pins `Mapper = IsNothing` and a target of
`IsPresent`: its `Output` is a *different* type, which is what makes a second call fail to resolve.

Since the primitive returns the old storage alongside the new value, `set_optional` simply exposes it and
`set` discards it. Under `IsOptional` that old storage is an `Option<Value>`, which is why the returned
value is optional even though the argument is not.

## Gotchas

**It is not in the prelude.** Import from `cgp::extra::field::impls`.

**It does not apply to a core builder.** The field must already be `IsOptional`, so start with
[`optional_builder()`](./has_optional_builder.md) or convert with [`to_optional()`](./to_optional.md).

**The builder's type never changes, so it carries no record of what you have set.** That is deliberate —
it is what allows re-setting — and it is why finalizing has to check at run time.

**`set_optional` returns `(previous, builder)`, in that order.** Reversing the binding is a type error,
and a confusing one in generic code.

**The argument is a `Value`, not an `Option<Value>`.** There is no "unset" operation; a field left alone
stays absent.

**A field whose type is already `Option<T>` takes an `Option<T>` here and is stored as
`Option<Option<T>>`.** The outer layer is the builder's presence tracking, and the two are easy to
conflate.

## Related constructs

- [`HasOptionalBuilder`](./has_optional_builder.md) — where an optional builder comes from.
- [`ToOptional`](./to_optional.md) — converting an existing builder into one.
- [`FinalizeOptional`](./finalize_optional.md) and
  [`CanFinalizeWithDefault`](./can_finalize_with_default.md) — the two endings.
- [`BuildField`](./build_field.md) — the strict, set-once counterpart.
- [`UpdateField`](./update_field.md) — the primitive this is one call of.
- [`MapType`](./map_type.md) — the `IsOptional` marker it pins on both sides.
- [`CanBuildFrom`](./can_build_from.md) — filling many fields from another record.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records, and where relaxing presence
  fits.

## Source

- [`set_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/set_optional.rs)
  — `SetOptional`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
