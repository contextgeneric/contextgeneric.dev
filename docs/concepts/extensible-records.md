---
sidebar_label: 'Extensible records'
sidebar_position: 13
---

# Extensible records

Extensible records let generic code read and assemble structs through their field names and types.
Independent providers can contribute fields without knowing the final struct, while the compiler
checks that construction is complete. This page explains the field representation, partial-record
builders, and the limits of matching fields by name.

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

This approach works well when the assembly code should know `App` and its complete set of fields.
The database and HTTP helpers can already be independent of `App`; the constructor connects their
outputs to the application's fields.

Generic assembly needs a way to make those connections without naming the target struct's fields
in the assembly code. For example, a subsystem might return a small configuration record whose
fields should populate any compatible application. CGP exposes field names and types through traits
so a reusable builder can perform that merge.

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

Each `Field` pairs a name tag with a value type. `Symbol!("url")` encodes the name as a type, allowing
trait bounds to identify the field during compilation. A product contains every listed field;
this differs from an enum's sum, which contains one variant at a time.

Generic code can use either the complete field representation or a single-field bound. `HasFields`
describes the whole structure, while `HasField<Tag>` provides access to one field. The latter is
also the mechanism used by [implicit arguments](./implicit-arguments.md). These operations use
generated Rust implementations rather than inspecting field names at runtime.

## Built one field at a time, checked all the way

A **partial record** stores field values while its type tracks which fields have been supplied.
The builder can merge several source records before producing the final struct:

```rust
let app: App = App::builder()
    .build_from(database)
    .build_from(http)
    .finalize_build();
```

Here `database` and `http` are smaller records whose fields together supply `App`. `builder()`
creates a partial record with every field absent. Each `build_from` moves the source fields into
matching target fields and returns a builder type that marks those fields present.

Merging requires matching names and value types. Every source field must be accepted by the target
builder; `build_from` does not silently discard fields the target lacks. The source needs the
whole-field and extraction support supplied by `CgpData`, and the target needs the builder support.
The `CanBuildFrom` trait providing the method is imported from `cgp::core::field::impls`.

`finalize_build` is available only when every field in the strict builder is present. If `App`
requires an `http_timeout` field, supplying only the database configuration leaves construction
incomplete:

```rust
// `http_timeout` has never been set, so there is no `finalize_build` to call.
let app: App = App::builder()
    .build_from(DatabaseConfig { database: "postgres://…".to_owned() })
    .finalize_build();
```

That final call fails to compile because the partial record does not implement the required
finalization trait. The error occurs before the program runs, rather than producing a partially
initialized `App` or a runtime failure.

Independent field contributions can arrive in any order. A provider that reads a previously built
field must still run after the provider that supplies it. The field-state types enforce that
requirement as well as final completeness.

## The pattern this is for

The **extensible builder pattern** assembles a target from independent providers, each producing a
small record. A dispatcher runs the providers, merges their outputs, and finalizes the target.
Each provider needs to know its own output fields, but not the final target or the other contributors.

Wiring chooses the target and its contributors. Replacing a subsystem provider changes the assembly
without changing the generic dispatcher, and another application can reuse the same providers for a
different compatible target. The compiler checks that the chosen outputs supply the required fields.

[Dispatching](./dispatching.md) explains how `BuildWithHandlers`, `BuildAndSetField`, and
`BuildAndMerge` sequence field computations. `BuildAndMergeOutputs` provides the related convenience
form for assembling a target from provider outputs.

## What it costs

Generic construction requires the corresponding traits. Deriving `CgpData` supplies them for a
supported struct, but a foreign struct without that support does not become accessible automatically.
A local wrapper or explicit conversion may be needed at that boundary.

Field names and types form the contract between contributors and the target. Two subsystems that
use `timeout: u32` for different purposes still describe the same field. The compiler checks type
compatibility and builder state, not the intended meaning. Renaming a required field breaks the
merge or leaves the target incomplete when checked.

Partial-record types can produce verbose diagnostics. An error may include the target's fields and
presence markers instead of a short message naming the omitted field. Checking smaller assembly
steps can help locate the contribution that is missing or incompatible.

The strict builder requires every field, even if an application considers some optional. An
`Option<T>` field still needs a value such as `None`. CGP also provides optional-field and default
extensions, but those use additional conventions beyond the strict construction shown here.

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

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
