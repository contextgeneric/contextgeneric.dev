---
sidebar_label: 'Void'
sidebar_position: 8
---

# `Void`

The uninhabited end marker of the sum list: an empty enum that can never be constructed, which lets a
complete variant match end without a runtime branch.

## Overview

`Void` marks the end of a sum. Unlike the [`Nil`](nil.md) that ends the other lists, it is uninhabited: a
value of `Void` cannot exist. A record, a string, or a path can each be empty and still exist, but an
empty *choice* cannot. A value that is "none of the branches" cannot exist, so the natural terminator for
the sum list is a type without values.

The sum list depends on this uninhabitedness. An [`Either`](either.md) chain ends in `Void`, so a
value that passed every real branch would have type `Void`, and such a value cannot exist. So the
chain is closed at its end by its definition, and a match that has handled every real variant has
nothing left to handle.

## Definition

`Void` is an empty enum:

```rust
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Void {}
```

Its variant list is empty, so it is uninhabited, like Rust's never type. It derives the standard
traits so that a sum ending in `Void` can inherit them. But code never builds a value of `Void`, so
those impls never run on one.

## Behavior

`Void` is the base case of a recursion over a sum, and it differs from the product list's base case in a
way that matters. A walk over an [`Either`](either.md) chain handles each `Left` as a branch and defers
each `Right` to the tail, until the tail is `Void`. At that point nothing is left to handle.

The extractor machinery depends on this uninhabitedness. After an extractor has tried every variant of a
sum and matched none, the leftover value has type `Void`, so it cannot exist.
[`FinalizeExtract`](../traits/variant/finalize_extract.md) for `Void` turns that value into any required
type with an empty `match self {}`, because nothing remains to handle. So a fully handled variant
extraction is complete at compile time and does not need an unreachable runtime branch:

```rust
// schematically:
impl<T> FinalizeExtract<T> for Void {
    fn finalize_extract(self) -> T {
        match self {} // no cases, because Void has no values
    }
}
```

A constructible terminator like [`Nil`](nil.md) could not be consumed this way, because the compiler would
demand an arm for the `Nil` case. Only an uninhabited type lets the empty match stand. This is why the sum
list terminates in `Void` and the product list in `Nil`.

## Examples

`Void` ends every sum chain, which the expansion of [`Sum!`](../macros/sum.md) shows:

```rust
use cgp::prelude::*;

// Sum![u32, bool] expands to:
type Token = Either<u32, Either<bool, Void>>;

// the empty sum is Void alone:
type Empty = Sum![]; // == Void
```

You do not construct a `Void`. You only eliminate one. The only thing code does with a `Void` value is
match on it with an empty `match`, and the extractor family does that when a variant match is complete.

## When to use it

**You read `Void` at the end of a sum and in an extractor's type, and you do not write it.**

- **Read `Void` as "the choice ends here, and this point is unreachable."** In an expanded
  [`Either`](either.md) chain, the `Void` marks the end, and a value can never sit at it.
- **Expect `Void`, not [`Nil`](nil.md), at the end of a variant.** A record, a string, and a path end in
  `Nil`, and a variant ends in `Void`. The wrong terminator in an error usually means the product and sum
  families have been mixed up.
- **Recognize `Void` in a [`FinalizeExtract`](../traits/variant/finalize_extract.md) error** as the
  all-ruled-out remainder of a variant extraction.

## Common Mistakes

**`Void` is uninhabited, and [`Nil`](nil.md) is not.** `Nil` is the real value of the empty product, while
`Void` is a type without values. Records use `Nil`, and variants use `Void`. An error that names the wrong
one usually means the two families have been mixed up.

**You cannot build a `Void`.** It has neither a constructor nor a literal, by design. Code that appears to
"return a `Void`" is really an empty match that never returns.

## Related constructs

- [`Either`](either.md): the head-or-rest cell this marker terminates.
- [`Nil`](nil.md): the product, string, and path lists' constructible end marker, the counterpart to
  this one.
- [`Sum!`](../macros/sum.md): the macro whose empty form is `Void`.
- [`FinalizeExtract`](../traits/variant/finalize_extract.md): eliminates the uninhabited remainder of
  a variant extraction with an empty match.
- [`FinalizeExtractResult`](../traits/variant/finalize_extract_result.md): the result-carrying form
  built on the same idea.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants): where the uninhabited terminator closes a
  total variant match.

## Source

- `Void` is in
  [`sum.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/sum.rs),
  alongside [`Either`](either.md)
- `FinalizeExtract for Void`, which eliminates the uninhabited remainder:
  [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
