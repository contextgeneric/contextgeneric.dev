---
sidebar_label: '#[derive(HasFields)]'
sidebar_position: 2
---

# `#[derive(HasFields)]`

The whole-struct or whole-enum field-list view.

## Overview

Some code needs one field of a type. Other code needs the *shape* of the type: every field, its name,
and its type, so it can walk them. A serializer, a validator, a builder that merges two structs, a
dispatcher that routes an enum to a handler per variant: none of these can be written against one field
at a time, and none of them can name the concrete type either.

`#[derive(HasFields)]` gives a type that shape as a single associated type. Where
[`#[derive(HasField)]`](./derive_has_field.md) (singular) answers "give me *this* field", `HasFields`
answers "describe *all* of them at once":

```rust
type Fields = Product![
    Field<Symbol!("name"), String>,
    Field<Symbol!("age"), u8>,
];
```

That is the whole struct written as a type: a [`Product!`](../macros/product.md) list with one
[`Field`](../types/field.md) entry per field, each pairing a value type with its type-level name. For an
enum it is a [`Sum!`](../macros/sum.md) instead: the same idea for a choice rather than a combination.
Code that is generic over the shape recurses over that list, and everything is resolved during
compilation.

The derive also generates the conversions that move values in and out of the representation, so it works
in both directions rather than only describing the type. Generic code can take a concrete value apart
into its anonymous shape, work on it, and put a concrete value back together.

## Usage

The macro is a plain derive that takes no arguments and has no helper attributes. Unlike the singular
derive it accepts **both structs and enums**:

```rust
#[derive(HasFields)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

#[derive(HasFields)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

For a struct the shape is a product; for an enum it is a sum. Naming follows the same rules everywhere: a
named field or a variant is keyed by [`Symbol!`](../macros/symbol.md), a positional field by
[`Index<N>`](../types/index.md). Applying the derive to anything other than a struct or an enum is a
compile error.

### Struct shapes

Every struct shape is accepted, and three of the four behave as you would predict from the field tags
alone.

- A **named-field struct** becomes a product of `Field<Symbol!("name"), T>` entries, in declaration order.
- A **multi-field tuple struct** becomes a product keyed by `Index<N>`.
- A **unit struct** becomes the empty product `Nil`. A fieldless struct is a valid, if trivial, record and
  round-trips through the representation like any other.
- A **single-field tuple struct** (a newtype) is the special case: its `Fields` is the inner type
  *directly*, not wrapped in a one-element product.

The newtype case is worth noting, because it is the one that will surprise you. `struct Wrapper(String)`
has `Fields = String`, not `Product![Field<Index<0>, String>]`. The reason is that a newtype is a
transparent wrapper and generic code almost always wants the inner type rather than the wrapper. The
practical effect is that adding a second field to a newtype changes its `Fields` type in shape rather
than in length.

### Variant shapes

**This is the one derive in the extensible-data family that places no restriction on an enum's variant
shapes.** The derives that take an enum *apart* (that is,
[`#[derive(ExtractField)]`](./derive_extract_field.md) and
[`#[derive(FromVariant)]`](./derive_from_variant.md), and therefore
[`#[derive(CgpData)]`](./derive_cgp_data.md)) require every variant to carry exactly one unnamed payload,
because each has to name a single type per variant. `HasFields` only *describes* a variant, so it accepts
all four shapes and nests that variant's own fields as a product inside its `Field` entry:

```rust
#[derive(HasFields)]
pub enum Shape {
    Empty,
    Circle(u32),
    Rectangle(u32, u32),
    Triangle { base: u32, height: u32 },
}
```

Each variant maps by the same rules a struct's fields do:

| Variant | Its entry in the sum |
|---|---|
| `Empty` | `Field<Symbol!("Empty"), Nil>` |
| `Circle(u32)` | `Field<Symbol!("Circle"), u32>` |
| `Rectangle(u32, u32)` | `Field<Symbol!("Rectangle"), Product![Field<Index<0>, u32>, Field<Index<1>, u32>]>` |
| `Triangle { base, height }` | `Field<Symbol!("Triangle"), Product![Field<Symbol!("base"), u32>, Field<Symbol!("height"), u32>]>` |

So a unit variant becomes the empty product, a newtype variant passes its payload straight through (the
same special case as a newtype struct, applied inside a variant), a multi-field tuple variant becomes an
`Index`-keyed product, and a named-field variant becomes a `Symbol!`-keyed one. All four round-trip
through the conversions.

The consequence is worth stating plainly, because it decides which derive to reach for: **an enum with
mixed variant shapes can have a structural representation but no generic constructor or extractor.**
`#[derive(HasFields)]` succeeds on the enum above; `#[derive(CgpData)]` on the same enum would not.

### Generic types

Generic parameters, lifetimes, and a `where` clause are carried onto every generated impl. A borrowed
field type appears verbatim in the shape, and the borrowed view of the shape layers its own borrow on top
of it: a field of type `&'a Name` appears as `&'__a &'a Name` in the borrowed form, where `'__a` is the
reserved lifetime the derive introduces.

## Examples

Deriving `HasFields` alongside [`HasField`](./derive_has_field.md) is the common pairing, giving one
struct both indexed and structural access:

```rust
use cgp::prelude::*;

#[derive(HasField, HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}
```

`Config::Fields` is now `Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>]`, and a
value round-trips through it:

```rust
let config = Config { host: "localhost".to_owned(), port: 8080 };

let fields = config.to_fields();                 // Config -> the product
let config_again = Config::from_fields(fields);  // the product -> Config
```

There is a borrowing form too, used by read-only generic code so it does not have to consume the value:

```rust
let config = Config { host: "localhost".to_owned(), port: 8080 };

let fields_ref = config.to_fields_ref();  // borrows each field in place
```

An enum's shape is a sum rather than a product, and the same conversions apply:

```rust
#[derive(HasFields)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

`Shape::Fields` is `Sum![Field<Symbol!("Circle"), Circle>, Field<Symbol!("Rectangle"), Rectangle>]`.

Generic code binds on the shape rather than on the type. An implementation that works for any record
writes `where Self: HasFields` and recurses over `Self::Fields`, so it applies to `Config`, to `Person`,
and to a struct declared in another crate that happens to derive the same thing.

## When to reach for it, and when not

**Reach for `#[derive(HasFields)]` when code must process a type's whole shape, and for
[`#[derive(HasField)]`](./derive_has_field.md) when it needs one named field.** That is the whole
distinction, and the two are complementary rather than ranked. Most types that need both derive both, in
one `#[derive(...)]`.

The finer question is whether to derive it on its own or take the umbrella.

- **Derive `HasFields` alone** when the type only needs to be *described* and converted: a payload being
  serialized, a struct being read structurally, an enum whose shape a dispatcher inspects. This is also
  the only choice available for an enum with mixed variant shapes.
- **Reach for [`#[derive(CgpData)]`](./derive_cgp_data.md)** when the type also needs to be built up field
  by field or taken apart variant by variant, since that derive includes this one plus the incremental
  machinery. If you find yourself deriving `HasFields`, `HasField`, and
  [`BuildField`](./derive_build_field.md) together, the umbrella is the shorter way to say it.
- **Do not derive it speculatively.** It generates five impls and a representation type per use, and a
  type nothing processes structurally gains nothing from having a shape. Deriving `HasField` for value
  reads is the far more common need.

One misreading to head off: **this is not runtime reflection.** A type has a shape only because it opted
in with a derive, there is nothing to query at runtime, and the shape is a type rather than data. The
gain is that generic code over it is checked when written and costs nothing when run. The cost is that a
type you do not own and cannot patch has no shape at all.

## Under the hood

The derive leaves the type definition untouched and emits **five impls**: the shape, the borrowed shape,
and three conversions. From a named-field struct:

```rust
#[derive(HasFields)]
pub struct Person {
    pub name: String,
    pub age: u8,
}
```

the two shape impls name the product and the same product with each value borrowed:

```rust
impl HasFields for Person {
    type Fields = Product![
        Field<Symbol!("name"), String>,
        Field<Symbol!("age"), u8>,
    ];
}

impl HasFieldsRef for Person {
    type FieldsRef<'__a> = Product![
        Field<Symbol!("name"), &'__a String>,
        Field<Symbol!("age"), &'__a u8>,
    ]
    where
        Self: '__a;
}
```

and the three conversions move values between `Person` and that product:

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

impl ToFieldsRef for Person {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.name).into(), Cons((&self.age).into(), Nil))
    }
}
```

`Product![A, B]` is sugar for `Cons<A, Cons<B, Nil>>`, which is why the bodies build a `Cons` chain. The
[type-level spines](../types/type_level_spines.md) page covers that shape, and it is the form printed in
an error.

An enum's shape is the dual: an [`Either`](../types/type_level_spines.md) chain terminated by `Void`
rather than a `Cons` chain terminated by `Nil`, with each arm tagged by the variant name:

```rust
impl HasFields for Shape {
    type Fields = Either<
        Field<Symbol!("Circle"), Circle>,
        Either<Field<Symbol!("Rectangle"), Rectangle>, Void>,
    >;
}
```

The conversions match each concrete variant onto its arm and back. The four variant shapes described
above all fall out of one rule rather than being special-cased: each variant's own fields go through the
same product construction a struct's fields do, which is why a unit variant lands on `Nil`, a newtype
variant on its payload type, and the two multi-field shapes on `Index`- and `Symbol!`-keyed products.

The empty shapes come from the same construction. A unit struct's `Fields` is `Nil`:

```rust
impl HasFields for Unit {
    type Fields = Nil;
}
```

and a variantless enum's is `Void`, with its borrowed conversions matching the uninhabited value through
an empty `match`.

Each generated impl is aimed at the type name the user wrote, so a conflict with a hand-written
`HasFields` impl underlines the struct or enum rather than the whole `#[derive(HasFields)]`.

## Common Mistakes

**A newtype's `Fields` is the inner type, not a one-element product.** `struct Wrapper(String)` has
`Fields = String`. Generic code written against `Cons<Field<Index<0>, _>, Nil>` will not match it, and
adding a second field changes the shape rather than lengthening it.

**A named-field variant is not a newtype variant.** `Circle { radius: f64 }` carries a one-entry product,
while `Circle(Circle)` carries its payload type directly. The two read almost alike and produce different
shapes.

**Accepting every variant shape is specific to this derive.** An enum with a unit, multi-field, or
struct-style variant takes `#[derive(HasFields)]` and rejects `#[derive(CgpData)]`,
`#[derive(CgpVariant)]`, `#[derive(ExtractField)]`, and `#[derive(FromVariant)]`. If you want the whole
family, wrap each richer payload in its own struct so every variant has a single nameable type.

**The borrowed shape layers borrows rather than collapsing them.** A field already of type `&'a Name`
appears as `&'__a &'a Name` in the borrowed form, which is correct and occasionally surprising in an error
message. `'__a` is the reserved name the derive introduces so it cannot collide with a lifetime of yours.

**Field order is declaration order, and it is part of the type.** Two structs with the same field names in
different orders have different `Fields` types. Structural conversions that match on names cope with that;
code written against a literal `Cons` chain does not.

**Two variant names are reserved, and using one does not compile.** The generated impls write
`Self::Fields` and `Self::FieldsRef`, so an enum with a variant called `Fields` or `FieldsRef` makes the
path ambiguous. The compiler reports `ambiguous associated item` with its headline on the derive, and a
note pointing at the offending variant, so following the note tells you which one to rename. The
[extractor](./derive_extract_field.md) and [constructor](./derive_from_variant.md) derives reserve five
more names, so an enum taking the whole family should avoid all seven.

**It is a different derive from the singular one.** `#[derive(HasField)]` gives per-field access and takes
only structs; `#[derive(HasFields)]` gives the whole shape and takes structs and enums. The names differ
by one letter and the outputs do not overlap.

## Related constructs

- [`#[derive(HasField)]`](./derive_has_field.md) — the singular counterpart, per-field access, commonly
  derived alongside this one.
- [`HasFields`](../traits/has_fields.md) — the traits this generates and the conversions between a value
  and its shape.
- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella derive, which includes this one plus the
  incremental machinery.
- [`Product!`](../macros/product.md) and [`Sum!`](../macros/sum.md) — the list types a struct and an enum
  shape are built from.
- [`Field`](../types/field.md) — one entry: a value paired with its type-level name.
- [`Symbol!`](../macros/symbol.md) and [`Index`](../types/index.md) — the tags that name an entry.
- [Type-level spines](../types/type_level_spines.md) — the `Cons`/`Nil` and `Either`/`Void` chains the
  sugar expands to.
- [`AppendProduct`](../traits/append_product.md) — operations over a shape once you have one.
- [`CanUpcast`](../traits/can_upcast.md) — converting between two types whose shapes overlap.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields, and
  what generic code does with one.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum half, and the expression problem it
  addresses.

## Source

- Entry point: [`derive_has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_has_fields.rs)
- Codegen: [`cgp_data/derive_has_fields/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_fields)
- Traits: [`has_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_fields.rs),
  [`to_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/to_fields.rs),
  and [`from_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/from_fields.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
