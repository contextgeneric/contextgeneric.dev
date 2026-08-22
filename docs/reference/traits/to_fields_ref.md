---
sidebar_label: 'ToFieldsRef'
---

# `ToFieldsRef`

Walking a value's shape without consuming it.

## Overview

[`ToFields`](./to_fields.md) takes a value apart into its shape and consumes it in the process. Code that
only reads — a validator, a serializer, a routine that inspects a struct and hands it back — should not
have to. `ToFieldsRef` produces the **borrowed** shape instead:

```rust
pub trait ToFieldsRef: HasFieldsRef {
    fn to_fields_ref<'a>(&'a self) -> Self::FieldsRef<'a>
    where
        Self: 'a;
}
```

The original survives, and every entry in the result holds a reference rather than a value.

**It is the weaker requirement, so prefer it wherever it suffices.** Bounding on
[`ToFields`](./to_fields.md) when a borrow would do forces every caller to give up ownership or clone,
which is the most common over-requirement in this family.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

It supertraits [`HasFieldsRef`](./has_fields_ref.md) rather than [`HasFields`](./has_fields.md), which
is the one structural difference from its owning counterpart: the borrowed shape and the owned shape are
named by two independent traits, and each conversion supertraits the one it produces.

The impls come from [`#[derive(HasFields)]`](../derives/derive_has_fields.md), which emits all five shape
traits together.

## Examples

Reading a shape and keeping the value:

```rust
use cgp::prelude::*;

#[derive(HasField, HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

let config = Config { host: "localhost".to_owned(), port: 8080 };

let fields_ref = config.to_fields_ref();

assert_eq!(fields_ref.0.value, &"localhost".to_owned());
```

`config` is still usable afterwards, which is the whole difference from
[`to_fields`](./to_fields.md).

In generic code it is the bound that says "I will not take your value":

```rust
fn inspect<T>(value: &T)
where
    T: ToFieldsRef,
{
    let _fields = value.to_fields_ref();
    // walk the borrowed entries
}
```

## When to reach for it, and when not

**Bound on it whenever generic code reads a shape and the caller keeps the value** — which is most
read-only structural code.

- **[`ToFields`](./to_fields.md)** when the code genuinely consumes: a conversion, a merge, a rebuild.
- **[`FromFields`](./from_fields.md)** for constructing a value, which has no borrowing counterpart —
  a borrowed shape cannot yield an owned value.
- **[`HasFieldsRef`](./has_fields_ref.md) alone** when the code only *names* the borrowed shape.
- **[`HasField`](./has_field.md)** when one named field is all that is wanted.

There is one thing this cannot do that [`ToFields`](./to_fields.md) can: hand the entries' values onward
by ownership. A routine that must move a field out of a struct needs the owning form.

## Under the hood

The generated impl borrows each field and wraps it into the corresponding entry of the borrowed shape,
under the reserved lifetime name `'__a` that
[`HasFieldsRef`](./has_fields_ref.md#under-the-hood) declares:

```rust
// each field is borrowed rather than moved:
//   Cons((&self.name).into(), Cons((&self.age).into(), Nil))
```

The `where Self: 'a` clause on both the trait method and the associated type is what keeps the result
tied to the borrow it came from, so the shape cannot outlive the value.

One consequence follows from the rewrite being per-entry rather than structural: **a field that is
already a reference gains another one**, appearing as `&'__a &'a Name`. That is correct, and it is the
detail most likely to look wrong in an error message.

An enum's borrowed conversion matches the concrete variant and produces the corresponding arm of the
borrowed sum, with the payload borrowed rather than moved — which is also how
[`HasExtractorRef`](./has_extractor_ref.md) reads a variant without consuming the value.

## Common Mistakes

**It supertraits [`HasFieldsRef`](./has_fields_ref.md), not [`HasFields`](./has_fields.md).** Bounding on
`ToFieldsRef` does not give you `Fields`; require both traits if the code needs both shapes.

**A field that is already borrowed gets a second borrow** in the result, which is correct and surprising.

**There is no `FromFieldsRef`.** The borrowed half of the family is read-only by construction.

**The lifetime usually has to be spelled out.** A signature that returns or stores `FieldsRef<'a>`
inherits the `where Self: 'a` clause and rarely elides cleanly.

**A newtype's borrowed shape is a borrow of the inner type**, not a one-element product — the same
special case the owned shape has.

## Related constructs

- [`HasFieldsRef`](./has_fields_ref.md) — the supertrait that names the borrowed shape.
- [`ToFields`](./to_fields.md) — the owning counterpart, which consumes the value.
- [`FromFields`](./from_fields.md) — rebuilding a value, which has no borrowing form.
- [`HasFields`](./has_fields.md) — the owned shape, and where the family is explained in full.
- [`#[derive(HasFields)]`](../derives/derive_has_fields.md) — generates this impl.
- [`HasExtractorRef`](./has_extractor_ref.md) — the same borrow-rather-than-consume idea on the enum
  side.
- [`Field`](../types/field.md) — one entry of a shape.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields.

## Source

- [`to_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/to_fields.rs)
  — `ToFieldsRef` and `ToFields`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
