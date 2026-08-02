---
sidebar_label: 'ToFields'
---

# `ToFields`

Taking a value apart into its shape.

## What it's for

[`HasFields`](./has_fields.md) *names* a type's shape. `ToFields` produces a value in it:

```rust
pub trait ToFields: HasFields {
    fn to_fields(self) -> Self::Fields;
}
```

That is what lets generic code stop working on a concrete struct and start working on an anonymous list
of named entries — the form a serializer walks, a converter rewrites, or a merge consumes. The
concrete type goes in; a [`Product!`](../macros/product.md) of [`Field`](../types/field.md) entries comes
out, or a [`Sum!`](../macros/sum.md) for an enum.

**It consumes the value.** That is the difference from [`ToFieldsRef`](./to_fields_ref.md), and it is the
commonest surprise in the family, because the two read alike at a call site.

The reverse direction is [`FromFields`](./from_fields.md), and the two round-trip through the identical
`Fields` type, so generic code can decompose, transform, and rebuild with the shapes lining up by
construction rather than by check.

## Using it

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

It supertraits [`HasFields`](./has_fields.md), so bounding on `ToFields` gives you `Fields` as well and
there is no need to name both — though writing `T: HasFields + ToFields` is common and harmless.

The impls come from [`#[derive(HasFields)]`](../derives/derive_has_fields.md), which emits all five shape
traits together.

## Examples

A value round-tripping through its shape:

```rust
use cgp::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

let config = Config { host: "localhost".to_owned(), port: 8080 };

let fields = config.clone().to_fields();          // Config -> the product
let config_again = Config::from_fields(fields);   // the product -> Config

assert_eq!(config, config_again);
```

The `clone()` is there only because the assertion compares against the original — `to_fields` would
otherwise have consumed it, which is exactly the point.

In generic code the bound is what matters:

```rust
fn shape_of<T>(value: T) -> T::Fields
where
    T: ToFields,
{
    value.to_fields()
}
```

## When to reach for it, and when not

**Bound on it when generic code must consume a value and work on its entries**, and reach for the
borrowing form otherwise.

- **[`ToFieldsRef`](./to_fields_ref.md)** when the value must survive. Requiring `ToFields` where a
  borrow would do forces every caller to clone, which is the most common over-requirement in this
  family.
- **[`FromFields`](./from_fields.md)** for the other direction, and both together for a round trip.
- **[`HasFields`](./has_fields.md) alone** when the code only *names* the shape and never holds a value
  in it.
- **[`HasField`](./has_field.md)** when one named field is all that is wanted. Decomposing a whole struct
  to read one entry is the wrong tool.
- **The [builder family](./has_builder.md)** when a value is assembled incrementally rather than
  converted wholesale. `to_fields` is a single flat conversion; a builder tracks presence field by field.

## Under the hood

:::note

### Advanced

This section shows the generated impl, which is where the `Cons` spine becomes visible.

:::

`Product![A, B]` is `Cons<A, Cons<B, Nil>>`, so the conversion builds a `Cons` chain, one node per field,
wrapping each value into its [`Field`](../types/field.md) entry:

```rust
impl ToFields for Person {
    fn to_fields(self) -> Self::Fields {
        Cons(self.name.into(), Cons(self.age.into(), Nil))
    }
}
```

That chain is what an error message prints when structural code fails to resolve, and the
[type-level spines](../types/type_level_spines.md) page covers reading it.

An enum's conversion is the dual: each concrete variant is matched onto its arm of an `Either` chain
terminated by `Void`, tagged with the variant name.

Because the shape is built positionally from declaration order, `to_fields` and
[`from_fields`](./from_fields.md) are exact inverses by construction — there is no lookup, no matching by
name at run time, and nothing that can fail.

## Gotchas

**It consumes the value.** Reach for [`ToFieldsRef`](./to_fields_ref.md) when you need to keep it. This
is the commonest surprise in the family, because the two read alike.

**Field order is declaration order and is part of the type.** Two structs with the same field names in
different orders produce unrelated `Fields` types, so a chain built from one will not satisfy the other.

**A newtype's shape is the inner type**, not a one-element product, so `to_fields` on
`struct Wrapper(String)` yields a `String`. Generic code written against a `Cons` chain will not match
it.

**It supertraits [`HasFields`](./has_fields.md)**, so a bound naming both is redundant rather than
wrong.

**There is no fallible form.** The conversion cannot fail; anything that could is a different operation,
such as a [cast](./can_downcast.md).

## Related constructs

- [`HasFields`](./has_fields.md) — the supertrait that names the shape.
- [`FromFields`](./from_fields.md) — the reverse conversion.
- [`ToFieldsRef`](./to_fields_ref.md) — the borrowing form, which leaves the value intact.
- [`HasFieldsRef`](./has_fields_ref.md) — the shape that form produces.
- [`#[derive(HasFields)]`](../derives/derive_has_fields.md) — generates this impl.
- [`Product!`](../macros/product.md), [`Sum!`](../macros/sum.md), and [`Field`](../types/field.md) — what
  a shape is made of.
- [Type-level spines](../types/type_level_spines.md) — the `Cons` chain the conversion builds.
- [`HasBuilder`](./has_builder.md) — incremental assembly, as against this wholesale conversion.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields.

## Source

- [`to_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/to_fields.rs)
  — `ToFields` and `ToFieldsRef`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
