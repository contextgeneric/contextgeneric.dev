---
title: 'Enum! — enum shapes'
sidebar_label: 'Enum!'
sidebar_position: 20
description: 'The type-level shape of an enum, written as the body of an enum declaration.'
---

# `Enum!`

The type-level shape of an enum, written as the body of an enum declaration.

## Overview

Generic code over enums works on an enum's **shape**: one type that lists every variant by name,
which [`#[derive(HasFields)]`](../derives/derive_has_fields.md) generates for an enum as its `Fields`
type. Written out by hand, that shape is a [`Sum!`](./sum.md) of [`Field`](../types/field.md)
entries that repeats each variant name as a
[type-level string](/docs/reference/glossary#type-level-string):

```rust
Sum![Field<Symbol!("Circle"), f64>, Field<Symbol!("Square"), f64>]
```

`Enum!` writes the same type as the enum body it describes:

```rust
Enum! { Circle(f64), Square(f64) }
```

The two are the identical type, and it is exactly the `Fields` the derive gives an enum with that
body. Reach for `Enum!` wherever code *names* a variant set's shape rather than deriving it, such as
a [trait bound](https://doc.rust-lang.org/book/ch10-02-traits.html) on `FromFields`. It is also the
form [`cargo cgp`](/docs/cargo-cgp) prints a variant list in, so a list read in the tool's output can
be copied back into code.

## Usage

The body is the inside of an enum declaration, and each variant takes one of the three Rust variant
shapes:

```rust
Enum! {
    Empty,                                  // a unit variant
    Circle(f64),                            // positional fields
    Rectangle { width: f64, height: f64 },  // named fields
}
```

Inside the body, a variant's brackets keep their Rust meaning: parentheses hold positional fields,
and braces hold named fields. The brackets around the whole `Enum!` invocation do not matter. A
variant name may be a raw identifier, tagged without the `r#`, and the list accepts a trailing comma.
An empty body, `Enum! {}`, is allowed.

`Enum!` builds a type, never a value. A value of the shape selects its variant with `Either::Left`
and `Either::Right`, wrapping the payload with `Field::from`, or comes from an existing enum through
`to_fields()`.

## Examples

A function bounded on an enum's shape converts a value built in that shape into the enum. Here
`Value` derives the whole variant family, and `from_shape` works for any enum with the same variants:

```rust
use cgp::prelude::*;

#[derive(Debug, PartialEq, CgpData)]
pub enum Value {
    Int(u64),
    Text(String),
}

pub fn from_shape<T>(fields: Enum! { Int(u64), Text(String) }) -> T
where
    T: FromFields<Fields = Enum! { Int(u64), Text(String) }>,
{
    T::from_fields(fields)
}

assert_eq!(from_shape::<Value>(Either::Left(Field::from(7))), Value::Int(7));
assert_eq!(
    from_shape::<Value>(Either::Right(Either::Left(Field::from("seven".to_owned())))),
    Value::Text("seven".to_owned()),
);
```

`Either::Left` selects the first variant and each `Either::Right` moves one variant further, and the
variant's name comes from the `Enum!` type the argument is expected to have.

## When to use it

**Write `Enum!` whenever you would otherwise spell a variant list by hand.** A
`Sum![Field<Symbol!("Int"), u64>, …]` written out is longer and hides the fact that it describes an
enum.

- **Let [`#[derive(HasFields)]`](../derives/derive_has_fields.md) or
  [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) produce an enum's shape**, and use `Enum!`
  only where code has to name one. Restating an enum you own as an `Enum!` next to it is a second copy
  that can drift.
- **Use [`Struct!`](./struct.md) for a record's shape**, where every field is present at once.
- **Use [`Sum!`](./sum.md) for a choice among types that are not named variants.**
- **Use a plain `enum` and `match` when the variant set is closed and consumed in one place.** A shape
  is worth naming when independent code handles variants it did not define.

## Under the hood

`Enum!` expands exactly as `#[derive(HasFields)]` expands the same body, because the macro runs the
derive's own code. Each variant becomes a `Field` entry keyed by its name, and the entries are chained
into a sum that ends in `Void`:

```rust
// before
Enum! { Circle(f64), Square(f64) }

// after
Either<Field<Symbol!("Circle"), f64>, Either<Field<Symbol!("Square"), f64>, Void>>
```

A variant's payload is its fields, encoded by the [`Struct!`](./struct.md#under-the-hood) rules:

| Variant | Payload |
| --- | --- |
| `Empty`, `Empty()`, or `Empty {}` | `Nil` |
| `Circle(f64)` | `f64`, the bare type |
| `Pair(u32, u32)` | `Struct!(u32, u32)`, a list keyed by position |
| `Rect { width: f64, height: f64 }` | `Struct! { width: f64, height: f64 }`, a list keyed by name |

So several spellings are the same type: `V`, `V()`, `V {}`, and `V(Nil)` are one variant, and
`V { a: u32 }` is `V(Struct! { a: u32 })`. `cargo cgp` relies on this when it prints a variant list,
choosing the shortest spelling. An empty body is `Void`, the shape of an enum with no variants.

## Formal grammar

The body is a list of variants, each a name with an optional field list, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
EnumInput -> ( Variant ( `,` Variant )* `,`? )?

Variant   -> IDENTIFIER ( `(` TupleFields `)` | `{` NamedFields `}` )?
```

`IDENTIFIER` includes raw identifiers. `TupleFields` and `NamedFields` are the
[`Struct!` productions](./struct.md#formal-grammar), so a variant's fields follow the rules of a
`Struct!` body of the same form.

## Common Mistakes

**The variant derives need each variant to hold exactly one positional field.** `Enum!` describes any
variant, as `#[derive(HasFields)]` does, but [`#[derive(CgpData)]`](../derives/derive_cgp_data.md),
`CgpVariant`, `ExtractField`, and `FromVariant` reject a unit, multi-field, or named-field variant. A
shape with such variants therefore has no generic constructor or extractor.

**An enum body's other parts are rejected.** An attribute on a variant (such as `#[default]` or a doc
comment), `pub` on a variant, a discriminant such as `= 1`, a variant name given twice, and any field
`Struct!` would reject each fail with their own error.

**A variant list tagged by position has no `Enum!` spelling.** A variant needs a name, so a sum whose
entries are keyed by `Index<N>` is written with `Sum!`.

**Clippy's `type_complexity` lint, and an imported abstract type inside the body,** behave as they do
for [`Struct!`](./struct.md#common-mistakes).

## Related constructs

- [`Struct!`](./struct.md) — the struct counterpart, whose rules encode each variant's payload.
- [`Sum!`](./sum.md) — the sum `Enum!` expands to.
- [`Field`](../types/field.md) and [`Symbol!`](./symbol.md) — the entry and the tag each variant
  becomes.
- [`HasFields`](../traits/shape/has_fields.md) — the trait whose `Fields` an `Enum!` names.
- [`#[derive(HasFields)]`](../derives/derive_has_fields.md) — the derive whose encoding `Enum!` runs.
- [`FromVariant`](../traits/variant/from_variant.md) and
  [`ExtractField`](../traits/variant/extract_field.md) — constructing and taking apart a variant
  generically.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — the variant representation a shape
  describes.

## Source

- Entry point: [`enum_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/enum_type.rs)
- Parsing the body: [`types/shape/enum_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/shape/enum_type.rs)
- The encoding shared with the derive: [`derive_has_fields/sum.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields/sum.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
