---
sidebar_label: 'FinalizeBuild'
sidebar_position: 6
---

# `FinalizeBuild`

Turning a fully-built partial record back into the concrete struct.

## Overview

`FinalizeBuild` is where a build ends. What makes it the family's safety argument is not the method but
**where the impl exists**. It is
implemented for exactly one configuration of a partial record — the one in which every field is
`IsPresent` — so calling it on an incomplete value is a *missing impl*, not a runtime check that fails:

```rust
let person = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, first)
    .build_field(PhantomData::<Symbol!("last_name")>, last)
    .finalize_build();                                       // resolves only here
```

Delete a middle line and this does not compile. There is nothing to run and nothing to panic — the method
is simply not in scope for a value with a field still absent.

The destination type comes from its supertrait [`PartialData`](./partial_data.md), which every
configuration implements. That division is deliberate: **one names where you are going, the other says
you have arrived.**

## Definition

`FinalizeBuild` has one method and a supertrait:

```rust
pub trait FinalizeBuild: PartialData {
    fn finalize_build(self) -> Self::Target;
}
```

`finalize_build` takes `self` by value, so it consumes the partial record, and returns `Self::Target`, the
concrete struct being built. `Target` is not declared here: it comes from the supertrait
[`PartialData`](./partial_data.md), which every configuration of a partial record implements, so the
destination is nameable at any point in a build. What `FinalizeBuild` adds is the method, and its impl
exists for only one configuration — every field `IsPresent` — which is the safety argument the rest of
this page works out.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

`finalize_build` consumes the partial value and returns `Self::Target`. It takes no arguments — there is
nothing left to decide by the time it applies.

Bounding on it is how generic builder code says it will produce a finished value:

```rust
fn assemble<Partial, Target>(partial: Partial) -> Target
where
    Partial: FinalizeBuild<Target = Target>,
{
    partial.finalize_build()
}
```

The impls come from [`#[derive(BuildField)]`](../../derives/derive_build_field.md), which emits exactly one
per record.

## Examples

The ordinary ending of a build chain:

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

And the failure it exists to produce — omitting a field:

```rust
// error: no method named `finalize_build` found for struct
//        `__PartialPerson<IsPresent, IsNothing>`
let person = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
    .finalize_build();
```

**Read the partial type in the message to see which marker is still `IsNothing`.** That is the field you
forgot, and it is the only place the compiler names it.

## When to use it

**Call it at the end of every build; bound on it when the code is generic over the record.**

- **Bound on [`HasBuilder`](./has_builder.md) + [`BuildField`](./build_field.md) + `FinalizeBuild`** to
  write a routine that assembles a record it does not name.
- **Use [`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md)** when unset fields should be filled
  from `Default` rather than rejected. It re-marks every field to `IsPresent` and then calls this, so the
  strict check still runs and always passes.
- **Use [`FinalizeOptional`](../optional/finalize_optional.md)** when absence should be *reported* at run time
  rather than caught at compile time. It returns a `Result` naming the first missing field.
- **Stay with `FinalizeBuild`** when every field is genuinely required. Moving to either of the other two
  gives up the compile-time completeness check, which is the reason the family exists.

## Under the hood

The derive emits one impl, with every marker fixed:

```rust
impl FinalizeBuild for __PartialPerson<IsPresent, IsPresent> {
    // ...
}
```

Compare its supertrait, which leaves every marker generic:

```rust
// impl<F0: MapType, F1: MapType> PartialData for __PartialPerson<F0, F1> {
//     type Target = Person;
// }
```

So a partial value always knows its destination and only sometimes has a way to reach it. Because
`IsPresent::Map<T>` is `T`, the all-present companion holds exactly the concrete struct's fields in the
same order, and the body is a field-by-field move with nothing to unwrap.

**This is why the error is a missing method rather than a missing field.** Method resolution looks for
`finalize_build` on `__PartialPerson<IsPresent, IsNothing>`, finds no impl, and reports that — the
compiler has no way to say "you forgot `last_name`", because nothing in the failed lookup mentions field
names. The marker list in the type is the diagnostic.

The optional layer reaches this same impl rather than replacing it:
[`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) runs a
[`TransformMapFields`](../type-level/transform_map_fields.md) to `IsPresent` first, so **the strict, all-present impl
remains the only way a partial value becomes a concrete struct.**

## Common Mistakes

**"No method named `finalize_build`" is the expected error for an incomplete build.** Read the partial
type in the message: the field whose marker is still `IsNothing` is the one missing.

**It consumes the partial value**, so the builder cannot be reused afterwards.

**It supertraits [`PartialData`](./partial_data.md)**, so naming both in a bound is redundant, and
`Target` is projected from the supertrait.

**There is no fallible variant here.** Reporting absence at run time is
[`FinalizeOptional`](../optional/finalize_optional.md), a different trait in a different crate.

**A fieldless struct finalizes immediately.** Its companion has no markers, so `builder().finalize_build()`
compiles — legal and useless.

**The enum side has its own ending.** [`FinalizeExtract`](../variant/finalize_extract.md) discharges an exhausted
extractor, and it is sound for the opposite reason: the value cannot exist, rather than being complete.

## Related constructs

- [`PartialData`](./partial_data.md) — the supertrait naming the destination.
- [`HasBuilder`](./has_builder.md) and [`IntoBuilder`](./into_builder.md) — where a partial value comes
  from.
- [`BuildField`](./build_field.md) and [`TakeField`](./take_field.md) — what moves it between
  configurations.
- [`UpdateField`](./update_field.md) — the primitive underneath both.
- [`CanFinalizeWithDefault`](../optional/can_finalize_with_default.md) and
  [`FinalizeOptional`](../optional/finalize_optional.md) — the two relaxed endings.
- [`MapType`](../type-level/map_type.md) — the `IsPresent` marker every field must reach.
- [`FinalizeExtract`](../variant/finalize_extract.md) — the enum family's ending, sound for the opposite reason.
- [`#[derive(BuildField)]`](../../derives/derive_build_field.md) — generates this impl.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records and the extensible builder
  pattern.

## Source

- [`build_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/build_field.rs)
  — `FinalizeBuild` and `BuildField`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
