---
sidebar_label: '#[derive(CgpRecord)]'
sidebar_position: 4
---

# `#[derive(CgpRecord)]`

`#[derive(CgpRecord)]` generates field access, a structural representation, and a builder for a struct.

## Overview

Generic code needs traits to read and assemble fields across different structs. A type parameter alone
does not let it refer to a field such as `first_name`.

`#[derive(CgpRecord)]` supplies those traits, making the struct an **extensible record**. It generates
per-field access, a representation of the whole struct, and a builder that fills fields individually.
Generic code can use these operations without naming the concrete struct.

`CgpRecord` accepts only structs. It produces the same output as
[`CgpData`](./derive_cgp_data.md) on a struct, but rejects an enum at parse time.

## Usage

Apply `CgpRecord` to a struct without arguments or helper attributes:

```rust
use cgp::prelude::*;

#[derive(CgpRecord)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

Every struct shape is accepted. Named fields use [`Symbol!`](../macros/symbol.md) tags, and tuple
fields use [`Index<N>`](../types/index_type.md) tags. A fieldless struct produces a companion without
field-state parameters, so its `builder()` can be finalized immediately.

Generic parameters, lifetimes, and a `where` clause are carried onto everything generated, including
the companion type.

### What it generates

The derive combines these outputs, each also available separately:

| Group | The slice on its own |
|---|---|
| Per-field access — a `HasField` and a `HasFieldMut` impl per field | [`#[derive(HasField)]`](./derive_has_field.md) |
| The representation — the struct as a product of named entries, with conversions | [`#[derive(HasFields)]`](./derive_has_fields.md) |
| The builder — the partial companion type and the impls that fill it | [`#[derive(BuildField)]`](./derive_build_field.md) |

## Examples

A record builder can combine fields from another record with fields supplied individually. In this
example, `build_from` moves the shared fields from `Person` into an `Employee` builder:

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

`build_from` matches the fields by their type-level names, so neither struct needs to name the other.
It moves `first_name` and `last_name` and leaves `employee_id` for the caller. Removing `build_field`
makes the example fail to compile because
[`finalize_build`](../traits/builder/finalize_build.md) is available only after every field is present.
See [`build_from`](../traits/casting/can_build_from.md) for the conversion requirements.

## When to use it

Use `CgpRecord` when generic code needs the struct's field access, representation, and builder, and
the derive name should specify that the input is a struct. Choose among the related derives according
to the operations and input restriction you need:

- **`CgpRecord`**: generate the full record output and reject non-struct inputs.
- **[`CgpData`](./derive_cgp_data.md)**: generate the same output while also accepting enums.
- **[`HasField`](./derive_has_field.md)**: generate per-field access when fields only need to be read.
- **[`HasFields`](./derive_has_fields.md)**: generate the representation and conversions.
- **[`BuildField`](./derive_build_field.md)**: generate the builder alone.

The [CgpData page](./derive_cgp_data.md#when-to-use-it) explains when generic structural operations
justify the additional generated code.

## Under the hood

The derive generates field access, representation traits, and a builder for the input struct:

```rust
#[derive(CgpRecord)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

The field-access output contains a [`HasField`](../traits/field-access/has_field.md) and a
[`HasFieldMut`](../traits/field-access/has_field_mut.md) implementation per field. It matches the output
of [`#[derive(HasField)]`](./derive_has_field.md).

The representation output describes the struct as a product of named entries and supplies conversions
in both directions. It matches [`#[derive(HasFields)]`](./derive_has_fields.md):

```rust
impl HasFields for Person {
    type Fields = Product![
        Field<Symbol!("first_name"), String>,
        Field<Symbol!("last_name"), String>,
    ];
}

// plus HasFieldsRef, FromFields, ToFields, ToFieldsRef
```

The builder output adds a companion type, as documented for
[`#[derive(BuildField)]`](./derive_build_field.md). Each field's type is wrapped in a
[`MapType`](../traits/type-level/map_type.md) marker: `IsPresent` stores its value, and `IsNothing`
stores `()`:

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

The builder's type prevents incomplete construction. `builder()` starts with every field absent,
[`build_field`](../traits/builder/build_field.md) changes one marker to `IsPresent`, and
[`finalize_build`](../traits/builder/finalize_build.md) applies only when every field is present.
Per-field [`UpdateField`](../traits/builder/update_field.md) implementations change the markers, and
`HasField` implementations on the companion allow reads of fields that have been set.

The companion is named `__Partial{Name}` and keeps the original type's visibility, so a `pub` struct
yields a `pub` companion. Each generated impl is aimed at the token it came from (a per-field impl at
its field, a whole-type impl at the type name), so a conflict with a hand-written impl underlines that
token rather than the whole derive.

## Common Mistakes

**The companion type does not inherit your attributes.** A `#[derive(Debug, Clone)]` on the record
does not apply to `__Partial{Name}`, so it does not make partial values printable or cloneable. Read a
set field through the companion's [`HasField`](../traits/field-access/has_field.md) implementation.

**A tuple struct's builder is keyed by position.** Its companion exposes `UpdateField<Index<0>, _>`
rather than symbol-keyed impls, so `build_field` takes `PhantomData::<Index<0>>`.

**A single-field tuple struct's representation is the inner type directly**, not a one-element product.
This is the newtype special case described on the [`#[derive(HasFields)]`](./derive_has_fields.md) page,
which this derive inherits.

**A fieldless struct has an immediately finalizable builder.** Its companion does not need field-state
parameters.

**`CgpRecord` rejects enums.** Use [`CgpVariant`](./derive_cgp_variant.md) or
[`CgpData`](./derive_cgp_data.md) for an enum.

**The full record derive adds compilation work.** It generates a companion type, whole-type
implementations, and per-field implementations. Use an individual derive when only part of the output
is needed.

## Related constructs

These references cover the related derives, generated traits, and supporting types:

- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella, which dispatches here for a struct.
- [`#[derive(CgpVariant)]`](./derive_cgp_variant.md) — the enum face.
- [`#[derive(HasField)]`](./derive_has_field.md), [`#[derive(HasFields)]`](./derive_has_fields.md), and
  [`#[derive(BuildField)]`](./derive_build_field.md) — the three slices this emits.
- [`HasBuilder`](../traits/builder/has_builder.md) — the builder family it generates impls for.
- [`MapType`](../traits/type-level/map_type.md) — the `IsPresent`/`IsNothing` markers the companion is
  parameterized by.
- [`CanBuildFrom`](../traits/casting/can_build_from.md) — merging one record into another's builder.

The ideas behind it are explained on these concept pages:

- [Extensible records](/docs/concepts/extensible-records) — a struct as a product of named fields, and
  the extensible builder pattern.

## Source

The implementation is defined in these source files:

- Entry point: [`cgp_record.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_record.rs)
- Record codegen: [`cgp_data/record.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/record.rs)
- Runtime traits: [`cgp-field/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-field/src/traits)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
