---
sidebar_label: 'Void'
sidebar_position: 8
---

# `Void`

The uninhabited end marker of the sum list: an empty enum that can never be constructed, which is what
lets a total variant match close with no runtime branch.

:::info

### Generated machinery

**You are not expected to write `Void` by hand.** It is produced by [`Sum!`](../macros/sum.md) as the
terminator of a sum, and it appears in the extractor machinery. You meet it at the end of an
[`Either`](either.md) chain and in an extraction error, and this page explains why it is uninhabited.

:::

## Overview

`Void` marks the end of a sum, and unlike the [`Nil`](nil.md) that ends the other lists, it is
uninhabited: it has no values. Where a record, a string, or a path can each be empty and still exist,
an empty *choice* cannot. There is no value that is "none of the branches", so the natural terminator
for the sum list is a type with no values at all.

That uninhabitedness is not a technicality; it makes the sum list work. An [`Either`](either.md)
chain ends in `Void`, so a value that walked past every real branch would have type `Void`, which is
impossible to reach. The chain is therefore closed at its end by construction, and a match that has
handled every real variant has nothing left to handle.

## Definition

`Void` is an empty enum:

```rust
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Void {}
```

It has no variants, so it has no values: it is uninhabited, the same idea as Rust's never type. It
derives the standard traits so a sum ending in `Void` can inherit them, but no value of `Void` is ever
built, so those impls are never run on one.

## Behavior

`Void` is the base case of a recursion over a sum, and it behaves differently from the product list's
base case in exactly one way that matters. A walk over an [`Either`](either.md) chain handles each
`Left` as a branch and defers each `Right` to the tail, until the tail is `Void`, at which point there
is nothing left to handle.

The uninhabitedness is essential to the extractor machinery. After an extractor has tried every
variant of a sum and matched none, the leftover value has type `Void`, a value that cannot exist.
[`FinalizeExtract`](../traits/variant/finalize_extract.md) for `Void` turns that into any required
type with an empty `match self {}`, since there are no cases to handle, so a fully-handled variant
extraction is total at compile time with no unreachable runtime branch:

```rust
// schematically:
impl<T> FinalizeExtract<T> for Void {
    fn finalize_extract(self) -> T {
        match self {} // no cases, because Void has no values
    }
}
```

A constructible terminator like [`Nil`](nil.md) could not be discharged this way, because the compiler
would demand a value for the `Nil` case. Only an uninhabited type lets the empty match stand, which is
why the sum list terminates in `Void` and the product list in `Nil`.

## Examples

`Void` closes off every sum chain, visible when the [`Sum!`](../macros/sum.md) sugar is expanded:

```rust
use cgp::prelude::*;

// Sum![u32, bool] expands to:
type Token = Either<u32, Either<bool, Void>>;

// the empty sum is Void alone:
type Empty = Sum![]; // == Void
```

You do not construct a `Void`; you discharge one. The only thing code does with a `Void` value is match
on it with no arms, which the extractor family does when a variant match is complete.

## When to use it

**You read `Void` at the end of a sum and in an extractor's type; you do not write it.**

- **Read `Void` as "the choice ends here, and this point is unreachable."** In an expanded
  [`Either`](either.md) chain, the `Void` marks the end, and a value can never sit at it.
- **Expect `Void`, not [`Nil`](nil.md), at the end of a variant.** A record, a string, and a path end in
  `Nil`; a variant ends in `Void`. Meeting the wrong terminator in an error usually means the product and
  sum families have been crossed.
- **Recognize `Void` in a [`FinalizeExtract`](../traits/variant/finalize_extract.md) error** as the
  all-ruled-out remainder of a variant extraction.

## Common Mistakes

**`Void` is uninhabited; [`Nil`](nil.md) is not.** `Nil` is a real value the empty product is, while
`Void` is a type with no values. Records use the first, variants the second, and an error naming the
wrong one usually means the two families have been crossed.

**You cannot build a `Void`.** There is no constructor and no literal for it, by design. Any code that
appears to "return a `Void`" is really an empty match that never returns at all.

## Related constructs

- [`Either`](either.md) — the head-or-rest cell this marker terminates.
- [`Nil`](nil.md) — the product, string, and path lists' constructible end marker, the counterpart to
  this one.
- [`Sum!`](../macros/sum.md) — the macro whose empty form is `Void`.
- [`FinalizeExtract`](../traits/variant/finalize_extract.md) — discharges the uninhabited remainder of
  a variant extraction with an empty match.
- [`FinalizeExtractResult`](../traits/variant/finalize_extract_result.md) — the result-carrying form
  built on the same idea.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — where the uninhabited terminator closes a
  total variant match.

## Source

- `Void` is in
  [`sum.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/sum.rs),
  alongside [`Either`](either.md)
- `FinalizeExtract for Void`, which discharges the uninhabited remainder:
  [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
