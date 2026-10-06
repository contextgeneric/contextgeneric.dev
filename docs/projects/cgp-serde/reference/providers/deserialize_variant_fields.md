---
title: 'DeserializeVariantFields — read an enum with no serialization derive in cgp-serde'
sidebar_label: 'DeserializeVariantFields'
sidebar_position: 15
description: 'The cgp-serde provider that reads an enum deriving CGP''s variant traits from Serde''s externally tagged form, reading the payload through the context.'
---

# `DeserializeVariantFields`

Read an enum from `{"Variant": payload}`, reading the payload through the context.

## Overview

`DeserializeVariantFields` is the reading half of
[`SerializeVariantFields`](./serialize_variant_fields.md). It reads the variant's name, asks the
**context**, the type whose wiring holds the application's choices, to read that variant's payload,
and builds the enum. The enum derives only CGP's variant traits, through
[`#[derive(CgpVariant)]`](/docs/reference/derives/derive_cgp_variant) or the narrower
[`HasFields`](/docs/reference/traits/shape/has_fields) derive, and every variant holds exactly one
unnamed payload.

## Definition

```rust
pub struct DeserializeVariantFields;

#[cgp_impl(DeserializeVariantFields)]
impl<'de, Value> ValueDeserializer<'de, Value>
where
    Value: FromFields,
    Value::Fields: VariantsDeserializer<'de, Self>,
{ ... }
```

[`FromFields`](/docs/reference/traits/shape/from_fields) builds the enum from one variant of its
variant list. `VariantsDeserializer` is a private trait that finds the variant by name or position,
and it requires, for each variant, that the context can read the payload's type.

## Usage

Import it from `cgp_serde::providers`, and wire it beside an entry for each variant's payload type:

```rust
delegate_components! {
    App {
        open ValueDeserializerComponent;

        @ValueDeserializerComponent.[u64, String, ()]: UseSerde,
        @ValueDeserializerComponent.Circle: DeserializeRecordFields,
        @ValueDeserializerComponent.Shape: DeserializeVariantFields,
    }
}
```

## Behavior

The provider reads the variant's identifier, then its payload through the context. A text format
names the variant, and a binary format such as postcard gives its position in the declaration, as
Serde's derive does. With JSON, `{"Circle":{"radius":3}}` reads as `Shape::Circle`, and
`{"Empty":null}` reads as `Shape::Empty(())` because `()` is wired to `UseSerde`. The payload may
borrow from the input: an enum `Token<'a>` with a `Word(&'a str)` variant reads `{"Word":"hello"}`
without copying the string.

**The context's entry for `()` decides how a variant with no data is written.** A context that wires
`()` to a provider writing an empty map reads `{"Empty":{}}` and rejects `{"Empty":null}`. Under
either choice, the bare `"Empty"` that Serde's derive uses for a unit variant is rejected, with
`invalid type: unit variant, expected newtype variant`.

The provider reports an unknown variant with the names it expected, in the same wording as Serde's
derive:

```text
unknown variant `Square`, expected one of `Circle`, `Rectangle`, `Label`, `Empty` at line 1 column 9
```

A payload of the wrong type fails with that payload's own error, and input that is not a single
variant, such as `{}` or an array, fails with the format's error.

## Context dependencies

`CanDeserializeValue<'de, P>` for the payload type `P` of every variant.

## Pairing

[`SerializeVariantFields`](./serialize_variant_fields.md) writes the form it reads.

## When to use it

**Reach for `DeserializeVariantFields` to read an enum whose payloads should follow the context's
choices.** It reads only Serde's default enum form, with one payload per variant. An enum written in
another form, or with variants that hold several fields, keeps its own Serde impl and is wired to
[`UseSerde`](./use_serde.md).

## Related constructs

- [`DeserializeRecordFields`](./deserialize_record_fields.md) is the matching provider for a struct,
  and usually reads a variant's payload.

## The ideas behind it

- [Derive-free records](../../architecture/derive-free-records.md): reading data through CGP's field
  traits.
- [Extensible variants](/docs/concepts/extensible-variants): building an enum from one of its named
  variants.

## Source

- [`crates/cgp-serde/src/providers/variant.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/variant.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
