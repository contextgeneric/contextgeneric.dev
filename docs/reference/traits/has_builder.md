---
sidebar_label: 'HasBuilder'
---

# `HasBuilder`

Starting an empty builder for a record.

## Overview

A struct literal supplies every field at once, in one place that names the type. `HasBuilder` is the
entry point for the case where that is impossible: the fields come from several independent places, none
of which should know the whole struct, and the result must still be checked at compile time.

```rust
pub trait HasBuilder {
    type Builder;

    fn builder() -> Self::Builder;
}
```

`builder()` hands back a partial value with **every field absent**. Filling it is
[`BuildField`](./build_field.md), and turning it back into the concrete struct is
[`FinalizeBuild`](./finalize_build.md) — which is implemented **only** for the fully-present
configuration, so finalizing early is a missing impl rather than a runtime panic:

```rust
let person = Person::builder()                                      // every field absent
    .build_field(PhantomData::<Symbol!("first_name")>, first)
    .build_field(PhantomData::<Symbol!("last_name")>, last)
    .finalize_build();                                              // all present: this resolves
```

Delete a middle line and this does not compile. **Field presence lives in the type**, which is the
defining idea of the whole family.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

`builder()` is an associated function with no receiver, called as `Person::builder()` or `T::builder()`
in generic code. `Builder` names the partial companion type the derive generated — you rarely write that
type, but it is what appears in an error when a build is incomplete.

The impls come from [`#[derive(BuildField)]`](../derives/derive_build_field.md), also available through
[`#[derive(CgpRecord)]`](../derives/derive_cgp_record.md) and
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md).

**Its counterpart is [`IntoBuilder`](./into_builder.md)**, which starts from an existing value with every
field *present* rather than from nothing. The two differ only in where they begin, and the choice is
about whether you are assembling a record or redistributing one.

## Examples

The everyday shape is `builder()`, some `build_field` calls, and `finalize_build`, with
[`build_from`](./can_build_from.md) copying every shared field from another record in one step:

```rust
use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

// `build_from` walks the *source's* field list, so the source needs `HasFields` too.
#[derive(HasFields, BuildField)]
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
}

#[derive(BuildField)]
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
}

fn extend(foo_bar: FooBar) -> FooBarBaz {
    FooBarBaz::builder()                                     // all absent
        .build_from(foo_bar)                                 // foo, bar now present
        .build_field(PhantomData::<Symbol!("baz")>, true)     // baz now present
        .finalize_build()                                    // only the all-present impl applies
}
```

Every line changes the partial *type*, and the last one type-checks only because every marker has reached
present. Reorder them so `finalize_build` runs before `baz` is set and it is a compile error.

A field that has been set can be read back mid-build, because the derive emits a
[`HasField`](./has_field.md) impl on the partial type gated on presence:

```rust
let partial = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());

assert_eq!(partial.get_field(PhantomData::<Symbol!("first_name")>), "Alice");
```

Asking for `last_name` there would not compile.

## When to use it

**Call `builder()` freely in concrete code; bound on the trait only in generic code.** In concrete code
you call it and never name a trait. The bound matters when you write code that is generic over the
record being built — which is the extensible builder pattern, and the reason the family exists.

- **Bound on `HasBuilder` + [`BuildField`](./build_field.md) + [`FinalizeBuild`](./finalize_build.md)**
  to write a routine that assembles some record it does not name.
- **Use [`IntoBuilder`](./into_builder.md)** when you start from an existing value rather than from
  nothing.
- **Do not reach for any of it for a struct you build with a literal.** A literal is already checked for
  completeness, reads better, and generates nothing. This family buys *decoupling*, and with nothing to
  decouple it is pure cost.

Two boundaries are worth stating plainly. This is **not a conventional builder**: it tracks presence and
nothing else — no defaults, no validation at finalize, no optional field unless the field's own type is
optional. The [optional-field extensions](./has_optional_builder.md) cover the defaulted and optional
cases, and a hand-written builder remains better when the *logic* is the point. And the enum counterparts
are a different family: [`ExtractField`](./extract_field.md) for taking a value apart and
[`FromVariant`](./from_variant.md) for constructing one.

## Under the hood

The derive generates a companion struct — `__Partial{Name}` — that is your struct with one
[`MapType`](./map_type.md) parameter added per field and each field's type wrapped in that parameter's
projection:

```rust
pub struct __PartialPerson<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub last_name: <__F1__ as MapType>::Map<String>,
}
```

`HasBuilder` then fixes the starting configuration to all-absent:

```rust
impl HasBuilder for Person {
    type Builder = __PartialPerson<IsNothing, IsNothing>;      // the empty builder
    // ...
}
```

and [`FinalizeBuild`](./finalize_build.md) exists only at the opposite end:

```rust
impl FinalizeBuild for __PartialPerson<IsPresent, IsPresent> {
    // ...
}
```

That pair is the whole safety argument. `builder()` starts at all-absent, each
[`build_field`](./build_field.md) flips one marker, and there is no check to run at the end — the impl
simply is not there for an incomplete value.

Everything between the two ends reduces to one primitive, [`UpdateField`](./update_field.md), which is
what the derive actually writes; [`BuildField`](./build_field.md) and [`TakeField`](./take_field.md) are
library blanket impls over it in opposite directions.

## Common Mistakes

**"No method named `finalize_build`" is the expected error for an incomplete build.** A missing field
means the all-present impl does not apply, so the compiler reports a missing method rather than a missing
field. Read the partial type in the message to see which marker is still `IsNothing`.

**The partial type cannot be printed or cloned.** The derive clears the original's attributes, so no
`Debug`, no `Clone`, whatever the record derives. Read a set field through the partial type's
[`HasField`](./has_field.md) impl instead.

**There are no defaults and no validation.** Presence is all that is tracked. A field with a sensible
default still has to be set, unless you reach for the
[optional-field extensions](./can_finalize_with_default.md).

**`builder()` is an associated function.** There is no receiver, so it is `Person::builder()` rather than
anything called on a value. Starting from a value is [`IntoBuilder`](./into_builder.md).

**A fieldless struct's builder is immediately finalizable**, since there is nothing to track. That is
legal and useless.

## Related constructs

- [`IntoBuilder`](./into_builder.md) — the other entry point, starting from a complete value.
- [`BuildField`](./build_field.md) — setting one absent field.
- [`TakeField`](./take_field.md) — the reverse, removing one present field.
- [`UpdateField`](./update_field.md) — the primitive both are built from.
- [`PartialData`](./partial_data.md) and [`FinalizeBuild`](./finalize_build.md) — naming the destination,
  and reaching it.
- [`CanBuildFrom`](./can_build_from.md) — merging every shared field from another record.
- [`#[derive(BuildField)]`](../derives/derive_build_field.md) — generates the partial type and every impl
  in the family.
- [`MapType`](./map_type.md) — the `IsPresent`/`IsNothing` markers presence is encoded in.
- [`HasField`](./has_field.md) — how a set field is read back off a partial value.
- [Optional fields](./has_optional_builder.md) — defaulted and optional finalization.
- [`ExtractField`](./extract_field.md) and [`FromVariant`](./from_variant.md) — the enum counterparts.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records and the extensible builder
  pattern.

## Source

- [`has_builder.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_builder.rs)
  — `HasBuilder` and `IntoBuilder`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
