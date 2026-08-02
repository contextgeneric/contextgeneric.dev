---
sidebar_label: 'ExtractField'
---

# `ExtractField`

The incremental-extractor trait family behind the extensible visitor pattern.

## What it's for

A `match` on a concrete enum is checked for exhaustiveness. Code that is generic over the enum cannot write
one, so it falls back on a wildcard arm and an `unreachable!()` — losing exactly the guarantee you wanted.

This family recovers it. An enum is taken apart one variant at a time, and **the still-possible variants are
tracked in the type**. Each attempt either yields the payload or hands back a *remainder* whose type has that
variant ruled out:

```rust
match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
    Ok(circle) => /* it was a Circle */,
    Err(remainder) => /* it wasn't; `remainder` can no longer be a Circle */,
}
```

Keep going and the remainder narrows. Once every variant has been ruled out its type is **uninhabited** — a
value of it cannot exist — and that is what closes the chain with no wildcard and no panic path. Add a variant
to the enum and the final remainder becomes inhabited again, so the code stops compiling until it is handled.

The family is the mirror of the [builder](./has_builder.md): a builder tracks which fields are *present*, an
extractor tracks which variants are still *possible*. The impls come from
[`#[derive(ExtractField)]`](../derives/derive_extract_field.md), which generates the partial companion enums
they operate on.

## Using it

Most of the family is in the prelude — `HasExtractor`, `HasExtractorRef`, `HasExtractorMut`, `ExtractField`,
`FinalizeExtract`. The exception is **`FinalizeExtractResult`, which is not**; import it from
`cgp::core::field::traits`, and you will want it, because it is what closes a chain.

### The three accessors

They differ only in ownership, and picking the right one is most of using the family:

```rust
pub trait HasExtractor {
    type Extractor;
    fn to_extractor(self) -> Self::Extractor;
    fn from_extractor(extractor: Self::Extractor) -> Self;
}

pub trait HasExtractorRef {
    type ExtractorRef<'a> where Self: 'a;
    fn extractor_ref(&self) -> Self::ExtractorRef<'_>;
}

pub trait HasExtractorMut {
    type ExtractorMut<'a> where Self: 'a;
    fn extractor_mut(&mut self) -> Self::ExtractorMut<'_>;
}
```

`to_extractor` **consumes** the value and yields owned payloads; `from_extractor` reverses it for a value that
was not matched. `extractor_ref` borrows, so payloads come out as shared references and the original survives.
`extractor_mut` borrows mutably, so a payload can be changed in place.

### The extraction

```rust
pub trait ExtractField<Tag> {
    type Value;
    type Remainder;
    fn extract_field(self, _tag: PhantomData<Tag>) -> Result<Self::Value, Self::Remainder>;
}
```

`Ok` carries the payload; `Err` carries the same extractor with this one variant ruled out. The impl exists
only while the requested variant is still possible, so attempting the same variant twice is a compile error
rather than a guaranteed miss.

### The finish

```rust
pub trait FinalizeExtract {
    fn finalize_extract<T>(self) -> T;
}

pub trait FinalizeExtractResult {
    type Output;
    fn finalize_extract_result(self) -> Self::Output;
}
```

`finalize_extract` returns *any* type, which is sound only because there is no value to return it from — it is
implemented for the uninhabited [`Void`](../types/type_level_spines.md), for `Infallible`, and for the
all-ruled-out configuration of a partial enum.

`finalize_extract_result` is the one you actually call. It collapses the `Result` from the last extraction,
and it is implemented for any `Result<T, E>` whose error half can be finalized:

```rust
impl<T, E> FinalizeExtractResult for Result<T, E>
where
    E: FinalizeExtract,
{
    type Output = T;
    // Ok(value) => value, Err(remainder) => remainder.finalize_extract()
}
```

## Examples

The chain reads as a sequence of attempts, each handling one variant, closed by
`finalize_extract_result`:

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

After the second extraction both variants are ruled out, so the remainder's type is uninhabited and
`finalize_extract_result` is accepted with no wildcard arm. That is not a convention — it is the type. Try to
finalize after only the first extraction and it does not compile.

Reading through a borrow leaves the value intact:

```rust
let radius = shape
    .extractor_ref()
    .extract_field(PhantomData::<Symbol!("Circle")>)
    .map(|circle| circle.radius)
    .ok();
```

and the mutable form changes a payload in place:

```rust
if let Ok(circle) = shape
    .extractor_mut()
    .extract_field(PhantomData::<Symbol!("Circle")>)
{
    circle.radius = 5.0;
}
```

**In practice you rarely write these chains.** The
[dispatch combinators](../providers/dispatch_combinators.md) build them from a set of per-variant
implementations, which is the extensible visitor pattern — a chain exactly like the one above, generated, with
one implementation per variant chosen by wiring.

## When to reach for it, and when not

**Reach for the family when independent code handles one variant each, or when the matching code cannot name
the enum.** Outside those two cases a `match` wins on every count: shorter, clearer, already exhaustive, and it
generates nothing. This is not an improvement on `match`; it is what you use where `match` is unavailable.

- **Bound on `HasExtractor` + `ExtractField`** to write a routine that deconstructs an enum it does not name.
- **Pick the accessor by ownership**, and prefer the weakest that works: `extractor_ref` for a read-only
  operation over a value you do not own, `extractor_mut` to mutate a payload, `to_extractor` only when you
  genuinely want to consume.
- **Do not use it to test which variant a value holds.** `matches!` or an `if let` answers that in a line. The
  family's value is in the *chain* and what the chain proves.
- **Reach for the [dispatch combinators](../providers/dispatch_combinators.md) rather than writing the chain**,
  since they derive it from the enum's own variant list instead of repeating it at each site — which is what
  keeps "add a variant" from breaking every call site by hand.

The construction counterpart is [`FromVariant`](./from_variant.md), and the two are commonly wanted together
because a generic pipeline usually takes a value apart and puts one back. The struct analogue of the whole
family is [`HasBuilder`](./has_builder.md).

## Under the hood

:::note

### Advanced

This section shows how the exhaustiveness argument works. You do not need it to use the family, but the partial
enums appear by name in every extraction error, and the `IsVoid`-versus-`IsNothing` distinction is the one thing
here that genuinely surprises people.

:::

The derive generates a companion enum with one [`MapType`](./map_type.md) parameter per variant, each payload
wrapped in that parameter's projection:

```rust
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Rectangle(<__F1__ as MapType>::Map<Rectangle>),
}
```

`to_extractor` starts at the all-`IsPresent` configuration, where every variant is still possible. Each
`extract_field` impl is in scope only while its variant's marker is `IsPresent`; on a miss it returns the
remainder with that one marker flipped to `IsVoid`, leaving the rest generic — which is why extractions may
happen in any order.

**The exhaustiveness argument turns on what `IsVoid` maps to.** It maps a payload to the uninhabited `Void`, so
once every marker is `IsVoid` every arm of the partial enum holds a `Void` and the whole type is uninhabited.
The derive supplies a `FinalizeExtract` impl on exactly that configuration, and it is sound only because the
value cannot exist:

```rust
impl FinalizeExtract for __PartialShape<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}       // no arms to write
    }
}
```

So a caller reaches `finalize_extract` only after trying every variant, and the compiler accepts the discharge
with no fallback.

**This is where the record and variant families diverge, and the difference is load-bearing.** A record uses
`IsNothing` for a missing field, which maps to `()` and is *inhabited* — an absent field is a real state a
value can be in. A variant uses `IsVoid`, which is *uninhabited* — a ruled-out variant is a state no value can
be in. That is exactly why a builder needs an explicit all-present `FinalizeBuild` impl while an extractor can
discharge its remainder with an empty `match`.

The borrowed accessors use the same partial enum with an extra [`MapTypeRef`](./map_type.md) parameter fixed to
`IsRef` or `IsMut`, mapping each payload slot to a shared or mutable reference — so a value can be matched
without being moved, and the narrowing works identically.

## Gotchas

**`FinalizeExtractResult` is not in the prelude.** Import it from `cgp::core::field::traits`. Without it there
is no `finalize_extract_result` in scope and the chain has no clean ending.

**Absence is `IsVoid` here and `IsNothing` in a builder, and they are not interchangeable.** An error naming the
wrong one usually means record and variant machinery have been crossed.

**Finalizing early does not compile, and the error names the partial enum.** The all-void impl does not apply
while any marker is still `IsPresent`, so the compiler reports a missing method. Read the type in the message to
see which variants remain.

**A remainder carries none of the enum's attributes.** The partial enums are generated without your derives, so
a `Result<Payload, Remainder>` is neither `Debug` nor `PartialEq` however the enum is derived — `assert_eq!` on
the whole result does not compile. Reach for `.ok()`, `.is_ok()`, or a `match`.

**Order is free but the set is not.** Each step changes only its own variant's marker, so extractions may be
written in any order — but every variant must be tried before the remainder can be finalized.

**Adding a variant breaks every hand-written chain, by design.** That is the guarantee, and it is the reason to
prefer the [dispatch combinators](../providers/dispatch_combinators.md).

**Five enum variant names are reserved**, because the generated impls name their associated types through
`Self::…`. The [derive's page](../derives/derive_extract_field.md) lists them.

## Related constructs

- [`#[derive(ExtractField)]`](../derives/derive_extract_field.md) — generates the partial enums and every impl
  here.
- [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) — bundles this family with the shape and the
  constructors.
- [`FromVariant`](./from_variant.md) — the construction counterpart, commonly derived alongside.
- [`HasBuilder`](./has_builder.md) — the struct analogue, and where the `IsNothing`/`IsVoid` contrast comes from.
- [`MapType`](./map_type.md) — the `IsPresent`/`IsVoid` markers and the `MapTypeRef` borrow markers.
- [`HasFields`](./has_fields.md) — the variant shape the casts and dispatchers walk.
- [`CanUpcast`](./cast.md) — upcasting and downcasting, built on this recursion.
- [Type-level spines](../types/type_level_spines.md) — `Either`/`Void`, where the uninhabited terminator comes
  from.
- [Dispatch combinators](../providers/dispatch_combinators.md) — the providers that build the chain for you.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — partial variants, the exhaustiveness argument,
  and the extensible visitor pattern.
- [Dispatching](/docs/concepts/dispatching) — routing a variant to the implementation that handles it.

## Source

- [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)
  — every trait on this page
- [`partial_data.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/partial_data.rs)
  — `PartialData`, which the partial enums also implement
- [`map_type_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/map_type_ref.rs)
  — the `IsRef`/`IsMut` borrow markers

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
