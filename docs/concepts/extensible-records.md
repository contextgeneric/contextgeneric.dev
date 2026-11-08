---
title: 'Build a struct field by field, checked at compile time'
sidebar_label: 'Extensible records'
sidebar_position: 13
description: 'How a struct field list becomes a type, how a builder records which fields are present, and how independent providers assemble a record the compiler checks.'
---

# Extensible records

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations whose
providers can contribute fields. Extensible records let generic code read and assemble structs
through their field names and types. A provider can contribute fields without knowing the final
struct, so several independent providers can supply parts of a larger value. As those parts are
assembled, the builder's type tracks which fields are present, letting the compiler check that
construction is complete.

## What a closed struct cannot do

A struct literal assembles a known type by supplying its fields in one expression. Individual field
values can come from reusable functions, as in this application constructor:

```rust
let app = App {
    database: connect_to_database()?,
    http_client: build_http_client()?,
    metrics: start_metrics()?,
    // …and every future subsystem edits this same expression
};
```

The helpers can already be independent of `App`: each returns a value that the constructor puts
in the appropriate field. When one place should know the application's complete structure, this
is a straightforward way to assemble it.

To reuse the assembly itself across different targets, however, that connection also needs an
abstraction. A subsystem might return a small configuration record whose fields belong in several
application types. CGP exposes field names and types through traits so a generic builder can merge
that record into any compatible target.

## A struct as a list of named fields

[`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data) gives a struct the traits needed for
generic field access, structural conversion, and incremental construction:

```rust
#[derive(CgpData)]
pub struct DatabaseClient {
    pub url: String,
    pub pool_size: u32,
}
```

The generated `HasFields::Fields` associated type describes the struct as a product of named fields:

```rust
Product![Field<Symbol!("url"), String>, Field<Symbol!("pool_size"), u32>]
```

Within the product, each `Field` associates a name tag with a value type. The tag
`Symbol!("url")` represents the name as a type, giving trait bounds a way to identify it during
compilation. The product describes all of these fields together, whereas an enum's sum describes
one alternative at a time.

This field list is the struct's **shape**. When a bound or implementation needs to name it,
[`Struct! { url: String, pool_size: u32 }`](/docs/reference/macros/struct) expresses the same list
using familiar struct syntax. The macro uses the derive's encoding, so both forms produce exactly
the same type.

An operation does not always need the entire shape. `HasFields` describes the complete structure,
while `HasField<Tag>` lets code require just one field, as [implicit arguments](./implicit-arguments.md)
do. In either case, generated Rust implementations connect the trait request to the field, so the
program need not inspect names at runtime.

## Built one field at a time, checked all the way

A **partial record** stores field values while its type tracks which fields have been supplied.
The builder can merge several source records before producing the final struct:

```rust
let app: App = App::builder()
    .build_from(database)
    .build_from(http)
    .finalize_build();
```

Starting with every field absent, `builder()` creates the partial record that will become `App`.
The `database` and `http` records then contribute their fields through `build_from`. Each call moves
the source values into matching target fields and returns a new builder type that records those
fields as present.

For a merge to work, both the names and value types must match. Every source field needs a place
in the target builder, so `build_from` cannot silently discard an extra source field. `CgpData`
supplies the source's whole-field and extraction support along with the target's builder support;
the `CanBuildFrom` trait that provides the merge method comes from `cgp::core::field::impls`.

`finalize_build` is available only when every field in the strict builder is present. If `App`
requires an `http_timeout` field, supplying only the database configuration leaves construction
incomplete:

```rust
// `http_timeout` has never been set, so there is no `finalize_build` to call.
let app: App = App::builder()
    .build_from(DatabaseConfig { database: "postgres://…".to_owned() })
    .finalize_build();
```

At the final call, the builder still records `http_timeout` as absent. It therefore lacks the
finalization trait, and Rust rejects the call before the program can run. Construction cannot
return a partially initialized `App` through this strict builder.

When contributions are independent, their order does not matter. If one provider reads a field
built by another, however, it must run after that field has been supplied. The same types that
track completeness also enforce this dependency between construction steps.

## The pattern this is for

The **extensible builder pattern** assembles a target from independent providers, each producing a
small record. A dispatcher runs the providers, merges their outputs, and finalizes the target.
Each provider needs to know its own output fields, but not the final target or the other contributors.

Once the providers are independent of the target, wiring can choose which ones contribute to a
particular application. Replacing a subsystem changes that selection while leaving the generic
dispatcher intact. Another application can reuse the providers for a different compatible record,
and the compiler checks whether their combined fields complete it.

[Dispatching](./dispatching.md) explains how `BuildWithHandlers`, `BuildAndSetField`, and
`BuildAndMerge` sequence field computations. `BuildAndMergeOutputs` provides the related convenience
form for assembling a target from provider outputs.

## What it costs

Generic construction requires the corresponding traits. Deriving `CgpData` supplies them for a
supported struct, but a foreign struct without that support does not become accessible automatically.
A local wrapper or explicit conversion may be needed at that boundary.

Matching names and types is enough for the compiler, so contributors must also agree on what
the fields mean. Two subsystems using `timeout: u32` for unrelated purposes still describe the
same field. Conversely, renaming a required field changes that contract and can break a merge
or leave the target incomplete when checked.

The types that enforce completeness can also make failures harder to read. An error may show the
target's fields and presence markers rather than simply naming an omitted field. When that happens,
checking smaller assembly steps can help locate the missing or incompatible contribution.

Completeness applies to `Option<T>` fields too. The strict builder needs a value for each one,
even if that value is `None` and the application treats the setting as optional. CGP's optional-field
and default extensions can express other conventions, but they add rules beyond the strict
construction illustrated here.

A struct literal or ordinary constructor is simpler when the assembly code can name the target.
Extensible construction is useful when a framework builds user-defined records or when several
applications reuse independently written subsystem providers.

## Where to go next

These pages explain related data operations and the traits used here:

- [Extensible variants](./extensible-variants.md): Tracking excluded enum variants to prove exhaustiveness.
- [Dispatching](./dispatching.md): Running the providers that construct and merge fields.
- [`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data): The combined extensible-data derive.
- [`#[derive(HasField)]`](/docs/reference/derives/derive_has_field) and
  [`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields): Individual field access and the
  whole-structure representation.
- [`#[derive(BuildField)]`](/docs/reference/derives/derive_build_field) and
  [`HasBuilder`](/docs/reference/traits/builder/has_builder): Partial records and construction.
- [`HasFields`](/docs/reference/traits/shape/has_fields), [`Field`](/docs/reference/types/field), and
  [`Symbol!`](/docs/reference/macros/symbol): The types that describe a record's fields.
- [Comparison: Row polymorphism](/docs/comparisons/row-polymorphism): records as rows: containment, concatenation, and presence flags.
- [Comparison: Reflection](/docs/comparisons/reflection): a derived shape beside runtime and compile-time reflection descriptors.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
