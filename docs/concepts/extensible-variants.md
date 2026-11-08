---
sidebar_label: 'Extensible variants'
sidebar_position: 14
description: 'How an enum variant list becomes a sum type, how extraction proves every variant is handled without a wildcard, and how one handler serves several enums.'
---

# Extensible variants

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations whose
handlers can cover enum variants. Extensible variants let generic code construct and handle enums
through variant names and payload types. By describing the cases it needs through traits, a handler
can work across compatible enums without depending on any one enum definition. The compiler still
checks that every variant is covered, even when independently written handlers supply the cases.

This uses the same field representation as [extensible records](./extensible-records.md), but an
enum contains one alternative at a time. That difference changes how generic code takes a value
apart and proves that it has handled every possibility.

## The problem a `match` builds in

With an exhaustive `match`, an operation lists the enum cases it handles. If a new variant has no
covering pattern, Rust rejects the match, prompting the operation's author to review the new case.
That is useful when the enum and its operations should evolve together.

Some of the work inside those cases may already be reusable. A circle-area calculation can serve
several shape enums, including a larger one that adds triangles. Ordinary helper functions share
that calculation, but each match must still connect its particular enum variants to those helpers.

Separating those helpers from the enum-specific connection is the purpose of CGP's generic
dispatcher. Existing payload handlers can keep working when another enum introduces additional
cases, provided those cases have handlers too. The enums remain ordinary closed Rust types:
a downstream crate defines a new enum rather than adding variants to an upstream definition.

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

This sum is the enum's **shape**. A bound or implementation can name it with
[`Enum! { Circle(Circle), Rectangle(Rectangle) }`](/docs/reference/macros/enum), using the same
syntax as the enum body. Because the macro uses the derive's encoding, that form produces exactly
the sum shown above.

These name tags also let generic code construct a value. With a `FromVariant` bound, it can supply
a payload for a named variant while leaving the enum's implementation to assemble the value.
Extraction uses the same representation in the other direction, taking a payload back out.

## Exhaustiveness without a wildcard

Extracting one payload leaves a question: what cases are still possible if the extraction misses?
A **partial variant** records that answer in its type. An enum's extractor begins with every case
possible, and each attempt either returns the requested payload or a remainder that excludes
that variant.

For `Shape`, an attempt to extract `Circle` either returns its payload or leaves a remainder in
which only `Rectangle` is possible. The value survives a failed attempt, but its type now records
that it cannot be a circle. Continuing this process until every variant has been excluded leaves
an uninhabited remainder: a type that cannot contain a value.

To finish extraction, the remainder must be uninhabited. This requirement is how the compiler
checks coverage: an unhandled variant would leave a possible value and prevent finalization from
compiling. Once coverage is complete, the final operation can eliminate the impossible remainder
without a wildcard arm or `unreachable!()`. Borrowed and mutable extractors track the same cases
while keeping the original enum in place.

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

For example, `Reading` might contain `Temperature(u64)` and `Label(String)`, while
`ExtendedReading` adds `Flag(bool)`. Because all those payloads implement `Display`, both enums
can use this wiring. `FieldToString` ignores the variant tag and formats whichever value it
receives. A new payload without `Display` would instead leave the enum's computation unsatisfied.

Separate providers can supply different behavior for different payloads. In the **extensible
visitor pattern**, those providers remain independent of the full enum, and a dispatcher routes the
current variant to its handler. [Dispatching](./dispatching.md) explains how the context selects them.

## Converting enums that share variants {#enums-that-share-variants-convert-for-free}

The same structural representation supports conversions between compatible enums. A cast matches
both the variant name and its payload type, so sharing a name alone is insufficient. At runtime,
the generated code identifies the current variant and reconstructs it in the target enum, without
a handwritten conversion implementation.

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

A `Label` value can pass from `ExtendedReading` to `Reading`, because both contain that case.
A `Flag` cannot, so narrowing returns its value in the extraction remainder instead. The remainder
can then be tried against another target with `CanDowncastFields`. These cast methods require
imports from `cgp::core::field::impls`.

A provider can also construct a small local enum and widen it into a compatible application enum.
The provider then needs to know only the variants it produces, while the application chooses the
complete result type.

## What it costs

Generic construction and extraction also constrain how variants are declared: the full derives
accept one unnamed payload or no fields. A richer case must collect its fields in a payload type,
as in `Circle(Circle)` rather than `Circle { radius: u64 }`. A fieldless variant uses `Nil` as its
payload. Because every fieldless variant has that same payload type, a handler needs the variant's
name as well as its payload to distinguish them.
[`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields) can describe all variant shapes,
but that structural description alone does not supply generic constructors or extractors.

These generic operations are available only where the enum implements the required traits.
A foreign enum without them cannot be dispatched automatically, nor can CGP add variants to its
definition. Where that boundary matters, a supported local enum can provide the extensible
representation and convert to other forms as needed.

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
- [Comparison: Row polymorphism](/docs/comparisons/row-polymorphism): variants as row-typed sums: injection, branching, and the expression problem.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
