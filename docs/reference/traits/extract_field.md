---
sidebar_label: 'ExtractField'
---

# `ExtractField`

Pulling one variant out of an extractor, or narrowing what remains.

## Overview

A `match` on a concrete enum is checked for exhaustiveness. Code that is generic over the enum cannot
write one, so it falls back on a wildcard arm and an `unreachable!()` — losing exactly the guarantee you
wanted.

`ExtractField` is the operation that recovers it. Each attempt either yields the payload or hands back a
*remainder* whose type has that variant ruled out:

```rust
pub trait ExtractField<Tag> {
    type Value;
    type Remainder;

    fn extract_field(self, _tag: PhantomData<Tag>) -> Result<Self::Value, Self::Remainder>;
}
```

`Ok` carries the payload; `Err` carries the same extractor with this one variant ruled out. **The impl
exists only while the requested variant is still possible**, so attempting the same variant twice is a
compile error rather than a guaranteed miss.

Keep going and the remainder narrows. Once every variant has been ruled out its type is **uninhabited** —
a value of it cannot exist — and [`FinalizeExtract`](./finalize_extract.md) is what closes the chain with
no wildcard and no panic path. Add a variant to the enum and the final remainder becomes inhabited again,
so the code stops compiling until it is handled.

The family is the mirror of the [builder](./has_builder.md): a builder tracks which fields are *present*,
an extractor tracks which variants are still *possible*.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

`Self` is an extractor rather than the enum, so a chain begins by obtaining one —
[`to_extractor`](./has_extractor.md) to consume the value, [`extractor_ref`](./has_extractor_ref.md) to
borrow it, or [`extractor_mut`](./has_extractor_mut.md) to borrow it mutably. Which one you pick decides
whether the payloads come out owned, shared, or mutable; the narrowing is identical in all three.

The `PhantomData<Tag>` argument names the variant, keyed by [`Symbol!`](../macros/symbol.md) of its name.

The impls come from [`#[derive(ExtractField)]`](../derives/derive_extract_field.md), also available
through [`#[derive(CgpVariant)]`](../derives/derive_cgp_variant.md) and
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md).

## Examples

The chain reads as a sequence of attempts, each handling one variant, closed by
[`finalize_extract_result`](./finalize_extract_result.md):

```rust
use cgp::core::field::traits::FinalizeExtractResult;
use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

fn area(shape: Shape) -> f64 {
    match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
        Ok(circle) => core::f64::consts::PI * circle.radius * circle.radius,
        Err(remainder) => {
            // `remainder` now has Circle ruled out
            let rect = remainder
                .extract_field(PhantomData::<Symbol!("Rectangle")>)
                .finalize_extract_result();   // uninhabited; cannot fail
            rect.width * rect.height
        }
    }
}
```

After the second extraction both variants are ruled out, so the remainder's type is uninhabited and the
finalize is accepted with no wildcard arm. That is not a convention — it is the type. Try to finalize
after only the first extraction and it does not compile.

**In practice you rarely write these chains.** The
[dispatch combinators](../providers/dispatch/index.md) build them from a set of per-variant
implementations, which is the extensible visitor pattern: a chain exactly like the one above, generated,
with one implementation per variant chosen by wiring.

## When to reach for it, and when not

**Reach for the family when independent code handles one variant each, or when the matching code cannot
name the enum.** Outside those two cases a `match` wins on every count: shorter, clearer, already
exhaustive, and it generates nothing. This is not an improvement on `match`; it is what you use where
`match` is unavailable.

- **Bound on [`HasExtractor`](./has_extractor.md) + `ExtractField`** to write a routine that
  deconstructs an enum it does not name.
- **Pick the accessor by ownership**, preferring the weakest that works:
  [`extractor_ref`](./has_extractor_ref.md) for a read-only operation,
  [`extractor_mut`](./has_extractor_mut.md) to mutate a payload,
  [`to_extractor`](./has_extractor.md) only when you genuinely want to consume.
- **Do not use it to test which variant a value holds.** `matches!` or an `if let` answers that in a
  line. The family's value is in the *chain* and what the chain proves.
- **Reach for the [dispatch combinators](../providers/dispatch/index.md) rather than writing the
  chain**, since they derive it from the enum's own variant list instead of repeating it at each site —
  which is what keeps "add a variant" from breaking every call site by hand.

The construction counterpart is [`FromVariant`](./from_variant.md), and the two are commonly wanted
together because a generic pipeline usually takes a value apart and puts one back. The struct analogue of
the whole family is [`HasBuilder`](./has_builder.md).

## Under the hood

The derive generates a companion enum with one [`MapType`](./map_type.md) parameter per variant, each
payload wrapped in that parameter's projection:

```rust
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Rectangle(<__F1__ as MapType>::Map<Rectangle>),
}
```

[`to_extractor`](./has_extractor.md) starts at the all-`IsPresent` configuration, where every variant is
still possible. **Each `extract_field` impl is in scope only while its variant's marker is `IsPresent`**;
on a miss it returns the remainder with that one marker flipped to `IsVoid`, leaving the rest generic —
which is why extractions may happen in any order.

That per-variant scoping is the mirror of [`BuildField`](./build_field.md)'s requirement that a field be
`IsNothing` before it can be set, and it is what makes a repeated attempt a compile error rather than a
guaranteed `Err`.

The exhaustiveness argument itself belongs to [`FinalizeExtract`](./finalize_extract.md), and turns on
`IsVoid` mapping a payload to the uninhabited `Void`.

The borrowed accessors use the same partial enum with an extra [`MapTypeRef`](./map_type_ref.md)
parameter fixed to `IsRef` or `IsMut`, so a value can be matched without being moved and the narrowing
works identically.

## Common Mistakes

**`Self` is an extractor, not the enum.** A chain starts with [`to_extractor`](./has_extractor.md) or one
of its borrowing siblings.

**Attempting the same variant twice does not compile.** The marker is already `IsVoid`, so no impl
applies, reported as a missing method.

**Absence is `IsVoid` here and `IsNothing` in a builder, and they are not interchangeable.** An error
naming the wrong one usually means record and variant machinery have been crossed.

**A remainder carries none of the enum's attributes.** The partial enums are generated without your
derives, so a `Result<Payload, Remainder>` is neither `Debug` nor `PartialEq` however the enum is derived
— `assert_eq!` on the whole result does not compile. Reach for `.ok()`, `.is_ok()`, or a `match`.

**Order is free but the set is not.** Each step changes only its own variant's marker, so extractions may
be written in any order — but every variant must be tried before the remainder can be finalized.

**Adding a variant breaks every hand-written chain, by design.** That is the guarantee, and it is the
reason to prefer the [dispatch combinators](../providers/dispatch/index.md).

**Five enum variant names are reserved**, because the generated impls name their associated types through
`Self::…`. The [derive's page](../derives/derive_extract_field.md) lists them.

## Related constructs

- [`HasExtractor`](./has_extractor.md), [`HasExtractorRef`](./has_extractor_ref.md), and
  [`HasExtractorMut`](./has_extractor_mut.md) — the three ways to obtain an extractor.
- [`FinalizeExtract`](./finalize_extract.md) and
  [`FinalizeExtractResult`](./finalize_extract_result.md) — how a chain ends.
- [`FromVariant`](./from_variant.md) — the construction counterpart.
- [`HasBuilder`](./has_builder.md) — the struct analogue of the whole family.
- [`MapType`](./map_type.md) — the `IsPresent`/`IsVoid` markers, and where the contrast with `IsNothing`
  is explained.
- [`CanDowncast`](./can_downcast.md) — narrowing to another enum rather than to a payload, built on this.
- [`#[derive(ExtractField)]`](../derives/derive_extract_field.md) — generates the partial enums and every
  impl here.
- [Dispatch combinators](../providers/dispatch/index.md) — the providers that build the chain for
  you.
- [Type-level spines](../types/type_level_spines.md) — `Either`/`Void`, where the uninhabited terminator
  comes from.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — partial variants, the exhaustiveness
  argument, and the extensible visitor pattern.
- [Dispatching](/docs/concepts/dispatching) — routing a variant to the implementation that handles it.

## Source

- [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)
  — `ExtractField` and the rest of the family

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
