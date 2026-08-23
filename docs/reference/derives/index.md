---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Derives

A derive here turns an ordinary struct or enum into data that generic CGP code can work with, without
that code ever naming the concrete type. Each one generates trait impls beside the type rather than
changing the type itself, so a struct stays an ordinary struct and gains the machinery alongside it.
This page groups every derive in this section by the job it does, in roughly the order most CGP code
reaches for them.

If you are new to CGP, start with the [reference overview](/docs/reference/) rather than here; this page
is the fuller map for once the essentials are familiar.

## Reading a value from a context

The two field-access derives are the ones nearly every CGP program uses.

[`#[derive(HasField)]`](./derive_has_field.md) is the foundational one, and usually the first derive
anyone writes. It gives each field a type-level name, so an implementation can ask for a field by name as
a trait bound and read it without knowing the concrete type. Every ergonomic value read stands on it: an
[`#[implicit]`](../attributes/implicit.md) argument and a
[`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) method both generate a `HasField` bound from a name
you already wrote.

[`#[derive(HasFields)]`](./derive_has_fields.md) gives the whole shape instead of one field: the struct
or enum described as a single type-level list, plus the conversions that move values in and out of it.
Reach for it when code must process every field at once, such as a serializer or a builder that merges
two records. It is often derived alongside `HasField`.

## Turning a type into extensible data

These three derives add the machinery for building a value up field by field, or taking one apart variant
by variant, with the progress tracked in the type so the compiler checks it. Each includes the field
access and representation above and adds the incremental half on top.

[`#[derive(CgpData)]`](./derive_cgp_data.md) is the umbrella and the default. On a struct it emits the
record machinery; on an enum, the variant machinery. Use it unless you have a reason to name the shape.

[`#[derive(CgpRecord)]`](./derive_cgp_record.md) is the struct-only face, and
[`#[derive(CgpVariant)]`](./derive_cgp_variant.md) the enum-only face. Each emits exactly what `CgpData`
emits for that shape and rejects the other shape at parse time, so a type that is meant to stay one kind
says so and a wrong shape fails at the derive.

## Deriving one slice on its own

Each half of the extensible-data output is also available as its own derive, for a type that needs only
part of the machinery.

[`#[derive(BuildField)]`](./derive_build_field.md) is the record builder alone: assemble a struct one
field at a time, from code that does not know the whole struct. [`#[derive(ExtractField)]`](./derive_extract_field.md)
is the enum extractor alone: take an enum apart one variant at a time, with a provably exhaustive end.
[`#[derive(FromVariant)]`](./derive_from_variant.md) is the variant constructors alone: build an enum
from a variant chosen by name. The first belongs to structs; the last two belong to enums and are
commonly derived together.

## One restriction to know before you reach for them

The derives that take an enum apart, `#[derive(ExtractField)]` and `#[derive(FromVariant)]` and therefore
`#[derive(CgpData)]` and `#[derive(CgpVariant)]`, require every enum variant to carry exactly one unnamed
payload, with no per-variant opt-out. `#[derive(HasFields)]` is the exception: it only describes a
variant, so it accepts every variant shape, which makes it the derive an enum with mixed variants can
still use.
