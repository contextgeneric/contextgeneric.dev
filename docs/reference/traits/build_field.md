---
sidebar_label: 'BuildField'
---

# `BuildField`

Setting one currently-absent field of a builder.

## Overview

`BuildField` is the direction of the [builder family](./has_builder.md) you write most: take a partial
record with a field absent, supply the value, and get back a partial record with that field present.

```rust
pub trait BuildField<Tag> {
    type Value;
    type Output;

    fn build_field(self, _tag: PhantomData<Tag>, value: Self::Value) -> Self::Output;
}
```

`Output` is a **different type** from `Self` — the same partial record with one marker flipped — which is
what makes the compiler track completeness. A chain of `build_field` calls walks through as many distinct
types as there are fields, and only the last one satisfies
[`FinalizeBuild`](./finalize_build.md).

It pins one transition of the [`UpdateField`](./update_field.md) primitive: `IsNothing → IsPresent`.
Because it *requires* the starting marker to be `IsNothing`, **building a field that is already set is a
compile error**, not a silent overwrite.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

The `PhantomData<Tag>` argument names the field, the same way a [`HasField`](./has_field.md) read does:
`PhantomData::<Symbol!("first_name")>` for a named field, `PhantomData::<Index<0>>` for a tuple-struct
position. `Value` is the field's declared type, so what you pass is an ordinary value with no wrapper.

**Nothing implements it directly.** It is a library blanket impl over
[`UpdateField`](./update_field.md), so every field the derive generates an `UpdateField` impl for gains
`build_field` for free.

## Examples

The everyday chain, one call per field:

```rust
use cgp::prelude::*;

#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

let person = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
    .build_field(PhantomData::<Symbol!("last_name")>, "Chen".to_owned())
    .finalize_build();
```

**Order does not matter**, because each impl constrains only its own field's marker and leaves the rest
generic. Setting `last_name` first compiles identically.

Mixing it with a merge is the usual shape once more than one source contributes:

```rust
use cgp::core::field::impls::CanBuildFrom;

let employee = Employee::builder()
    .build_from(person)                                       // the shared fields
    .build_field(PhantomData::<Symbol!("employee_id")>, 7)    // and the one it did not carry
    .finalize_build();
```

## When to reach for it, and when not

**Call `build_field` in any code that fills a builder; bound on the trait when the code is generic over
the record.**

- **Bound on [`HasBuilder`](./has_builder.md) + `BuildField` + [`FinalizeBuild`](./finalize_build.md)**
  to write a routine that assembles a record it does not name.
- **Use [`CanBuildFrom`](./can_build_from.md)** to copy every shared field from another record at once,
  rather than one `build_field` per field.
- **Use [`SetOptional`](./set_optional.md)** when a field must be settable more than once. `build_field`
  consumes an absent slot exactly once by design; the optional layer is what relaxes that.
- **Use [`UpdateField`](./update_field.md)** for a transition this does not cover.
- **Prefer a struct literal** in concrete code that knows every field.

## Under the hood

:::note

### Advanced

This section shows why the trait is not generated.

:::

`BuildField` is a **library blanket impl** over [`UpdateField`](./update_field.md), pinning the target
marker to `IsPresent` and constraining the reported source marker to `IsNothing`:

```rust
// conceptually:
//   impl<Tag, Partial> BuildField<Tag> for Partial
//   where Partial: UpdateField<Tag, IsPresent, Mapper = IsNothing>
```

The `Mapper = IsNothing` constraint is what does the checking. `Mapper` is an *output* of the primitive —
the marker the field was in — so constraining it selects only those partial types whose field is
currently absent. A field already set has `Mapper = IsPresent`, no impl matches, and the call fails to
resolve.

`update_field` returns the old value alongside the new partial; `build_field` discards it, which is sound
because the old value under `IsNothing` is `()`.

[`TakeField`](./take_field.md) is the mirror image — the same primitive with `IsNothing` as the target
and `Mapper = IsPresent` — which is why the two read as opposites and share every mechanism.

## Gotchas

**Building a field twice does not compile.** The second call finds `Mapper = IsPresent` and no impl. That
is the guarantee, and the error is a missing-method one rather than anything mentioning "already set".

**`Output` is a different type from `Self`.** A builder cannot be stored in a variable of fixed type
across a chain, and it cannot be filled in a loop — the chain is unrolled by construction.

**The error for an incomplete build arrives at `finalize_build`**, not here. Forgetting a field is only
detectable at the end, since any prefix of a chain is a legal partial value.

**It sets, it does not overwrite.** For a settable-and-resettable field, reach for
[`SetOptional`](./set_optional.md).

**The `PhantomData` argument carries the tag.** At a site where inference cannot determine it, write
`PhantomData::<Symbol!("name")>` in full.

**Nothing implements it directly**, so an unsatisfied `BuildField` bound is really an unsatisfied
[`UpdateField`](./update_field.md) one — which is what the error names.

## Related constructs

- [`UpdateField`](./update_field.md) — the primitive this pins one direction of.
- [`TakeField`](./take_field.md) — the opposite direction.
- [`HasBuilder`](./has_builder.md) and [`IntoBuilder`](./into_builder.md) — where a partial value comes
  from.
- [`FinalizeBuild`](./finalize_build.md) — where a completed one goes.
- [`CanBuildFrom`](./can_build_from.md) — filling many fields from another record in one call.
- [`SetOptional`](./set_optional.md) — the settable-repeatedly counterpart.
- [`MapType`](./map_type.md) — the `IsNothing`/`IsPresent` markers this moves between.
- [`#[derive(BuildField)]`](../derives/derive_build_field.md) — generates the `UpdateField` impls behind
  it.
- [`Symbol!`](../macros/symbol.md) and [`Index`](../types/index.md) — the tags that name a field.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records and the extensible builder
  pattern.

## Source

- [`build_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/build_field.rs)
  — `BuildField` and `FinalizeBuild`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
