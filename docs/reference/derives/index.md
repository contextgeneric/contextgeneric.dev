---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Derives

CGP derives let generic code access, describe, build, and deconstruct structs and enums without naming
their concrete types. Each derive adds trait implementations alongside the original type. Choose one
according to the operations your generic code needs.

Start with the [reference overview](/docs/reference/) if you are new to CGP. This page compares the
derives once you are familiar with the essentials.

## Reading a value from a context

Use `HasField` to access individual fields and `HasFields` to process a type's whole structure.

[`#[derive(HasField)]`](./derive_has_field.md) lets an implementation read a struct field by a
type-level tag. The implementation declares the field it needs as a trait bound. An
[`#[implicit]`](../attributes/implicit.md) argument and a
[`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) method generate that bound from the field name.

[`#[derive(HasFields)]`](./derive_has_fields.md) gives the whole shape instead of one field: the struct
or enum described as a single type-level list, plus the conversions that move values in and out of it.
Reach for it when code must process every field at once, such as a serializer or a builder that merges
two records. It is often derived alongside `HasField`.

## Turning a type into extensible data

The extensible-data derives combine a structural representation with incremental operations checked
by the compiler. Structs gain per-field access and a builder; enums gain constructors and an extractor
that handles variants individually.

[`#[derive(CgpData)]`](./derive_cgp_data.md) is the default when you need the full set of operations.
It generates the record implementations for a struct and the variant implementations for an enum.

[`#[derive(CgpRecord)]`](./derive_cgp_record.md) accepts only structs, and
[`#[derive(CgpVariant)]`](./derive_cgp_variant.md) accepts only enums. Each emits the same output as
`CgpData` for its accepted shape. Use the specific name to document that constraint and reject a
different input shape at the derive.

## Deriving one slice on its own

Use an individual derive when generic code needs only construction or extraction.

The individual derives provide these operations:

- [`#[derive(BuildField)]`](./derive_build_field.md): assemble a struct one field at a time.
- [`#[derive(ExtractField)]`](./derive_extract_field.md): handle enum variants one at a time, with
  compile-time exhaustiveness checking.
- [`#[derive(FromVariant)]`](./derive_from_variant.md): construct an enum from a variant selected by name.

`BuildField` accepts structs. `ExtractField` and `FromVariant` accept enums and are commonly derived
together.

## One restriction to know before you reach for them

The enum constructor and extractor derives require every variant to carry exactly one unnamed
payload. This restriction applies to `FromVariant`, `ExtractField`, `CgpData`, and `CgpVariant`;
individual variants cannot opt out. `HasFields` accepts every variant shape and provides a structural
representation with whole-value conversions, so it can be used with enums that mix variant shapes.