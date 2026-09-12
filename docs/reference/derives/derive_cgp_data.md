---
sidebar_label: '#[derive(CgpData)]'
sidebar_position: 3
---

# `#[derive(CgpData)]`

`#[derive(CgpData)]` generates structural access and incremental operations for a struct or enum.

## Overview

Generic code needs traits to access a struct's fields or an enum's variants. A type parameter alone
does not let it name a `first_name` field or a `Circle` variant.

`#[derive(CgpData)]` supplies those traits by turning the type into **extensible data**. On a struct,
it generates field access, a structural representation, and a builder. On an enum, it generates a
structural representation, constructors, and an extractor. Generic code can then work with the type's
fields or variants without naming the concrete type.

The generated representation and companion types support different operations. The representation
lists the type's named entries so generic code can process its structure. A companion type tracks
which record fields are present or which enum variants remain possible. A partial value and a
finished one therefore have different types, so the compiler rejects an incomplete build or extraction.

## Usage

Apply `CgpData` to a struct or enum without arguments or helper attributes:

```rust
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

Naming follows the family's rules: a named struct field or a variant is keyed by
[`Symbol!`](../macros/symbol.md), and a positional field of a tuple struct by
[`Index<N>`](../types/index_type.md). Generic parameters, lifetimes, and a `where` clause are carried onto
everything generated, including the companion types.

### What each shape emits, and where it is documented

`CgpData` generates the same output as the matching shape-specific derive. These pages document each
output in detail:

| Applied to | It emits | Documented on |
|---|---|---|
| a struct | per-field access, the representation, and the builder | [`#[derive(CgpRecord)]`](./derive_cgp_record.md) |
| an enum | the representation, the constructors, and the extractor | [`#[derive(CgpVariant)]`](./derive_cgp_variant.md) |

Every enum variant must carry exactly one unnamed payload, and individual variants cannot opt out.
The matching page covers the accepted shapes, generated code, and corner cases.

### Choosing among the three

Choose `CgpData`, `CgpRecord`, or `CgpVariant` according to the input restriction you want to express.
Their output is identical for the same accepted input:

- **`#[derive(CgpData)]`**: use as the default for either structs or enums.
- **[`#[derive(CgpRecord)]`](./derive_cgp_record.md)**: accept only structs.
- **[`#[derive(CgpVariant)]`](./derive_cgp_variant.md)**: accept only enums.

Use a shape-specific derive when its name documents a constraint on the type. It rejects the wrong
shape at the derive.

## Examples

One derive covers both halves of a program that builds records and takes enums apart:

```rust
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Circle {
    pub radius: f64,
}

#[derive(CgpData)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

`Circle` and `Rectangle` gain field access, a shape, and a builder; `Shape` gains a variant list,
constructors, and an extractor. Worked examples of each half are on the
[record](./derive_cgp_record.md#examples) and [variant](./derive_cgp_variant.md#examples) pages.

## When to use it

Use `CgpData` when generic code needs both a type's structure and its incremental operations. For
reading individual values from a context (the type a capability runs against, which supplies values
as fields), [`#[derive(HasField)]`](./derive_has_field.md) is sufficient.

The full derive supports these uses:

- **Records assembled from independent contributions:** the extensible builder pattern combines fields
  without requiring each contributor to know the target struct.
- **Enums handled by independent implementations:** the extensible visitor pattern lets variants and
  operations grow without requiring each implementation to know the whole enum.
- **Frameworks over user-defined types:** serializers, validators, and mappers can process the shape
  generically. Use `HasFields` alone if the framework only needs representation and conversions.

Prefer simpler operations in these cases:

- **Closed enums with fixed operations:** use a `match`, which already checks exhaustiveness.
- **Structs whose fields are only read:** derive [`HasField`](./derive_has_field.md) for per-field access.
- **One-off structural manipulations:** consider a narrower generic-programming library when a single
  conversion is the only reason to adopt this family.

Individual derives let you generate only the operations you need:

- [`HasField`](./derive_has_field.md): per-field access.
- [`HasFields`](./derive_has_fields.md): the representation and conversions.
- [`BuildField`](./derive_build_field.md): the record builder.
- [`ExtractField`](./derive_extract_field.md): the enum extractor.
- [`FromVariant`](./derive_from_variant.md): the variant constructors.

`CgpData` adds compilation work and generated types that can appear in diagnostics. It generates one
companion type for a struct or two for an enum, plus whole-type and per-field or per-variant
implementations. Derive only the parts you need when the full set of operations is unnecessary.

## Under the hood

`CgpData` selects the record or variant code path according to its input.
[`CgpRecord`](./derive_cgp_record.md#under-the-hood) and
[`CgpVariant`](./derive_cgp_variant.md#under-the-hood) call those paths directly, which is why their
output matches. Their pages show the generated implementations.

All three derives reject unions. The family models structs and enums.

## Common Mistakes

The applicable restrictions depend on the input shape. The
[record page](./derive_cgp_record.md#common-mistakes) covers cleared companion attributes, positional
tuple builders, and the newtype representation. The
[variant page](./derive_cgp_variant.md#common-mistakes) covers payload shapes and reserved names.
The following restrictions apply when choosing the combined derive.

**Every enum variant needs exactly one unnamed payload.** Unit, multi-field, and struct-style variants
are rejected, and individual variants cannot opt out. [`HasFields`](./derive_has_fields.md) accepts
all of these shapes when only a representation and whole-value conversions are needed.

**Record and variant absence use different markers.** `IsNothing` represents a missing record field;
`IsVoid` represents a ruled-out variant. An error mentioning the wrong marker usually means record
and variant operations have been mixed.

**The combined derive generates more code than an individual derive.** A five-field struct gains a
companion type and per-field implementations in addition to its whole-type implementations. Use an
individual derive when only part of that output is needed.

## Related constructs

These references cover the related derives, generated traits, and supporting types:

- [`#[derive(CgpRecord)]`](./derive_cgp_record.md) and
  [`#[derive(CgpVariant)]`](./derive_cgp_variant.md) — the two faces this dispatches to.
- [`#[derive(HasField)]`](./derive_has_field.md) — the per-field access slice, on its own.
- [`#[derive(HasFields)]`](./derive_has_fields.md) — the representation slice, on its own, and the only
  one that accepts every variant shape.
- [`#[derive(BuildField)]`](./derive_build_field.md) — the record builder slice, on its own.
- [`#[derive(ExtractField)]`](./derive_extract_field.md) — the extractor slice, on its own.
- [`#[derive(FromVariant)]`](./derive_from_variant.md) — the variant constructors, on their own.
- [`HasBuilder`](../traits/builder/has_builder.md) and [`ExtractField`](../traits/variant/extract_field.md) — the two
  trait families the shapes generate impls for.
- [`MapType`](../traits/type-level/map_type.md) — the markers the companion types are parameterized by.

The ideas behind it are explained on these concept pages:

- [Extensible records](/docs/concepts/extensible-records) — the struct half, and the extensible builder
  pattern.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum half, and the extensible visitor
  pattern.
- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to a handler per field or
  variant.

## Source

The implementation is defined in these source files:

- Entry point: [`cgp_data.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_data.rs)
- Shape dispatch: [`cgp_data/item.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/item.rs)
- Record and variant codegen: [`cgp_data/record.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/record.rs)
  and [`cgp_data/variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/variant.rs)
- Runtime traits: [`cgp-field/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-field/src/traits)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
