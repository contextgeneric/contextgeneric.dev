---
title: 'Void — the end of a sum list'
sidebar_label: 'Void'
sidebar_position: 8
description: 'The uninhabited end marker of the sum list: an empty enum that can never be constructed, which lets a complete variant match end without a runtime branch.'
---

# `Void`

The uninhabited end marker of the sum list: an empty enum that can never be constructed, which lets a
complete variant match end without a runtime branch.

## Overview

`Void` marks the end of a sum. Unlike the [`Nil`](nil.md) that ends the other lists, it is uninhabited: a
value of `Void` cannot exist. A record, a string, or a path can each be empty and still exist, but an
empty *choice* cannot, so the natural terminator for the sum list is a type without values.

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

Its variant list is empty, so it is uninhabited, like Rust's never type `!` or
[`Infallible`](https://doc.rust-lang.org/std/convert/enum.Infallible.html). CGP defines its own so
that the terminator of a sum is named for that job. It derives the standard traits so that a sum
ending in `Void` can inherit them. But code never builds a value of `Void`, so those impls never run
on one. It is in the prelude, so `use cgp::prelude::*;` is enough.

## Behavior

`Void` is the base case of a recursion over a sum, and it differs from the product list's base case
in a way that matters. A walk over an [`Either`](either.md) chain handles each `Left` as a branch
and defers each `Right` to the tail, until the tail is `Void`. At that point nothing is left to
handle, so the base-case impl for `Void` ends the recursion, handing back what it was given or
discharging a value that cannot exist.

Variant extraction uses the same uninhabitedness in a second place. An extractor from
[`#[derive(ExtractField)]`](../derives/derive_extract_field.md) is a companion enum with one marker
per variant, and [`extract_field`](../traits/variant/extract_field.md) marks each variant it has
ruled out as `IsVoid`, whose payload type is `Void`. Once every variant is `IsVoid`, every variant
of the companion holds a `Void`, so the companion itself is uninhabited. The derive gives that
configuration a [`FinalizeExtract`](../traits/variant/finalize_extract.md) impl whose body is an
empty `match self {}`, so a fully handled extraction ends without an unreachable runtime branch. The
trait's impl for `Void` itself has the same body:

```rust
pub trait FinalizeExtract {
    fn finalize_extract<T>(self) -> T;
}

impl FinalizeExtract for Void {
    fn finalize_extract<T>(self) -> T {
        match self {}
    }
}
```

A constructible type could not be discharged this way, because the compiler would demand an arm for
its values. This is why a ruled-out variant holds `Void` rather than the `()` a missing record field
holds, and why the sum list terminates in `Void` while the product list terminates in `Nil`.

## Examples

`Void` ends every sum chain, closes a match over one, and fills the error position of a `Result`
that cannot fail:

```rust
use cgp::core::field::traits::FinalizeExtractResult;
use cgp::prelude::*;

pub type Token = Sum![u32, bool];

// Behind a reference, the `Void` arm is required.
pub fn describe(token: &Token) -> String {
    match token {
        Either::Left(number) => format!("number {number}"),
        Either::Right(Either::Left(flag)) => format!("flag {flag}"),
        Either::Right(Either::Right(void)) => match *void {},
    }
}

// By value, it may be left out.
pub fn is_number(token: Token) -> bool {
    match token {
        Either::Left(_) => true,
        Either::Right(Either::Left(_)) => false,
    }
}

pub fn demo() {
    // The annotations are the check: the chain ends in `Void`, and the empty sum is `Void`.
    let _: PhantomData<Either<u32, Either<bool, Void>>> = PhantomData::<Token>;
    let _: PhantomData<Void> = PhantomData::<Sum![]>;

    assert_eq!(describe(&Either::Right(Either::Left(true))), "flag true");
    assert!(is_number(Either::Left(3)));

    // A `Result` that cannot fail unwraps without an `unwrap`.
    let result: Result<u32, Void> = Ok(5);
    assert_eq!(result.finalize_extract_result(), 5);
}
```

You do not construct a `Void`. You only eliminate one, with an empty `match`, and
[`FinalizeExtractResult`](../traits/variant/finalize_extract_result.md) does that for the error side
of a `Result<T, Void>`.

## When to use it

**You read `Void` at the end of a sum and in an extractor's type, and you write it only to close a
match.**

- **Read `Void` as "the choice ends here, and this point is unreachable."** In an expanded
  [`Either`](either.md) chain, the `Void` marks the end, and a value can never sit at it.
- **Write `match void {}` for the last arm** of a match on a sum behind a reference, where Rust
  requires the arm.
- **Expect `Void`, not [`Nil`](nil.md), at the end of a variant.** A record, a string, and a path end in
  `Nil`, and a variant ends in `Void`. The wrong terminator in an error usually means the product and sum
  families have been mixed up.
- **Recognize `Void` in a [`FinalizeExtract`](../traits/variant/finalize_extract.md) error** as the
  payload of a variant already ruled out. An extraction that stops short leaves some variant
  `IsPresent`, and the missing `FinalizeExtract` impl is for that partly ruled-out companion.

## Common Mistakes

**`Void` is uninhabited, and [`Nil`](nil.md) is not.** `Nil` is the real value of the empty product, while
`Void` is a type without values. Records use `Nil`, and variants use `Void`. An error that names the wrong
one usually means the two families have been mixed up.

**A match behind a reference needs the `Void` arm.** A match on a sum by value may leave the last
arm out, since Rust treats an uninhabited payload as unreachable there. Through a reference it does
not:

```rust
pub type Token = Sum![u32, bool];

pub fn describe(token: &Token) -> String {
    match token {
        Either::Left(number) => format!("number {number}"),
        Either::Right(Either::Left(flag)) => format!("flag {flag}"),
    }
}
```

This fails with
``error[E0004]: non-exhaustive patterns: `&cgp::prelude::Either::Right(cgp::prelude::Either::Right(_))` not covered``,
and the compiler says why:

```text
   = note: `Void` is uninhabited but is not being matched by value, so a wildcard `_` is required
```

Add `Either::Right(Either::Right(void)) => match *void {}` to close it.

**You cannot build a `Void`.** It has neither a constructor nor a literal, by design. Code that appears to
"return a `Void`" is really an empty match that never returns.

## Related constructs

- [`Either`](either.md): the head-or-rest cell this marker terminates.
- [`Nil`](nil.md): the product, string, and path lists' constructible end marker, the counterpart to
  this one.
- [`Sum!`](../macros/sum.md): the macro whose empty form is `Void`.
- [`FinalizeExtract`](../traits/variant/finalize_extract.md): discharges an extractor whose every
  variant holds `Void`, with an empty match.
- [`FinalizeExtractResult`](../traits/variant/finalize_extract_result.md): the result-carrying form
  built on the same idea.
- [`MapType`](../traits/type-level/map_type.md): whose `IsVoid` marker maps a ruled-out variant's
  payload to `Void`.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants): where the uninhabited terminator closes a
  total variant match.

## Source

- `Void` is in
  [`sum.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/sum.rs),
  alongside [`Either`](either.md)
- `FinalizeExtract` and its impl for `Void`:
  [`extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/extract_field.rs)
- The `IsVoid` marker:
  [`map_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/map_type.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
