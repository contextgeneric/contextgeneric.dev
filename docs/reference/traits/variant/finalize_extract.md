---
sidebar_label: 'FinalizeExtract'
sidebar_position: 5
---

# `FinalizeExtract`

Discharging an extractor that has nothing left in it.

## Overview

An [extraction chain](./extract_field.md) ends when every variant has been ruled out. At that point the
remainder's type is **uninhabited** — a value of it cannot exist — and `FinalizeExtract` turns
that fact into a usable ending.

**It returns *any* type.** That looks unsound and is not, because there is no value to return it from:
the method can only be called on something that cannot exist, so its body is an empty `match` and no
execution path reaches it.

That closes a generic match with **no wildcard arm and no `unreachable!()`**. It is the mirror of
[`FinalizeBuild`](../builder/finalize_build.md), and the two are sound for opposite reasons: a build finalizes
because the value is *complete*, an extraction because the value is *impossible*.

## Definition

```rust
pub trait FinalizeExtract {
    fn finalize_extract<T>(self) -> T;
}
```

`Self` is the exhausted remainder, and `finalize_extract` consumes it to return whatever type `T` the
surrounding code needs. That return is sound because the receiver is uninhabited: no value of it can
exist, so no execution path reaches the method. The trait is implemented for the uninhabited
[`Void`](../../types/type_level_spines.md), the standard `Infallible`, and the all-ruled-out
configuration of a partial enum.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

**You will usually call [`FinalizeExtractResult`](./finalize_extract_result.md) instead**, which collapses
the `Result` a last extraction returns and calls this on the error half. Reaching for `finalize_extract`
directly means you already hold the remainder rather than a `Result`.

The impls come from [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md).

## Examples

Called directly, on a remainder held in hand:

```rust
use cgp::prelude::*;

match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
    Ok(circle) => handle_circle(circle),
    Err(remainder) => match remainder.extract_field(PhantomData::<Symbol!("Rectangle")>) {
        Ok(rect) => handle_rectangle(rect),
        Err(remainder) => remainder.finalize_extract(),   // uninhabited: cannot be reached
    },
}
```

The last arm returns whatever the surrounding `match` needs, and the compiler accepts it because the
value being matched cannot exist.

The same chain written with the `Result` helper is shorter and is the form most code uses:

```rust
use cgp::core::field::traits::FinalizeExtractResult;

let rect = remainder
    .extract_field(PhantomData::<Symbol!("Rectangle")>)
    .finalize_extract_result();
```

## When to use it

**Reach for [`FinalizeExtractResult`](./finalize_extract_result.md) unless you already hold a
remainder.** The two do the same job at different points in the chain.

- **[`FinalizeExtractResult`](./finalize_extract_result.md)** to close a chain whose last step returned a
  `Result`, which is nearly always.
- **`FinalizeExtract`** when a remainder has already been unwrapped out of its `Result`, typically inside
  a nested `match`.
- **The [dispatch combinators](../../providers/dispatch/index.md)** rather than writing the chain at
  all: they generate it from the enum's variant list, which stops "add a variant" from breaking
  every call site by hand.
- **A `match`** when the enum is concrete, where Rust's own exhaustiveness check already does this job.

## Under the hood

**Everything turns on what `IsVoid` maps to.** The [`MapType`](../type-level/map_type.md) marker `IsVoid` maps a
payload to the uninhabited [`Void`](../../types/type_level_spines.md), so once every variant's marker is
`IsVoid`, every arm of the partial enum holds a `Void` and **the whole type is uninhabited**.

The derive supplies an impl on exactly that configuration:

```rust
impl FinalizeExtract for __PartialShape<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}       // no arms to write
    }
}
```

An empty `match` is legal precisely because no value of the scrutinee's type can exist, and it satisfies
any return type for the same reason. So a caller reaches `finalize_extract` only after trying every
variant, and the compiler accepts the discharge with no fallback.

**This is where the record and variant families diverge, and the difference decides the whole design.** A record
uses `IsNothing` for a missing field, which maps to `()` and is *inhabited* — an absent field is a real
state a value can be in. A variant uses `IsVoid`, which is *uninhabited* — a ruled-out variant is a state
no value can be in. That is exactly why a builder needs an explicit all-present
[`FinalizeBuild`](../builder/finalize_build.md) impl while an extractor can discharge its remainder with an empty
`match`.

## Common Mistakes

**Finalizing early does not compile, and the error names the partial enum.** The all-void impl does not
apply while any marker is still `IsPresent`, so the compiler reports a missing method. Read the type in
the message to see which variants remain.

**It returns any type, which reads as unsound and is not.** The generic return is sound only because the
receiver is uninhabited; there is no value and no execution.

**Absence is `IsVoid` here and `IsNothing` in a builder.** An error naming the wrong one usually means
record and variant machinery have been crossed.

**Adding a variant re-inhabits the final remainder**, so every hand-written chain stops compiling. That
is the guarantee, and the reason to prefer the
[dispatch combinators](../../providers/dispatch/index.md).

**It also covers `Infallible`**, which is occasionally useful when threading a chain through an error type
that cannot occur.

**It consumes the remainder**, though there was never anything to consume.

## Related constructs

- [`FinalizeExtractResult`](./finalize_extract_result.md) — the form that closes a chain from a `Result`.
- [`ExtractField`](./extract_field.md) — the narrowing that leads here.
- [`HasExtractor`](./has_extractor.md) — where a chain begins.
- [`FinalizeBuild`](../builder/finalize_build.md) — the record family's ending, sound for the opposite reason.
- [`MapType`](../type-level/map_type.md) — the `IsVoid` marker the argument rests on.
- [Type-level spines](../../types/type_level_spines.md) — where the uninhabited `Void` comes from.
- [`CanUpcast`](../casting/can_upcast.md) — a cast whose total walk ends with this same discharge.
- [Dispatch combinators](../../providers/dispatch/index.md) — the providers that build the chain.
- [`#[derive(ExtractField)]`](../../derives/derive_extract_field.md) — generates this impl.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — the exhaustiveness argument, in full.

## Source

- [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)
  — `FinalizeExtract` and the rest of the family

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
