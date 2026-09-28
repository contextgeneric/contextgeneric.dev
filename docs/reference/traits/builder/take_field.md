---
title: 'TakeField — remove one field of a builder'
sidebar_label: 'TakeField'
sidebar_position: 4
description: 'Take a present field out of a partial record, getting the value and a remainder that cannot be finalized until the field is put back.'
---

# `TakeField`

Removing one currently-present field from a builder.

## Overview

`TakeField` is the reverse of [`BuildField`](./build_field.md): it removes a field that is currently set,
handing back the value and the [partial record](/docs/reference/glossary#partial-record) without it. The value comes out owned, and the remainder is
the partial record with that one field flipped back to absent, so it can no longer be finalized until the
field is put back.

It pins the `IsPresent → IsNothing` transition of the [`UpdateField`](./update_field.md) primitive, and
because it *requires* the starting marker to be `IsPresent`, **taking a field that is absent is a compile
error** rather than an `Option` you have to handle.

Two uses reach for it. One is redistributing a record: [`into_builder`](./into_builder.md), take fields
out, put them somewhere else. The other, far more often, is [`CanBuildFrom`](../casting/can_build_from.md),
whose merge recursion takes each field out of the source and builds it into the target.

## Definition

`TakeField` is keyed by the field's `Tag`, with two associated types and one method:

```rust
pub trait TakeField<Tag> {
    type Value;
    type Remainder;

    fn take_field(self, _tag: PhantomData<Tag>) -> (Self::Value, Self::Remainder);
}
```

`Tag` names the field. `Value` is the field's declared type, returned owned. `Remainder` is the partial
record with that one field's marker flipped from `IsPresent` back to `IsNothing`, a different type from
`Self` that can no longer be finalized until the field is restored. `take_field` returns the two as a
tuple, the value first and the remainder second. The blanket impl over [`UpdateField`](./update_field.md)
that supplies this is shown in [Under the hood](#under-the-hood).

## Usage

**It is not in the prelude**, the one member of the core builder family that is not. Import it from
`cgp::core::field::traits` when you call `take_field` directly:

```rust
use cgp::core::field::traits::TakeField;
```

The `PhantomData<Tag>` argument names the field, and the method returns a tuple: **the value first, the
remainder second.**

**Nothing implements it directly.** It is a library blanket impl over
[`UpdateField`](./update_field.md), so every field the derive generates an `UpdateField` impl for
gains `take_field` without an impl of its own.

## Examples

Taking a field out of a full builder, changing it, and putting it back:

```rust
use cgp::prelude::*;
use cgp::core::field::traits::TakeField;

#[derive(Debug, PartialEq, BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

pub fn shout_first_name(person: Person) -> Person {
    let (first_name, remainder) = person
        .into_builder()
        .take_field(PhantomData::<Symbol!("first_name")>);

    remainder
        .build_field(
            PhantomData::<Symbol!("first_name")>,
            first_name.to_uppercase(),
        )
        .finalize_build()
}

pub fn demo() {
    let person = Person {
        first_name: "Alice".to_owned(),
        last_name: "Chen".to_owned(),
    };

    assert_eq!(shout_first_name(person).first_name, "ALICE");
}
```

Dropping the `build_field` step makes `finalize_build` fail to resolve, which is the guarantee: a
field taken out and not replaced cannot be forgotten silently.

## When to use it

**Reach for it when a complete value must be decomposed**, and remember that the common case,
merging one record into another, already uses it for you.

- **Use [`CanBuildFrom`](../casting/can_build_from.md)** to move every field of one record into
  another's builder. It is this trait applied in a loop, written once.
- **Use `take_field`** when a *specific* field must be extracted from a partial value, typically after
  [`into_builder`](./into_builder.md).
- **Use [`BuildField`](./build_field.md)** for the opposite direction.
- **Use [`ToFields`](../shape/to_fields.md)** when the whole value should become a flat shape rather than a
  partial record with one field missing.
- **Prefer ordinary destructuring** in concrete code. `let Person { first_name, .. } = person;`
  needs none of this machinery, and this family is for code that cannot name the type.

## Under the hood

`TakeField` is a library blanket impl over [`UpdateField`](./update_field.md), the mirror image of
[`BuildField`](./build_field.md#under-the-hood):

```rust
impl<Context, Tag> TakeField<Tag> for Context
where
    Context: UpdateField<Tag, IsNothing, Mapper = IsPresent>,
{
    type Value = Context::Value;

    type Remainder = Context::Output;

    fn take_field(self, tag: PhantomData<Tag>) -> (Self::Value, Self::Remainder) {
        self.update_field(tag, ())
    }
}
```

The target marker is `IsNothing` and the constraint on the reported source marker is `IsPresent`,
both swapped relative to `BuildField`, so under this transition the primitive takes `()` in and
hands the real value back. **Its heaviest user is [`CanBuildFrom`](../casting/can_build_from.md)**,
whose recursion walks the source's field list, taking each field out with `take_field` and writing
it into the target with [`build_field`](./build_field.md), which is why a merge needs
[`HasFields`](../shape/has_fields.md) on the source as well as a builder.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::traits`. Every other trait in the core
builder family is.

**Taking an absent field does not compile.** Taking `first_name` from an empty builder:

```rust
let _ = Person::builder().take_field(PhantomData::<Symbol!("first_name")>);
```

finds `Mapper = IsNothing` where the blanket impl requires `IsPresent`, reported as a mismatch
rather than as anything about absence:

```text
error[E0271]: type mismatch resolving `<__PartialPerson<IsNothing, IsNothing> as UpdateField<Symbol<10, Chars<'f', Chars<'i', Chars<'r', Chars<'s', Chars<'t', Chars<'_', Chars<'n', Chars<'a', Chars<'m', Chars<'e', Nil>>>>>>>>>>>, IsNothing>>::Mapper == IsPresent`
...
note: expected this to be `IsPresent`
```

**It returns `(value, remainder)`, in that order.** Reversing the binding is a type error, but a
confusing one when both are generic.

**The remainder cannot be finalized.** That is the point, and it means a decomposition that loses a field
fails at [`finalize_build`](./finalize_build.md) rather than where the field was dropped.

**`Remainder` is a different type from `Self`**, so a take cannot happen in a loop over a fixed-type
variable.

**Nothing implements it directly**, so an unsatisfied `TakeField` bound surfaces as an unsatisfied
[`UpdateField`](./update_field.md) one.

## Related constructs

- [`BuildField`](./build_field.md): the opposite direction.
- [`UpdateField`](./update_field.md): the primitive both are built from.
- [`IntoBuilder`](./into_builder.md): how a complete value becomes a partial one to take from.
- [`CanBuildFrom`](../casting/can_build_from.md): the merge that uses this internally.
- [`FinalizeBuild`](./finalize_build.md): what a remainder cannot satisfy until the field is restored.
- [`HasBuilder`](./has_builder.md): the family's entry point.
- [`MapType`](../type-level/map_type.md): the `IsPresent`/`IsNothing` markers this moves between.
- [`ToFields`](../shape/to_fields.md): the flat-shape alternative to decomposing a partial value.
- [`#[derive(BuildField)]`](../../derives/derive_build_field.md): generates the `UpdateField` impls behind
  it.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and the extensible builder
  pattern.

## Source

- [`take_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/take_field.rs):
  `TakeField`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
