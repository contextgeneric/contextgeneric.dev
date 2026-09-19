---
sidebar_label: 'Extensible variants'
sidebar_position: 14
---

# Extensible variants

Extensible variants let generic code construct and handle enums through variant names and payload
types. Handlers can be reused across compatible enums, while the compiler still checks that every
variant is covered. This page explains the structural representation, exhaustiveness, and conversions
between enums that share variants. [Extensible records](./extensible-records.md) introduces the
related field representation.

## The problem a `match` builds in

An exhaustive `match` makes an operation's cases explicit. Adding a variant to an enum makes existing
matches fail to compile unless they already have a pattern that covers it. This is useful when each
operation should be reviewed as the enum changes.

Reusable per-variant logic does not always need to change with the enum. A circle-area calculation,
for example, can serve several shape enums, including one extended with triangles. Ordinary helper
functions can share that calculation, but each concrete `match` still has to connect its enum's
variants to the helpers.

CGP separates the payload handlers from the generic dispatcher that selects them. An operation can
then support another enum by supplying handlers for its additional payloads, leaving existing
handlers unchanged. The enum definitions remain ordinary closed Rust enums: a downstream crate
extends the design by defining another enum, not by adding variants to an upstream definition.

## An enum as a list of named variants

[`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data) exposes an enum's structure and
provides generic construction and extraction operations:

```rust
#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

For `Shape`, the generated `HasFields::Fields` associated type is a sum of named payloads:

```rust
Sum![Field<Symbol!("Circle"), Circle>, Field<Symbol!("Rectangle"), Rectangle>]
```

Each `Field` pairs a variant-name tag with its payload type. A sum holds one of those alternatives,
where a record's product holds all its fields. Generic code can require an operation on the
`Circle` variant without naming `Shape` itself.

Construction uses the same tags. A `FromVariant` bound lets generic code supply a payload for a
named variant, while the implementing enum determines how to construct the value. Extraction
provides the corresponding operation for taking a payload out.

## Exhaustiveness without a wildcard

A **partial variant** tracks which alternatives remain possible during extraction. Converting an
enum value to its extractor starts with every variant possible. Each extraction attempt either
returns the requested payload or returns a remainder whose type excludes that variant.

An unsuccessful attempt narrows the remainder type without losing the value. For `Shape`, failing
to extract `Circle` leaves only `Rectangle` possible. If every variant has been excluded, the
remainder is uninhabited: its type cannot contain a value.

Finalization requires an uninhabited remainder, so the compiler verifies that extraction covers
every variant. The final operation eliminates that impossible remainder without a wildcard arm or
`unreachable!()`. Adding an unhandled variant leaves a possible case and prevents finalization from
compiling. Borrowed and mutable extractor forms preserve the same tracking without consuming the
original enum.

## One handler, many enums

A generic handler can cover several payload types and therefore serve several enums. This handler
converts any `Display` payload to a string, while `MatchWithFieldHandlers` generates an extraction
step for each variant:

```rust
#[cgp_computer]
pub fn field_to_string<Tag, Value>(Field { value, .. }: Field<Tag, Value>) -> String
where
    Value: Display,
{
    value.to_string()
}

delegate_components! {
    App {
        ComputerComponent: MatchWithFieldHandlers<FieldToString>,
    }
}
```

Suppose `Reading` contains `Temperature(u64)` and `Label(String)`, while `ExtendedReading` also
contains `Flag(bool)`. The same wiring handles both enums because every payload implements `Display`.
The handler ignores the variant tag and formats its value. An added payload without `Display` would
make this wiring fail to satisfy the enum's computation.

Separate providers can supply different behavior for different payloads. In the **extensible
visitor pattern**, those providers remain independent of the full enum, and a dispatcher routes the
current variant to its handler. [Dispatching](./dispatching.md) explains how the context selects them.

## Converting enums that share variants {#enums-that-share-variants-convert-for-free}

Structural casts convert compatible enums without a handwritten conversion implementation. They
match variant names and payload types; sharing names alone is insufficient. The generated code
selects the current variant and reconstructs it in the target enum at runtime.

Widening always succeeds when the target supports every source variant with the same payload type.
For the `Reading` and `ExtendedReading` types above:

```rust
let wide: ExtendedReading = narrow.upcast(PhantomData::<ExtendedReading>);
```

Narrowing checks whether the current variant belongs to the smaller target enum. It returns the
target value on success or an extraction remainder on failure:

```rust
let narrow: Result<Reading, _> = wide.downcast(PhantomData::<Reading>);
```

`ExtendedReading::Label` can become `Reading::Label`, but `ExtendedReading::Flag` cannot become a
`Reading`. The remainder preserves the unmatched value, and `CanDowncastFields` can try it against
another target. Import the cast traits from `cgp::core::field::impls` when using these methods.

A provider can also construct a small local enum and widen it into a compatible application enum.
The provider then needs to know only the variants it produces, while the application chooses the
complete result type.

## What it costs

The full construction and extraction derives require exactly one unnamed payload per variant.
A richer case must wrap its fields in a payload type, such as `Circle(Circle)` instead of
`Circle { radius: u64 }`. Unit variants also need a payload if they are to use these derives.
[`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields) can describe all variant shapes,
but that structural description alone does not supply generic constructors or extractors.

An enum must implement the traits used by the generic operation. A foreign enum without them is
not automatically available to the dispatcher, and CGP does not permit adding variants to it.
A local enum with the required support can provide the extensible boundary and convert to other
representations where necessary.

Unhandled variants can produce long trait errors. The diagnostic may name a missing payload-handler
bound or a partial-variant type whose markers describe the remaining cases. This is less direct to
read than an ordinary non-exhaustive `match` error.

A concrete `match` is usually easier to follow when an enum and its operations are maintained
together. Extensible variants help when operations must work across different enum definitions or
when new cases should reuse existing handlers without editing them.

## Where to go next

These pages explain the related representations and operations:

- [Extensible records](./extensible-records.md): Field-based construction and completeness.
- [Dispatching](./dispatching.md): Connecting variants to handlers.
- [Type-level DSLs](./type-level-dsls.md): Using composable handlers to interpret typed programs.
- [`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data) and
  [`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields): Structural support and its subsets.
- [`#[derive(ExtractField)]`](/docs/reference/derives/derive_extract_field) and
  [`ExtractField`](/docs/reference/traits/variant/extract_field): Extraction and narrowed remainders.
- [`#[derive(FromVariant)]`](/docs/reference/derives/derive_from_variant) and
  [`FromVariant`](/docs/reference/traits/variant/from_variant): Construction by variant name.
- [`CanUpcast`](/docs/reference/traits/casting/can_upcast) and
  [`CanDowncast`](/docs/reference/traits/casting/can_downcast): Widening and narrowing compatible enums.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
