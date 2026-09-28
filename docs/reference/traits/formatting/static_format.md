---
title: 'StaticFormat — write a type-level string'
sidebar_label: 'StaticFormat'
sidebar_position: 2
description: 'Write a type-level string''s characters into a formatter without a value to format; the trait behind Display on Symbol and Chars.'
---

# `StaticFormat`

Writing a [type-level string](/docs/reference/glossary#type-level-string) into a formatter: the trait behind `Display` on `Symbol`.

:::info

### Generated machinery

**You are not expected to name `StaticFormat`.** It backs the `Display` impls on
`Symbol` and `Chars`, which is how a type-level string prints at all. So reach for `Display`, or for
[`StaticString`](./static_string.md) when you want a constant. This page explains the trait behind them,
and the narrow case where bounding on it is the answer.

:::

## Overview

CGP encodes field and variant names as *types*, so a name can drive trait resolution. Printing one means
turning that type back into characters, and `StaticFormat` is the trait that does it **lazily**, by
writing into a formatter rather than producing a value.

**It backs the `Display` impls on `Symbol` and `Chars`**, which is how you usually meet it: a
type-level string can be printed with `{}` or `to_string()` because of this trait.

## Definition

`StaticFormat` carries a single associated function and nothing else:

```rust
pub trait StaticFormat {
    fn fmt(f: &mut Formatter<'_>) -> Result<(), fmt::Error>;
}
```

Note the absent `self`: the string exists only as a type, so `fmt` takes the formatter alone and
writes the type's characters into it. The trait is implemented for the type-level string itself, by
recursion over the character list, and the terminator writes nothing.

## Usage

**It is not in the prelude.** Import it from `cgp::core::base::traits`:

```rust
use cgp::core::base::traits::StaticFormat;
```

That module is where the base type-level crate is re-exported, and it is a different home from its
neighbours: [`StaticString`](./static_string.md) comes from `cgp::core::field::traits`, and
[`ConcatPath`](./concat_path.md), which lives in the same crate as this trait, is in the prelude.
Three neighbouring traits, three different imports.

Most of the time you reach the effect rather than the trait. Any type-level string can be
interpolated or turned into an owned `String` without any import:

```rust
use cgp::prelude::*;

let s = <Symbol!("hello")>::default();

assert_eq!(s.to_string(), "hello");
assert_eq!(format!("field: {s}"), "field: hello");
```

The impls exist for `Chars`, the character list, and for `Nil`, which terminates it, with `Symbol`
delegating to its inner list. Every type-level string therefore formats, including the empty one.

## Examples

Formatting through `Display`, a `Display` bound in generic code, and the one case for the trait
itself:

```rust
use core::fmt::{self, Display, Formatter};
use cgp::prelude::*;
use cgp::core::base::traits::StaticFormat;

// A `Display` bound is all that formatting a name needs.
pub fn describe<Tag: Default + Display>(_tag: PhantomData<Tag>) -> String {
    format!("missing field `{}`", Tag::default())
}

// A wrapper holds only `PhantomData`, so it formats its tag through the trait.
pub struct FieldName<Tag>(pub PhantomData<Tag>);

impl<Tag: StaticFormat> Display for FieldName<Tag> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Tag::fmt(f)
    }
}

pub fn demo() {
    let s = <Symbol!("hello")>::default();
    assert_eq!(s.to_string(), "hello");
    assert_eq!(format!("field: {s}"), "field: hello");

    assert_eq!(
        describe(PhantomData::<Symbol!("height")>),
        "missing field `height`"
    );

    let name = FieldName(PhantomData::<Symbol!("width")>);
    assert_eq!(format!("[{name}]"), "[width]");
}
```

`describe` needs nothing imported: a `Display` bound is all that formatting a name takes.
`FieldName` is the narrow case the trait exists for. It holds only a `PhantomData`, so there is no
symbol value to call `Display` on, and its own `Display` impl writes the tag's characters through
`Tag::fmt`. For a name used more than once, [`StaticString`](./static_string.md)'s constant is
cheaper.

## When to use it

**Reach for `Display` for one-off formatting, [`StaticString`](./static_string.md) for a name you use
more than once, and this trait only when you need to write characters without a value to hand.**

- **`Display` / `to_string()`** when the name goes straight into a message. This is what `StaticFormat`
  exists to power, and it is the form to prefer.
- **[`StaticString`](./static_string.md)** for a constant, a key, or a comparison. It is computed at
  compile time, so it costs nothing per call.
- **`StaticFormat`** when a formatter has to be written into from a type without a value, such as
  implementing `Display` for a wrapper over a type-level string. This is the narrow case, and it is
  why the method does not take `self`.
- **[`ConcatPath`](./concat_path.md)** when the thing being composed is a path rather than a string.

## Under the hood

Each character node writes itself and defers to the tail, and the terminator writes nothing:

```rust
impl<const CHAR: char, Tail> StaticFormat for Chars<CHAR, Tail>
where
    Tail: StaticFormat,
{
    fn fmt(f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{CHAR}")?;
        Tail::fmt(f)
    }
}

impl StaticFormat for Nil {
    fn fmt(_f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}
```

`Symbol` implements it by delegating to its inner character list, and both `Symbol` and `Chars`
implement `Display` by calling it:

```rust
impl<const LEN: usize, Chars> Display for Symbol<LEN, Chars>
where
    Self: StaticFormat,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <Self as StaticFormat>::fmt(f)
    }
}
```

So any type-level string can be interpolated or turned into an owned `String`, and the string is
*reconstructed* on each call rather than read out of storage, since a `Symbol` does not store a
`&str` to read. That per-call reconstruction is the difference from
[`StaticString`](./static_string.md), which does the same decoding once, at compile time, into a
`&'static str` constant.

## Common Mistakes

**It is not in the prelude, and its import differs from both its neighbours'.** This trait comes from
`cgp::core::base::traits`, [`StaticString`](./static_string.md) from `cgp::core::field::traits`, and
[`ConcatPath`](./concat_path.md), defined in the same crate as this one, is in the prelude. Reaching for
the wrong module is the usual first failure.

**Prefer a `Display` bound where one will do.** Any code that only needs to *format* a type-level
string should require `Display`, which needs nothing imported and is what the trait produces.

**`Display` reconstructs the string on every call.** For a name used repeatedly,
[`StaticString`](./static_string.md)'s `VALUE` is the cheaper choice.

**The method does not take `self`.** It is an associated function, because a type-level string
lacks a value. Formatting through `Display` therefore needs a value, which for a symbol is
`<Symbol!("name")>::default()`, and code holding only the type bounds on this trait instead.

## Related constructs

- [`StaticString`](./static_string.md): the eager counterpart, and the one to reach for.
- [`ConcatPath`](./concat_path.md): path composition, its reachable sibling in the same group.
- [`Symbol!`](../../macros/symbol.md): the type-level string being formatted.
- [Type-level lists](../../types/index.md): the `Chars` chain being walked.
- [`HasField`](../field-access/has_field.md): where the names being decoded are used as keys.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): where field-name types are put to work at
  scale.

## Source

- [`static_format.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/traits/static_format.rs):
  `StaticFormat` and its character-list impls

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
