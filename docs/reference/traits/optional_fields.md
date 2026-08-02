---
sidebar_label: 'Optional-field traits'
---

# Optional-field traits

The builder and extractor traits for optional and defaulted fields.

## What it's for

The core [builder family](./has_builder.md) is deliberately strict: a partial record becomes its concrete struct
only once **every** field is set, because `FinalizeBuild` is implemented solely on the all-present
configuration. That strictness is what catches a missing field at compile time — and it is too rigid for a record
where some fields have sensible defaults, or are genuinely allowed to be absent.

These traits relax it in two controlled ways, without giving up the machinery underneath.

**Defaulted finalization** fills any unset field from `Default` on the way out, so a record can be completed
without setting everything. **Optional fields** re-mark every field as an `Option`, which makes absence a
*runtime* condition — fields can then be set and re-set freely, and finalizing either requires presence with an
error or falls back to defaults.

The whole layer is composition rather than new mechanism. It reuses
[`UpdateField`](./has_builder.md) to move individual fields, [`TransformMapFields`](./map_type.md) to re-wrap a
whole record at once, and the [`MapType`](./map_type.md) markers `IsPresent`, `IsNothing`, and `IsOptional` to
name each state. Read it as a small set of transforms plus the entry points that drive them.

## Using it

**Nothing here is in the prelude.** Everything comes from `cgp::extra::field::impls`:

```rust
use cgp::extra::field::impls::{
    CanBuildWithDefault, CanFinalizeWithDefault, FinalizeOptional, HasOptionalBuilder, SetOptional,
};
```

The traits divide into the two capabilities, plus the two transform markers that power them.

### Defaulted finalization

`CanFinalizeWithDefault` finalizes a partial record, defaulting whatever is unset:

```rust
pub trait CanFinalizeWithDefault {
    type Output;
    fn finalize_with_default(self) -> Self::Output;
}
```

It is implemented for any partial value whose fields can be transformed to all-present, and its body is the
layer's core motion in one line: re-wrap every field to `IsPresent`, then call the ordinary
[`finalize_build`](./has_builder.md). The strict presence check still runs — it just always passes, because the
transform guaranteed presence first.

`CanBuildWithDefault` is the one-call version that also copies from a source:

```rust
pub trait CanBuildWithDefault<Source> {
    fn build_with_default(source: Source) -> Self;
}
```

It chains `builder()`, [`build_from`](./cast.md), and `finalize_with_default`, which makes it the field-level
"widening cast" — turning a `Point2d` into a `Point3d` whose extra `z` is `0`, naming no field explicitly.

### Optional fields

`HasOptionalBuilder` starts a builder in which every field is already optional:

```rust
pub trait HasOptionalBuilder {
    type Builder;
    fn optional_builder() -> Self::Builder;
}
```

`ToOptional` is the conversion it composes with, re-marking an existing partial value's every field to
`IsOptional`. `SetOptional<Tag>` then sets one:

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

**The crucial detail is that both methods return `Self`.** The field's marker is `IsOptional` before *and* after,
so the builder's type does not change — which is what makes an optional field settable repeatedly, unlike the
core `build_field` that consumes an absent slot exactly once. `set` discards whatever it replaced;
`set_optional` hands it back.

`FinalizeOptional` is the strict way out:

```rust
pub trait FinalizeOptional: PartialData {
    fn finalize_optional(self) -> Result<Self::Target, &'static str>;
}
```

It succeeds only if every field holds a value, and the error is the **first missing field's own name** as a
`&'static str` — so a caller learns which field was left unset.

### The two transforms

`TransformMapDefault` and `TransformOptional` are the [`TransformMap`](./map_type.md) markers that do the
per-field work. `TransformMapDefault` targets `IsPresent` — a present field passes through, an absent one becomes
its default, an optional one becomes its contents or the default. `TransformOptional` targets `IsOptional` —
present becomes `Some`, absent becomes `None`. You name these only when extending the layer.

## Examples

The optional workflow starts an all-optional builder, sets fields freely, and picks a finalize strategy at the
end:

```rust
use cgp::extra::field::impls::{
    CanFinalizeWithDefault, FinalizeOptional, HasOptionalBuilder, SetOptional,
};
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

// Every field set: finalize_optional succeeds.
let builder = Context::optional_builder()
    .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
    .set(PhantomData::<Symbol!("bar")>, 42);

// Setting an already-set field hands back what it replaced.
let (replaced, builder) = builder.set_optional(PhantomData::<Symbol!("foo")>, "bar".to_owned());
assert_eq!(replaced, Some("foo".to_owned()));

let context = builder.finalize_optional().unwrap();
assert_eq!(context.foo, "bar");
```

Leave a field unset and the two strategies diverge — which is the whole point of deferring the choice to the
finalize call:

```rust
// finalize_with_default fills `bar` from Default
let context = Context::optional_builder()
    .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
    .finalize_with_default();

assert_eq!(context.bar, 0);
```

Had that used `finalize_optional`, it would have returned `Err("bar")` instead of defaulting.

The defaulted-build path widens one record into another in a single call:

```rust
use cgp::extra::field::impls::CanBuildWithDefault;

#[derive(Debug, Clone, Eq, PartialEq, CgpData)]
struct Point2d { x: u64, y: u64 }

#[derive(Debug, Clone, Eq, PartialEq, CgpData)]
struct Point3d { x: u64, y: u64, z: u64 }

let point_3d = Point3d::build_with_default(Point2d { x: 1, y: 2 });

assert_eq!(point_3d, Point3d { x: 1, y: 2, z: 0 });   // z defaulted
```

## When to reach for it, and when not

**Reach for this layer when the core builder's strictness is genuinely wrong for the record**, and stay on the
core builder otherwise. The strictness is a feature — it is what turns a missing field into a compile error — so
relaxing it should be a decision rather than a convenience.

- **`finalize_with_default`** when unset fields have meaningful defaults and silence is the right outcome.
- **`finalize_optional`** when absence is an error you want to *report* rather than paper over. The two are the
  same builder finalized differently, and the choice is made at the call site — so the same value can go either
  way.
- **`optional_builder` and `set`** when fields must be settable in any order and more than once, which the core
  `build_field` cannot do.
- **`build_with_default`** for a one-call widening from a narrower record.
- **Stay on the [core builder](./has_builder.md)** when every field is genuinely required. You lose the
  compile-time completeness check by moving here, and that check is the reason the builder family exists.
- **Reach for a hand-written builder** when the *logic* is the point — validation at finalize, interdependent
  fields, computed defaults that are not `Default::default()`. This layer models presence and defaulting, and
  nothing else.

One thing to notice about the trade: `finalize_optional` moves a missing field from a compile error to a
`Result`, and its error type is a `&'static str` field name rather than a structured value. That is enough to
report which field is missing and not enough to match on programmatically.

## Under the hood

:::note

### Advanced

This section shows how both paths reuse one recursion. It is the shortest explanation of why the defaulted and
optional workflows behave so symmetrically.

:::

Both capabilities are the **same** [`transform_map_fields`](./map_type.md) recursion with a different marker, which
is why they mirror each other so exactly:

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

`CanFinalizeWithDefault` drives `TransformMapDefault` toward `IsPresent`; `ToOptional` drives
`TransformOptional` toward `IsOptional`. Both walk the same field spine, both rebuild the partial type one field
at a time through [`UpdateField`](./has_builder.md), and neither changes a value's runtime layout beyond wrapping
or unwrapping an `Option` or substituting a default.

`SetOptional` is the one that does *not* transform the whole record — it is a single `UpdateField` call writing
`Some(value)` into an `IsOptional` slot, with `Mapper = IsOptional, Output = Context` pinning the type unchanged
on both sides. That pinning is exactly what allows repeated sets.

`FinalizeOptional` walks the target's [`HasFields`](./has_fields.md) spine itself rather than using a transform:
for each field it pulls the `Option` out with `UpdateField`, writes it back as `IsPresent` with
[`BuildField`](./has_builder.md) if it is `Some`, and returns `Err(Tag::VALUE)` — the field's name recovered as a
static string through [`StaticString`](./static_format.md) — the moment it finds a `None`. Only if every field
yields a value does it reach `finalize_build`.

The strict, all-present `FinalizeBuild` from the core remains the **only** way a partial value becomes a concrete
struct. Everything here simply guarantees the all-present configuration is reached before it is invoked.

## Gotchas

**Nothing here is in the prelude.** Import from `cgp::extra::field::impls`.

**`set` returns `Self`, so the builder's type never changes.** That is deliberate — it is what allows re-setting —
but it also means the type carries no record of *which* fields you have set, which is why finalizing has to check
at run time.

**Moving to an optional builder gives up the compile-time completeness check.** With the core builder a missing
field is a compile error; here it is an `Err` or a silently substituted default. That is the trade, and it is
worth making consciously.

**`finalize_optional`'s error is a `&'static str`.** It names the first missing field and is not a structured
error. It also reports only the first.

**`finalize_with_default` needs `Default` on every field's type.** A field whose type has no `Default` makes the
transform unresolvable, and the error names the missing `TransformMap` impl rather than the field.

**`build_with_default` requires the source to have a shape.** It chains [`build_from`](./cast.md), so the source
needs [`HasFields`](./has_fields.md) — the same requirement as any merge.

**A field whose type is already `Option<T>` is not the same as an `IsOptional` field.** The former is a value that
happens to be optional and must still be *set*; the latter is a builder slot that may be empty. Conflating them
produces an `Option<Option<T>>`.

## Related constructs

- [`HasBuilder`](./has_builder.md) — the core family this layer relaxes, and where `FinalizeBuild` and
  `UpdateField` are documented.
- [`MapType`](./map_type.md) — `TransformMap`, `TransformMapFields`, and the `IsOptional` marker this layer runs
  on.
- [`CanUpcast`](./cast.md) — where `CanBuildFrom` and `build_from` are documented.
- [`HasFields`](./has_fields.md) — the shape both the transform and `FinalizeOptional` walk.
- [`#[derive(BuildField)]`](../derives/derive_build_field.md) — generates the partial types these operate on,
  also through [`#[derive(CgpData)]`](../derives/derive_cgp_data.md).
- [`StaticString`](./static_format.md) — how `FinalizeOptional` recovers a field's name as a string.
- [`ExtractField`](./extract_field.md) — the enum-side counterpart of the record machinery.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records, and where relaxing presence fits.

## Source

- [`build_default.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/build_default.rs)
  — `CanBuildWithDefault`, `CanFinalizeWithDefault`, `TransformMapDefault`
- [`to_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/to_optional.rs)
  — `HasOptionalBuilder`, `ToOptional`, `TransformOptional`
- [`set_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/set_optional.rs)
  — `SetOptional`
- [`finalize_optional.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-field-extra/src/impls/finalize_optional.rs)
  — `FinalizeOptional`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
