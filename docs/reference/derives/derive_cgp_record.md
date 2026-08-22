---
sidebar_label: '#[derive(CgpRecord)]'
---

# `#[derive(CgpRecord)]`

The extensible-data derive for a struct.

## Overview

A plain Rust struct is opaque to generic code. There is no way to refer to "the `first_name` field"
through a type parameter, so anything that must work across several structs ends up written once per
struct.

`#[derive(CgpRecord)]` turns a struct into **extensible data**: a type whose fields generic code can
name, read, and assemble without ever mentioning the concrete type. It produces the whole record half
of the family in one line — per-field access, the whole-shape field list, and an incremental builder
that fills a value one field at a time.

It is the struct-only face of [`#[derive(CgpData)]`](./derive_cgp_data.md). The two run the same code
and emit the same output on a struct; the difference is that this one **rejects an enum at parse
time**, so a type that is meant to stay a struct says so and the error arrives at the derive rather
than further along.

## Usage

The derive takes no arguments and has no helper attributes:

```rust
use cgp::prelude::*;

#[derive(CgpRecord)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

**Every struct shape is accepted.** A named-field struct is keyed by
[`Symbol!`](../macros/symbol.md), a tuple struct by [`Index<N>`](../types/index.md), and a fieldless
struct is the degenerate case rather than an error — its companion type takes no parameters at all, so
`builder()` is immediately finalizable because there is nothing to track.

Generic parameters, lifetimes, and a `where` clause are carried onto everything generated, including
the companion type.

### What it generates

Three groups, each of which is also available as a derive of its own when only part of the output is
wanted:

| Group | The slice on its own |
|---|---|
| Per-field access — a `HasField` and a `HasFieldMut` impl per field | [`#[derive(HasField)]`](./derive_has_field.md) |
| The representation — the struct as a product of named entries, with conversions | [`#[derive(HasFields)]`](./derive_has_fields.md) |
| The builder — the partial companion type and the impls that fill it | [`#[derive(BuildField)]`](./derive_build_field.md) |

## Examples

The record machinery is most useful for assembling one struct out of pieces. Because both structs
derive it, a builder can copy every shared field from another record in one step and fill in the rest:

```rust
use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

#[derive(CgpRecord)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

#[derive(CgpRecord)]
pub struct Employee {
    pub employee_id: u64,
    pub first_name: String,
    pub last_name: String,
}

fn promote(person: Person, id: u64) -> Employee {
    Employee::builder()                                       // every field absent
        .build_from(person)                                   // first_name + last_name now present
        .build_field(PhantomData::<Symbol!("employee_id")>, id)
        .finalize_build()                                     // all present: this is where it closes
}
```

Neither struct knows about the other. They share two field *names*, matched at the type level, so
[`build_from`](../traits/can_build_from.md) moves those fields across and leaves `employee_id` for the
caller. Remove the `build_field` line and this stops compiling, because
[`finalize_build`](../traits/finalize_build.md) only exists once every field is present.

## When to reach for it, and when not

**Reach for it when generic code has to work over the struct's own structure**, and prefer
[`#[derive(CgpData)]`](./derive_cgp_data.md) unless naming the shape earns its keep as documentation.
The two are interchangeable on a struct, so this is a choice about what the code says rather than
about what it emits.

- **Use `#[derive(CgpRecord)]`** to state that a type is always a struct, and to have a later change
  to an enum fail at the derive rather than inside the generated code.
- **Use [`#[derive(CgpData)]`](./derive_cgp_data.md)** as the default, especially in a module where
  structs and enums both take the derive.
- **Use [`#[derive(HasField)]`](./derive_has_field.md) alone** when the fields are only ever read. That
  is most types in a CGP program, and it is one impl pair per field rather than a companion type and a
  dozen impls.
- **Derive the slice you want** — [`HasFields`](./derive_has_fields.md) for the representation,
  [`BuildField`](./derive_build_field.md) for the builder — when only part of the output is wanted. The
  umbrella is the right call once you want most of them.

The full argument for when a type earns the extensible-data machinery at all is on the
[umbrella page](./derive_cgp_data.md#when-to-reach-for-it-and-when-not).

## Under the hood

:::note

### Advanced

This section shows what the derive generates. You do not need it to use the derive, but the companion
type appears by name in every builder error, so recognizing one turns an intimidating error into a
legible one. `cargo cgp expand` prints the same thing for your own code, with the tags resugared.

:::

The derive emits three groups in order. From:

```rust
#[derive(CgpRecord)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

it first emits the **per-field access** — a [`HasField`](../traits/has_field.md) and a
[`HasFieldMut`](../traits/has_field_mut.md) impl per field, exactly what
[`#[derive(HasField)]`](./derive_has_field.md) produces on its own.

Then the **representation**, exposing the struct as a product of named entries with conversions in both
directions, which is [`#[derive(HasFields)]`](./derive_has_fields.md)'s output:

```rust
impl HasFields for Person {
    type Fields = Product![
        Field<Symbol!("first_name"), String>,
        Field<Symbol!("last_name"), String>,
    ];
}

// plus HasFieldsRef, FromFields, ToFields, ToFieldsRef
```

Then the **builder**, which is [`#[derive(BuildField)]`](./derive_build_field.md)'s output and where the
companion type appears. Each field's type is wrapped in a [`MapType`](../traits/map_type.md) marker, so
a field can be present (`IsPresent`, holding the value) or absent (`IsNothing`, holding `()`):

```rust
pub struct __PartialPerson<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub last_name: <__F1__ as MapType>::Map<String>,
}

impl HasBuilder for Person {
    type Builder = __PartialPerson<IsNothing, IsNothing>;   // the empty builder
    // ...
}

impl FinalizeBuild for __PartialPerson<IsPresent, IsPresent> {
    // ...                                                     only at all-present
}
```

That pair of impls is the whole safety argument: `builder()` starts at all-absent, each
[`build_field`](../traits/build_field.md) flips one marker, and
[`finalize_build`](../traits/finalize_build.md) exists only at all-present, so finalizing early is a
missing impl rather than a runtime check. The per-field [`UpdateField`](../traits/update_field.md) impls
that move a marker, and the `HasField` impls on the companion that let a set field be read back, follow.

The companion is named `__Partial{Name}` and keeps the original type's visibility, so a `pub` struct
yields a `pub` companion. Each generated impl is aimed at the token it came from — a per-field impl at
its field, a whole-type impl at the type name — so a conflict with a hand-written impl underlines that
token rather than the whole derive.

## Gotchas

**The companion type carries none of your attributes.** The derive clears them, so a
`#[derive(Debug, Clone)]` on the record does not reach `__Partial{Name}` and a partially-built value can
be neither printed nor cloned. Read a set field back through the companion's
[`HasField`](../traits/has_field.md) impl instead.

**A tuple struct's builder is keyed by position.** Its companion exposes `UpdateField<Index<0>, _>`
rather than symbol-keyed impls, so `build_field` takes `PhantomData::<Index<0>>`.

**A single-field tuple struct's representation is the inner type directly**, not a one-element product —
the newtype special case described on the [`#[derive(HasFields)]`](./derive_has_fields.md) page, which
this derive inherits.

**A fieldless struct compiles and does nothing useful.** It yields a parameterless companion whose
`builder()` is immediately finalizable.

**An enum is rejected**, which is the point of the name. Reach for
[`#[derive(CgpVariant)]`](./derive_cgp_variant.md) or the umbrella.

**This is one of the heaviest derives in CGP.** One of these on a five-field struct generates a
companion type and roughly twenty impls. If only part of the output is wanted, derive the slice.

## Related constructs

- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella, which dispatches here for a struct.
- [`#[derive(CgpVariant)]`](./derive_cgp_variant.md) — the enum face.
- [`#[derive(HasField)]`](./derive_has_field.md), [`#[derive(HasFields)]`](./derive_has_fields.md), and
  [`#[derive(BuildField)]`](./derive_build_field.md) — the three slices this emits.
- [`HasBuilder`](../traits/has_builder.md) — the builder family it generates impls for.
- [`MapType`](../traits/map_type.md) — the `IsPresent`/`IsNothing` markers the companion is
  parameterized by.
- [`CanBuildFrom`](../traits/can_build_from.md) — merging one record into another's builder.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields, and
  the extensible builder pattern.

## Source

- Entry point: [`cgp_record.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_record.rs)
- Record codegen: [`cgp_data/record.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/record.rs)
- Runtime traits: [`cgp-field/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-field/src/traits)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
