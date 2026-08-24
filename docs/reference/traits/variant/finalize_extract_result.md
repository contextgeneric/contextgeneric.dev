---
sidebar_label: 'FinalizeExtractResult'
sidebar_position: 6
---

# `FinalizeExtractResult`

Collapsing the `Result` at the end of an extraction chain.

## Overview

Every step of an [extraction chain](./extract_field.md) returns a `Result`: the payload on the left, the
narrowed remainder on the right. At the last step the remainder is uninhabited, so the `Err` case cannot
happen, and `FinalizeExtractResult` says so.

**This is the one you actually call.** [`FinalizeExtract`](./finalize_extract.md) is the underlying
discharge, and it applies to a remainder you already hold; this applies to the `Result` a chain hands
you, which is the shape the last `extract_field` actually returns.

## Definition

`FinalizeExtractResult` collapses the `Result` a chain's last step returns:

```rust
pub trait FinalizeExtractResult {
    type Output;

    fn finalize_extract_result(self) -> Self::Output;
}
```

`Self` is the `Result` and `Output` is the payload alone. The trait has one blanket impl, for any
`Result<T, E>` whose error half can be discharged:

```rust
impl<T, E> FinalizeExtractResult for Result<T, E>
where
    E: FinalizeExtract,
{
    type Output = T;
    // Ok(value) => value, Err(remainder) => remainder.finalize_extract()
}
```

`finalize_extract_result` returns the `Ok` value directly and discharges the `Err` half through
[`FinalizeExtract`](./finalize_extract.md), which cannot fail once the remainder is uninhabited.

## Usage

**It is not in the prelude** — the one member of the extractor family that is not. Import it from
`cgp::core::field::traits`, and you will want it, because it closes a chain:

```rust
use cgp::core::field::traits::FinalizeExtractResult;
```

There is nothing to implement. The blanket impl covers every `Result` whose error half is
dischargeable, so it applies the moment a chain has tried every variant.

## Examples

Closing a chain, which is the whole use:

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
            let rect = remainder
                .extract_field(PhantomData::<Symbol!("Rectangle")>)
                .finalize_extract_result();   // no variants left, so this cannot fail
            rect.width * rect.height
        }
    }
}
```

No `.unwrap()`, no `expect`, and no wildcard arm. The call compiles only because both variants have been
tried, which makes it a proof rather than an assertion.

Calling it one step early does not compile:

```rust
// error: the trait bound is not satisfied — Circle is still possible,
// so the remainder is inhabited and cannot be discharged
let rect = shape
    .to_extractor()
    .extract_field(PhantomData::<Symbol!("Rectangle")>)
    .finalize_extract_result();
```

## When to use it

**Use it to close every hand-written extraction chain**, and prefer not writing the chain at all.

- **`finalize_extract_result`** at the end of a chain, which is where the last `extract_field` left you
  holding a `Result`.
- **[`FinalizeExtract`](./finalize_extract.md)** when a nested `match` has already unwrapped the
  remainder out of its `Result`.
- **The [dispatch combinators](../../providers/dispatch/index.md)** rather than a hand-written chain,
  since they generate it from the enum's own variant list, which keeps "add a variant" from
  breaking every call site.
- **`.ok()` or a `match`** when you are making a single attempt and do not intend to exhaust the enum.
  Reaching for this trait then simply will not resolve.
- **A `match`** when the enum is concrete.

## Under the hood

The impl delegates to [`FinalizeExtract`](./finalize_extract.md) on the error half, which is where the
soundness argument lives: the remainder is uninhabited once every variant is `IsVoid`, so the `Err` arm
is discharged with an empty `match` and no execution path reaches it.

Because the bound is on `E` rather than on any CGP type in particular, the impl also covers a `Result`
whose error is `Infallible` or [`Void`](../../types/spines/void.md). That is occasionally useful
outside the extractor family, for collapsing a `Result` that a signature required but that cannot fail.

`Output` being an associated type rather than a generic lets the call sit at the end of a method
chain with nothing to annotate.

## Common Mistakes

**It is not in the prelude.** Import it from `cgp::core::field::traits`. Without it there is no
`finalize_extract_result` in scope and the chain has no clean ending, and the error is a
missing-method one, which reads as though the chain is wrong rather than the import missing.

**Calling it early does not compile.** Any variant still possible leaves the remainder inhabited and the
bound unsatisfied.

**It discards nothing and cannot panic.** Unlike `.unwrap()`, there is no runtime branch: the `Err` case
is discharged at the type level.

**It applies to any `Result` with a dischargeable error**, not only to extraction remainders, which is
occasionally surprising when it resolves somewhere you did not expect.

**Adding a variant breaks the call**, by design: the final remainder becomes inhabited again and the
bound stops being satisfied.

## Related constructs

- [`FinalizeExtract`](./finalize_extract.md) — the underlying discharge, and the full exhaustiveness
  argument.
- [`ExtractField`](./extract_field.md) — the narrowing that produces the `Result` this collapses.
- [`HasExtractor`](./has_extractor.md) — where a chain begins.
- [`FinalizeBuild`](../builder/finalize_build.md) — the record family's ending.
- [Type-level spines](../../types/spines/index.md) — where the uninhabited `Void` comes from.
- [Dispatch combinators](../../providers/dispatch/index.md) — the providers that build the chain for
  you.
- [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md) — what makes an enum extractable.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — the exhaustiveness argument and the
  extensible visitor pattern.

## Source

- [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)
  — `FinalizeExtractResult` and the rest of the family

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
