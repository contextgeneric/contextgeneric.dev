---
title: 'BuildField — set one field of a builder'
sidebar_label: 'BuildField'
sidebar_position: 3
description: 'Set a currently-absent field on a partial record, which is the direction of the builder family you write most when assembling a struct field by field.'
---

# `BuildField`

Setting one currently-absent field of a builder.

## Overview

`BuildField` is the direction of the [builder family](./has_builder.md) you write most: take a partial
record with a field absent, supply the value, and get back a [partial record](/docs/reference/glossary#partial-record) with that field present.

`Output` is a **different type** from `Self` (the same partial record with one marker flipped), which is
what makes the compiler track completeness. A chain of `build_field` calls walks through as many distinct
types as there are fields, and only the last one satisfies
[`FinalizeBuild`](./finalize_build.md).

It pins one transition of the [`UpdateField`](./update_field.md) primitive: `IsNothing → IsPresent`.
Because it *requires* the starting marker to be `IsNothing`, **building a field that is already set is a
compile error**, not a silent overwrite.

## Definition

`BuildField` is keyed by the field's `Tag`, with two associated types and one method:

```rust
pub trait BuildField<Tag> {
    type Value;
    type Output;

    fn build_field(self, _tag: PhantomData<Tag>, value: Self::Value) -> Self::Output;
}
```

`Tag` names the field. `Value` is the field's declared type, so `build_field` takes an ordinary
value without a marker wrapper. `Output` is the partial record with that one field's marker flipped
from `IsNothing` to `IsPresent`, a different type from `Self`. The blanket impl over
[`UpdateField`](./update_field.md) that supplies all of this is shown in [Under the
hood](#under-the-hood).

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

The `PhantomData<Tag>` argument names the field, the same way a
[`HasField`](../field-access/has_field.md) read does: `PhantomData::<Symbol!("first_name")>` for a
named field, `PhantomData::<Index<0>>` for a tuple-struct position. `Value` is the field's declared
type, so what you pass is an ordinary value without a wrapper.

**Nothing implements it directly.** It is a library blanket impl over
[`UpdateField`](./update_field.md), so every field the derive generates an `UpdateField` impl for
gains `build_field` without an impl of its own.

## Examples

A record built one field at a time, in either order, and a wider record built from it plus one
field:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::CanBuildFrom;

#[derive(Debug, PartialEq, HasFields, BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, PartialEq, BuildField)]
pub struct Employee {
    pub first_name: String,
    pub last_name: String,
    pub employee_id: u64,
}

pub fn demo() {
    // Order does not matter: `last_name` is set first here.
    let person = Person::builder()
        .build_field(PhantomData::<Symbol!("last_name")>, "Chen".to_owned())
        .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
        .finalize_build();

    let employee = Employee::builder()
        .build_from(person)
        .build_field(PhantomData::<Symbol!("employee_id")>, 7)
        .finalize_build();

    assert_eq!(
        employee,
        Employee {
            first_name: "Alice".to_owned(),
            last_name: "Chen".to_owned(),
            employee_id: 7,
        }
    );
}
```

Each impl constrains only its own field's marker, so setting `last_name` first compiles identically.
[`build_from`](../casting/can_build_from.md) copies the shared fields from `person`, which is why
`Person` also derives [`HasFields`](../shape/has_fields.md).

## When to use it

**Call `build_field` in any code that fills a builder; bound on the trait when the code is generic over
the record.**

- **Bound on [`HasBuilder`](./has_builder.md) + `BuildField` + [`FinalizeBuild`](./finalize_build.md)**
  to write a routine that assembles a record it does not name.
- **Use [`CanBuildFrom`](../casting/can_build_from.md)** to copy every shared field from another record at once,
  rather than one `build_field` per field.
- **Use [`SetOptional`](../optional/set_optional.md)** when a field must be settable more than once. `build_field`
  consumes an absent slot exactly once by design; the optional layer relaxes that.
- **Use [`UpdateField`](./update_field.md)** for a transition this does not cover.
- **Prefer a struct literal** in concrete code that knows every field.

## Under the hood

`BuildField` is a library blanket impl over [`UpdateField`](./update_field.md), pinning the target
marker to `IsPresent` and constraining the reported source marker to `IsNothing`:

```rust
impl<Context, Tag> BuildField<Tag> for Context
where
    Context: UpdateField<Tag, IsPresent, Mapper = IsNothing>,
{
    type Value = Context::Value;

    type Output = Context::Output;

    fn build_field(self, tag: PhantomData<Tag>, value: Self::Value) -> Self::Output {
        self.update_field(tag, value).1
    }
}
```

The `Mapper = IsNothing` constraint does the checking. `Mapper` is an output of the primitive, the
marker the field was in, so constraining it selects only the partial types whose field is absent. A
field already set has `Mapper = IsPresent`, and the constraint fails. `build_field` discards the old
value the primitive returns, which is sound because under `IsNothing` it is `()`.
[`TakeField`](./take_field.md) is the mirror image: the same primitive, with `IsNothing` as the
target and `Mapper = IsPresent`.

## Common Mistakes

**Building a field twice does not compile.** On the two-field `Person` of the
[example](#examples), setting `first_name` a second time:

```rust
let _ = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
    .build_field(PhantomData::<Symbol!("first_name")>, "Bob".to_owned());
```

makes the second call find `Mapper = IsPresent` where the blanket impl requires `IsNothing`, and
rustc reports the mismatch on the partial type, whose markers show the field already set:

```text
error[E0271]: type mismatch resolving `<__PartialPerson<IsPresent, IsNothing> as UpdateField<Symbol<10, Chars<'f', Chars<'i', Chars<'r', Chars<'s', Chars<'t', Chars<'_', Chars<'n', Chars<'a', Chars<'m', Chars<'e', Nil>>>>>>>>>>>, IsPresent>>::Mapper == IsNothing`
...
note: expected this to be `IsNothing`
```

Nothing in the message says "already set"; the first `IsPresent` in the partial type does.

**`Output` is a different type from `Self`.** A builder cannot be stored in a variable of fixed type
across a chain, and it cannot be filled in a loop. The chain is unrolled by construction.

**The error for an incomplete build arrives at `finalize_build`**, not here. Forgetting a field is only
detectable at the end, since any prefix of a chain is a legal partial value.

**It sets, it does not overwrite.** For a settable-and-resettable field, reach for
[`SetOptional`](../optional/set_optional.md).

**The `PhantomData` argument carries the tag.** At a site where inference cannot determine it, write
`PhantomData::<Symbol!("name")>` in full.

**Nothing implements it directly**, so an unsatisfied `BuildField` bound is really an unsatisfied
[`UpdateField`](./update_field.md) one, which the error names, as a mismatch on `Mapper` for a field
already set or a missing impl for a field the record lacks.

## Related constructs

- [`UpdateField`](./update_field.md): the primitive this pins one direction of.
- [`TakeField`](./take_field.md): the opposite direction.
- [`HasBuilder`](./has_builder.md) and [`IntoBuilder`](./into_builder.md): where a partial value comes
  from.
- [`FinalizeBuild`](./finalize_build.md): where a completed one goes.
- [`CanBuildFrom`](../casting/can_build_from.md): filling many fields from another record in one call.
- [`SetOptional`](../optional/set_optional.md): the settable-repeatedly counterpart.
- [`MapType`](../type-level/map_type.md): the `IsNothing`/`IsPresent` markers this moves between.
- [`#[derive(BuildField)]`](../../derives/derive_build_field.md): generates the `UpdateField` impls behind
  it.
- [`Symbol!`](../../macros/symbol.md) and [`Index`](../../types/index_type.md): the tags that name a field.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): partial records and the extensible builder
  pattern.

## Source

- [`build_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/build_field.rs):
  `BuildField` and `FinalizeBuild`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
