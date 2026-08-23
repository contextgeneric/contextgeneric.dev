---
sidebar_label: 'FromFields'
---

# `FromFields`

Rebuilding a concrete value from its shape.

## Overview

[`ToFields`](./to_fields.md) takes a value apart into an anonymous list of named entries. `FromFields` is
the return journey:

```rust
pub trait FromFields: HasFields {
    fn from_fields(fields: Self::Fields) -> Self;
}
```

It is what closes the loop for generic code: decompose a value, work on the entries without naming the
type, and put a concrete value back together at the end. Both directions go through the identical
`Fields` type, so **the shapes line up by construction rather than by check** — there is nothing to
validate and nothing that can fail.

It is an associated function rather than a method, since there is no value to call it on. The call reads
`Person::from_fields(fields)`, or `T::from_fields(fields)` in generic code.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

It supertraits [`HasFields`](./has_fields.md), so bounding on `FromFields` gives you `Fields` as well.
The impls come from [`#[derive(HasFields)]`](../derives/derive_has_fields.md), which emits all five shape
traits together.

The argument must be **exactly** `Self::Fields` — the same entries, the same tags, in the same order. A
shape that merely has the same field names in a different order is a different type and will not be
accepted, which is what makes the conversion total.

## Examples

Rebuilding after a round trip:

```rust
use cgp::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

let config = Config { host: "localhost".to_owned(), port: 8080 };

let fields = config.clone().to_fields();
let config_again = Config::from_fields(fields);

assert_eq!(config, config_again);
```

In generic code it is the half that names the destination:

```rust
fn rebuild<T>(fields: T::Fields) -> T
where
    T: FromFields,
{
    T::from_fields(fields)
}
```

An enum rebuilds the same way, from a [`Sum!`](../macros/sum.md) rather than a product — the arm that is
present becomes the variant that is constructed.

## When to use it

**Bound on it when generic code produces a concrete value from entries it has assembled or transformed.**
Pair it with [`ToFields`](./to_fields.md) for a round trip, and require only one of the two when only one
direction happens.

- **[`ToFields`](./to_fields.md)** for the decomposition half.
- **[`HasFields`](./has_fields.md) alone** when the code names the shape and never builds a value.
- **The [builder family](./has_builder.md)** when the value is assembled *incrementally*, from pieces
  arriving at different times. `from_fields` needs the whole shape at once, already complete;
  [`HasBuilder`](./has_builder.md) tracks presence field by field and is what the extensible builder
  pattern uses.
- **[`FromVariant`](./from_variant.md)** when an enum is built from *one* named variant rather than from
  a whole shape. That is the far more common way to construct an enum generically.
- **A plain constructor** when the type is concrete. Nothing about `from_fields` improves on a struct
  literal where one can be written.

## Under the hood

The conversion destructures the `Cons` chain positionally and unwraps each entry's value:

```rust
impl FromFields for Person {
    fn from_fields(Cons(name, Cons(age, Nil)): Self::Fields) -> Self {
        Self { name: name.value, age: age.value }
    }
}
```

Note the pattern in the argument position: the shape is matched apart in the signature itself, one node
per field, terminated by `Nil`. Because the chain is built and matched in declaration order, this is the
exact inverse of [`to_fields`](./to_fields.md) — no lookup by name happens at run time, and the tags
exist only to make the types distinct.

An enum's impl is the dual: a `match` over the `Either` chain, each arm reconstructing the corresponding
variant, with the `Void` terminator unreachable by construction.

## Common Mistakes

**It is an associated function, not a method.** Write `T::from_fields(fields)`; there is no receiver.

**The shape must match exactly.** Same tags, same value types, same order. Two structs with identical
field names in different orders have unrelated `Fields` types, and the mismatch is reported against the
whole chain rather than against the field that moved.

**A newtype's shape is the inner type**, not a one-element product, so `from_fields` on
`struct Wrapper(String)` takes a `String`.

**It supertraits [`HasFields`](./has_fields.md)**, so naming both in a bound is redundant.

**It cannot build a value incrementally.** The whole shape is required at once. Assembling from
independent pieces is [`HasBuilder`](./has_builder.md)'s job, and reaching for `from_fields` there means
constructing the complete product by hand first.

**There is no borrowing counterpart.** [`ToFieldsRef`](./to_fields_ref.md) has no `FromFieldsRef`, because
a borrowed shape cannot yield an owned value.

## Related constructs

- [`ToFields`](./to_fields.md) — the reverse conversion, and the one usually written first.
- [`HasFields`](./has_fields.md) — the supertrait that names the shape.
- [`ToFieldsRef`](./to_fields_ref.md) and [`HasFieldsRef`](./has_fields_ref.md) — the borrowed half of
  the family, which has no rebuild.
- [`FromVariant`](./from_variant.md) — constructing an enum from one named variant.
- [`HasBuilder`](./has_builder.md) — incremental assembly, as against this wholesale conversion.
- [`#[derive(HasFields)]`](../derives/derive_has_fields.md) — generates this impl.
- [`Product!`](../macros/product.md), [`Sum!`](../macros/sum.md), and [`Field`](../types/field.md) — what
  a shape is made of.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum half.

## Source

- [`from_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/from_fields.rs)
  — `FromFields`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
