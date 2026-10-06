---
title: 'Struct! — struct shapes'
sidebar_label: 'Struct!'
sidebar_position: 19
description: 'The type-level shape of a struct, written as the body of a struct declaration.'
---

# `Struct!`

The type-level shape of a struct, written as the body of a struct declaration.

## Overview

Generic code over records works on a struct's **shape**: one type that lists every field by name,
which [`#[derive(HasFields)]`](../derives/derive_has_fields.md) generates for a struct as its
`Fields` type. Written out by hand, that shape is a [`Product!`](./product.md) of
[`Field`](../types/field.md) entries that repeats each field name as a
[type-level string](/docs/reference/glossary#type-level-string):

```rust
Product![Field<Symbol!("name"), String>, Field<Symbol!("age"), u8>]
```

`Struct!` writes the same type as the struct body it describes:

```rust
Struct! { name: String, age: u8 }
```

The two are the identical type, and it is exactly the `Fields` the derive gives a struct with that
body. Reach for `Struct!` wherever code *names* a shape rather than deriving it: a
[trait bound](https://doc.rust-lang.org/book/ch10-02-traits.html) on `HasFields`, a trait
implemented for a shape, or a wiring entry. It is also the form
[`cargo cgp check`](/docs/cargo-cgp/check) and [`cargo cgp expand`](/docs/cargo-cgp/expand) print
a shape in, so a shape read in the tool's output can be copied back into code.

## Usage

The body is the inside of a struct declaration, in either of its two forms:

```rust
Struct! { name: String, age: u8 }   // like `struct S { name: String, age: u8 }`
Struct!(u64, String)                // like `struct S(u64, String);`
Struct! {}                          // an empty body
```

**The form comes from the entries, not from the brackets.** A procedural macro never sees the
delimiter it was invoked with, so `Struct!(a: u8)` is the named form and `Struct! { u8, bool }` is
the tuple form. Write braces for named fields and parentheses for a tuple body, by convention. An
entry is a named field when it starts with a name followed by a single `:`, so a path type such as
`core::marker::PhantomData<u8>` is a positional field.

Both forms accept a trailing comma. A field name may be a raw identifier, and a name that is a
keyword such as `type` must be written as one. Its tag is the name without the `r#`:
`Struct! { r#type: u8 }` tags its field `"type"`, as the derive does. A field type is any Rust
type, including generic parameters, references with
[lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html), and
[associated-type](https://doc.rust-lang.org/reference/items/associated-items.html) projections.

Two bodies give a type you might not expect, because `Struct!` follows the derive. A body with
**exactly one positional field** is that field's type itself, so `Struct!(u64)` is plain `u64`
rather than a list with one entry, and an **empty body** is `Nil`, the empty list. A single named
field still makes a list. [Under the hood](#under-the-hood) shows both expansions.

`Struct!` builds a type, never a value. A value of the shape comes from an existing struct through
`to_fields()`, or is built with [`product!`](./product.md), wrapping each field value with `.into()`.

## Examples

A function generic over every struct with a given shape bounds the shape directly. Here `rebuild`
turns a value of the shape into any type whose `Fields` it is:

```rust
use cgp::prelude::*;

#[derive(Debug, PartialEq, HasFields)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

pub fn rebuild<T>(fields: Struct! { name: String, age: u8 }) -> T
where
    T: FromFields<Fields = Struct! { name: String, age: u8 }>,
{
    T::from_fields(fields)
}

let person: Person = rebuild(product!["Carol".to_owned().into(), 25u8.into()]);
assert_eq!(person.age, 25);
```

Each `.into()` wraps one value in its `Field`, and the field's name comes from the `Struct!` type
the argument is expected to have.

A shape is an ordinary type, so a trait can be implemented for it, in either form:

```rust
pub trait Describe {
    fn describe() -> &'static str;
}

impl Describe for Struct! { name: String, age: u8 } {
    fn describe() -> &'static str {
        "a person's shape"
    }
}

impl Describe for Struct!(u8, u16) {
    fn describe() -> &'static str {
        "a pair's shape"
    }
}
```

A shape can also key an `open` dispatch entry in [`delegate_components!`](./delegate_components.md),
so `App` chooses a provider by the structure of a record. This fragment assumes a component
`CanDescribeShape<Shape>` and its providers `DescribePoint` and `DescribePair`, defined elsewhere:

```rust
delegate_components! {
    App {
        open ShapeDescriberComponent;

        @ShapeDescriberComponent.Struct! { x: f64, y: f64 }: DescribePoint,
        @ShapeDescriberComponent.Struct!(u8, u16): DescribePair,
    }
}
```

## When to use it

**Write `Struct!` whenever you would otherwise spell a field list by hand.** A
`Product![Field<Symbol!("name"), String>, …]` written out is longer, repeats every name as a string,
and hides the fact that it describes a record.

- **Let [`#[derive(HasFields)]`](../derives/derive_has_fields.md) produce a struct's shape**, and use
  `Struct!` only where code has to name a shape: in a bound, an impl, or a wiring entry. Restating a
  struct you own as a `Struct!` next to it is a second copy that can drift.
- **Use [`Enum!`](./enum.md) for an enum's shape**, a choice among variants rather than a record.
- **Use [`Product!`](./product.md) for a list that is not a set of named fields**, such as a handler
  pipeline, or for the one field list `Struct!` cannot spell, described under
  [Common Mistakes](#common-mistakes).
- **Use a plain struct when no generic code walks the fields.** A shape is worth naming when
  independent code builds, reads, or converts records by their field names.

## Under the hood

`Struct!` expands exactly as `#[derive(HasFields)]` expands the same body, because the macro runs
the derive's own code. Named fields become `Field` entries keyed by `Symbol!`, in declaration
order:

```rust
// before
Struct! { name: String, age: u8 }

// after
Cons<Field<Symbol!("name"), String>, Cons<Field<Symbol!("age"), u8>, Nil>>
```

Positional fields become `Field` entries keyed by [`Index<N>`](../types/index_type.md):

```rust
// before
Struct!(u64, String)

// after
Cons<Field<Index<0>, u64>, Cons<Field<Index<1>, String>, Nil>>
```

Two bodies follow special rules, both taken from the derive. A body with **exactly one positional
field** is that field's type, with no `Field` and no list, so `Struct!(u64)` is `u64`: the same
newtype rule that makes the shape of `struct Wrap(u64);` plain `u64`. An **empty body** is `Nil`,
the shape of a unit struct. A single *named* field is not unwrapped: `Struct! { value: u64 }` is the
one-element list `Cons<Field<Symbol!("value"), u64>, Nil>`.

Every CGP name in the expansion is written with its full path, so the macro works in a module that
imports nothing else from `cgp`.

## Formal grammar

The body is either a list of named fields or a list of types, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
StructInput -> NamedFields | TupleFields

NamedFields -> ( NamedField ( `,` NamedField )* `,`? )?

NamedField  -> IDENTIFIER `:` Type

TupleFields -> ( Type ( `,` Type )* `,`? )?
```

`IDENTIFIER` includes raw identifiers and excludes `_`, and `Type` is the Rust grammar's type
production. The empty body matches both alternatives, and both give the same type.

## Common Mistakes

**A one-element positional list has no `Struct!` spelling.** `Struct!(T)` is `T`, by the newtype
rule, so the list holding a single `Field<Index<0>, T>` is written
`Product![Field<Index<0>, T>]`. A trailing comma does not help: `Struct!(T,)` is `T` as well, unlike
the tuple type `(T,)`.

**The shape is a type, not a value.** A literal where a type belongs is rejected:

```text
error: expected a type: a type-level shape lists field types, not values
```

Build a value with `product!` and `.into()`, or with `to_fields()`.

**A struct body's other parts are rejected.** Attributes (doc comments included), visibility such as
`pub`, a field named `_`, a field name given twice, and a body that mixes named and positional entries
each fail with their own error, since a shape has no use for them.

**A keyword field name needs its `r#`.** A keyword is not an identifier, so `Struct! { type: u8 }`
is rejected:

```text
error: `type` is a keyword: write the field name as `r#type`
```

Write `Struct! { r#type: u8 }`, which tags the field `"type"`. The four keywords that have no raw
form, `self`, `Self`, `super`, and `crate`, cannot name a field at all.

**Clippy's `type_complexity` lint can fire on an ordinary shape.** Clippy measures the expanded type,
in which every field name is a nested type-level string, so a shape nested in a larger type in a
signature can cross the lint's threshold. A `type` alias for the shape, or
`#[allow(clippy::type_complexity)]`, silences it.

**An imported abstract type is not rewritten inside the body.** In an item that imports `Error` with
[`#[use_type(HasErrorType.Error)]`](../attributes/use_type.md), a bare `Error` written inside
`Struct! { … }` is left as it is and fails as an unknown type. Write
`<Self as HasErrorType>::Error` inside the macro instead.

## Related constructs

- [`Enum!`](./enum.md) — the enum counterpart, whose variant payloads follow the `Struct!` rules.
- [`Product!`](./product.md) — the list `Struct!` expands to.
- [`Field`](../types/field.md) — the entry each field becomes.
- [`Symbol!`](./symbol.md) and [`Index`](../types/index_type.md) — the name and position tags.
- [`HasFields`](../traits/shape/has_fields.md) — the trait whose `Fields` a `Struct!` names.
- [`#[derive(HasFields)]`](../derives/derive_has_fields.md) — the derive whose encoding `Struct!`
  runs.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — the record representation a shape
  describes.

## Source

- Entry point: [`struct_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/struct_type.rs)
- Parsing the body: [`types/shape/struct_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/shape/struct_type.rs) and [`functions/shape/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/functions/shape)
- The encoding shared with the derive: [`derive_has_fields/product.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields/product.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
