---
sidebar_label: 'HasFields'
---

# `HasFields`

The whole-shape field representation and its conversions.

## What it's for

Some code needs one field of a type. Other code needs the *shape* — every field, its name, and its type — so
it can walk them: a serializer, a validator, a builder that merges two structs, a dispatcher that routes an
enum to a handler per variant. None of those can be written one field at a time, and none can name the
concrete type either.

`HasFields` gives a type that shape as a single associated type. Where [`HasField<Tag>`](./has_field.md) —
singular — answers *"give me this field"*, `HasFields` answers *"describe all of them at once"*:

```rust
type Fields = Product![
    Field<Symbol!("name"), String>,
    Field<Symbol!("age"), u8>,
];
```

That is the struct written as a type: a [`Product!`](../macros/product.md) list with one
[`Field`](../types/field.md) entry per field, each pairing a value type with its type-level name. For an enum
it is a [`Sum!`](../macros/sum.md) instead — the same idea for a choice rather than a combination.

Four companion traits turn that description into a two-way door. `HasFieldsRef` names the borrowed shape, and
`ToFields`, `FromFields`, and `ToFieldsRef` convert between a concrete value and it — so generic code can take
a value apart, work on the anonymous form, and put a concrete value back together.

**The impls come from [`#[derive(HasFields)]`](../derives/derive_has_fields.md).** What you write is the
derive; what you bound against is the trait.

## Using it

All five traits are in the prelude. Two name a shape and three convert to or from it.

### The shape traits

```rust
pub trait HasFields {
    type Fields;
}

pub trait HasFieldsRef {
    type FieldsRef<'a>
    where
        Self: 'a;
}
```

`Fields` is the owned shape and `FieldsRef<'a>` the same shape with each value borrowed for `'a`. Both are
associated types and nothing else — there is no method, because describing a type is not an operation.

The shape's construction follows one rule everywhere: a named field or variant is keyed by
[`Symbol!`](../macros/symbol.md), a positional field by [`Index<N>`](../types/index.md), a struct becomes a
`Cons`/`Nil` product, and an enum becomes an `Either`/`Void` sum. A single-field tuple struct is the one
special case — its `Fields` is the inner type directly rather than a one-element product — and the
[derive's page](../derives/derive_has_fields.md) covers that and the other shapes in full.

### The conversion traits

Each supertraits one of the two shape traits and adds a single method:

```rust
pub trait ToFields: HasFields {
    fn to_fields(self) -> Self::Fields;
}

pub trait FromFields: HasFields {
    fn from_fields(fields: Self::Fields) -> Self;
}

pub trait ToFieldsRef: HasFieldsRef {
    fn to_fields_ref<'a>(&'a self) -> Self::FieldsRef<'a>
    where
        Self: 'a;
}
```

`to_fields` **consumes** the value to produce the owned shape, `from_fields` consumes the shape to rebuild the
value, and `to_fields_ref` **borrows** — which is the one to reach for when the original value must survive.

### Bounding on the shape

The point of all this is a bound. Generic code writes `T: HasFields` and recurses over `T::Fields`, so it
applies to any type that derives the shape — including one declared in another crate:

```rust
fn shape_of<T>(value: T) -> T::Fields
where
    T: HasFields + ToFields,
{
    value.to_fields()
}
```

Which of the five to require depends on what the code does: `HasFields` alone to *name* the shape,
`ToFields`/`FromFields` to move owned values through it, `ToFieldsRef` to read without consuming.

## Examples

Deriving it alongside [`HasField`](./has_field.md) is the common pairing, giving one struct both indexed and
structural access:

```rust
use cgp::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, HasField, HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}
```

`Config::Fields` is now `Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>]`, and a value
round-trips through it:

```rust
let config = Config { host: "localhost".to_owned(), port: 8080 };

let fields = config.clone().to_fields();          // Config -> the product
let config_again = Config::from_fields(fields);   // the product -> Config

assert_eq!(config, config_again);
```

The borrowing form leaves the value intact, which is what read-only generic code uses:

```rust
let fields_ref = config.to_fields_ref();

assert_eq!(fields_ref.0.value, &"localhost".to_owned());
```

An enum's shape is a sum rather than a product, and the same conversions apply:

```rust
#[derive(HasFields)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

giving `Sum![Field<Symbol!("Circle"), Circle>, Field<Symbol!("Rectangle"), Rectangle>]`.

## When to reach for it, and when not

**Bound on `HasFields` when code must process a type's whole shape; bound on
[`HasField`](./has_field.md) when it needs one named field.** That is the entire distinction, and the two are
complementary rather than ranked — most types that need both derive both in one `#[derive(...)]`.

The finer question is which of the five to require, and being precise pays because each one narrows what a
caller must supply.

- **`HasFields` alone** when the code only names the shape — a `where` clause, an associated-type projection,
  a type-level computation like those in [`AppendProduct`](./product_ops.md).
- **`ToFields` and `FromFields`** when values move through the shape in both directions, which is what a
  structural conversion needs.
- **`ToFieldsRef`** when the value must not be consumed. Requiring `ToFields` where a borrow would do forces
  callers to clone.

Two things this is not. It is **not runtime reflection**: a type has a shape only because it opted in with a
derive, there is nothing to query at run time, and the shape is a type rather than data. What that buys is
static checking and no runtime cost; what it costs is that a foreign type you cannot patch has no shape at
all. And it is **not the whole extensible-data story** — the shape describes a type, while building one up
field by field or taking one apart variant by variant is the
[builder](./has_builder.md) and [extractor](./extract_field.md) families, bundled with this one by
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md).

## Under the hood

:::note

### Advanced

This section shows what the shape expands to. You do not need it to bound on the trait, but the `Fields` type
appears verbatim in compiler errors about structural code, so reading one makes those errors legible.
`cargo cgp expand` prints it resugared for your own code.

:::

The trait module defines only the bare trait shapes; all five impls come from
[`#[derive(HasFields)]`](../derives/derive_has_fields.md). The load-bearing part is what the derive puts in
`Fields`, and the sugar hides one level of structure:

```rust
impl HasFields for Person {
    type Fields = Product![
        Field<Symbol!("name"), String>,
        Field<Symbol!("age"), u8>,
    ];
}
```

`Product![A, B]` is `Cons<A, Cons<B, Nil>>`, so the conversions build and match a `Cons` chain:

```rust
impl ToFields for Person {
    fn to_fields(self) -> Self::Fields {
        Cons(self.name.into(), Cons(self.age.into(), Nil))
    }
}

impl FromFields for Person {
    fn from_fields(Cons(name, Cons(age, Nil)): Self::Fields) -> Self {
        Self { name: name.value, age: age.value }
    }
}
```

That `Cons` spine is what an error message prints, and the
[type-level spines](../types/type_level_spines.md) page covers reading it.

The borrowed shape uses a reserved lifetime name, `'__a`, so it cannot collide with a lifetime of yours:

```rust
impl HasFieldsRef for Person {
    type FieldsRef<'__a> = Product![
        Field<Symbol!("name"), &'__a String>,
        Field<Symbol!("age"), &'__a u8>,
    ]
    where
        Self: '__a;
}
```

An enum's shape is the dual — an `Either` chain terminated by `Void` rather than a `Cons` chain terminated by
`Nil` — with each arm tagged by the variant name and carrying that variant's own fields as a nested product.
The two conversions match each concrete variant onto its arm and back.

`from_fields` and `to_fields` round-trip through the identical `Fields` type, so generic code can decompose,
transform, and rebuild with the shapes lining up by construction rather than by check.

## Gotchas

**`HasFields` and [`HasField`](./has_field.md) differ by one letter and do not overlap.** The plural is the
whole shape and takes structs and enums; the singular is per-field access and takes only structs. Their
derives are likewise distinct.

**`to_fields` consumes the value.** Reach for `to_fields_ref` when you need to keep it. This is the commonest
surprise, because the two read alike.

**A newtype's shape is the inner type, not a one-element product.** `struct Wrapper(String)` has
`Fields = String`. Generic code written against a `Cons` chain will not match it — the
[derive's page](../derives/derive_has_fields.md) has the rule and the other shapes.

**Field order is declaration order and is part of the type.** Two structs with the same names in different
orders have different `Fields` types. Name-driven conversions cope; code written against a literal `Cons`
chain does not.

**The borrowed shape layers borrows rather than collapsing them.** A field already of type `&'a Name` appears
as `&'__a &'a Name` in `FieldsRef`, which is correct and occasionally surprising in an error.

**Two enum variant names are reserved.** A variant called `Fields` or `FieldsRef` collides with these
associated types and the derive fails; the [derive's page](../derives/derive_has_fields.md) lists it with the
other reserved names.

## Related constructs

- [`#[derive(HasFields)]`](../derives/derive_has_fields.md) — generates all five impls; what you write.
- [`HasField`](./has_field.md) — the singular counterpart, per-field access.
- [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) — bundles this with the builder and extractor.
- [`Product!`](../macros/product.md) and [`Sum!`](../macros/sum.md) — the list types a shape is built from.
- [`Field`](../types/field.md) — one entry: a value paired with its type-level name.
- [Type-level spines](../types/type_level_spines.md) — the `Cons`/`Nil` and `Either`/`Void` chains underneath.
- [`AppendProduct`](./product_ops.md) — the operations that compute new shapes from old ones.
- [`CanUpcast`](./cast.md) — conversions between two types whose shapes overlap, driven by this one.
- [`HasBuilder`](./has_builder.md) and [`ExtractField`](./extract_field.md) — building and deconstructing,
  rather than describing.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum half, and the expression problem.

## Source

- [`has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_fields.rs)
  — `HasFields`, `HasFieldsRef`
- [`to_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/to_fields.rs)
  — `ToFields`, `ToFieldsRef`
- [`from_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/from_fields.rs)
  — `FromFields`
- Derive codegen: [`cgp_data/derive_has_fields/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
