---
title: 'StaticString — a symbol as a &static str'
sidebar_label: 'StaticString'
sidebar_position: 1
description: 'Decode a Symbol into a compile-time &''static str constant, the eager way to recover a field or variant name as data.'
---

# `StaticString`

Recovering a [type-level string](/docs/reference/glossary#type-level-string) as a compile-time `&'static str`.

## Overview

CGP encodes field and variant names as *types* (a [`Symbol!`](../../macros/symbol.md) is a length plus a
character list, one node per character) so that names can drive trait resolution. But a program
eventually needs those names as ordinary strings: to report a missing field, to build a key, to compare
against input.

`StaticString` is how a name comes back out, and it does it **eagerly**.

**This is the one of the three recovery traits to reach for.** Its sibling
[`StaticFormat`](./static_format.md) recovers a name lazily by writing into a formatter, which backs
`Display`; reach for it only where a value to format is missing and a constant will not do.

## Definition

`StaticString` carries a single associated constant and nothing else:

```rust
pub trait StaticString {
    const VALUE: &'static str;
}
```

`VALUE` is the decoded string as a constant, computed by const evaluation rather than at run time,
so the string is available wherever a `const` is, costs nothing per use, and can be compared or
stored freely. The trait is implemented for the type-level string itself and lacks a method, because
there is nothing to call one on.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::StaticString;
```

`VALUE` is an associated constant rather than a method, so it is named through the trait:
`<Symbol!("name") as StaticString>::VALUE`. Nothing needs a value or a call.

Every [`Symbol!`](../../macros/symbol.md) has it, including the empty one, and so does `Nil`, whose
value is `""`. A bare [`Chars`](../../types/chars.md) list does not, because the impl needs the byte
length only `Symbol` carries; [Common Mistakes](#common-mistakes) shows the error.

## Examples

Decoding a symbol eagerly and lazily, multi-byte and empty symbols, and a generic routine naming its
tag:

```rust
use cgp::prelude::*;
use cgp::core::field::traits::StaticString;

// A generic routine recovers the name of whatever tag it is given.
pub fn field_name<Tag: StaticString>(_tag: PhantomData<Tag>) -> &'static str {
    Tag::VALUE
}

pub fn demo() {
    // eagerly, as a compile-time constant
    assert_eq!(<Symbol!("hello") as StaticString>::VALUE, "hello");

    // lazily, through `Display`: reconstructed at the point of formatting
    let s = <Symbol!("hello")>::default();
    assert_eq!(s.to_string(), "hello");

    assert_eq!(<Symbol!("世界你好") as StaticString>::VALUE, "世界你好");
    assert_eq!(<Symbol!("") as StaticString>::VALUE, "");

    assert_eq!(field_name(PhantomData::<Symbol!("height")>), "height");
}
```

Multi-byte Unicode round-trips faithfully and the empty symbol decodes to the empty string.
`field_name` is where the trait shows up in real code: the [optional-field
layer](../optional/finalize_optional.md)'s `finalize_optional` reports a missing field by returning
`Tag::VALUE`, the field's own name as a static string, with no allocation.

## When to use it

**Reach for `StaticString` when you need a name as data, and for `Display` when you need it in a
message.** That is the whole decision.

- **`StaticString`** for a constant, a key, a comparison, or anywhere the name is used more than
  once. It is computed at compile time, so it costs nothing per call.
- **`Display` / `to_string()`** for one-off formatting, which is what
  [`StaticFormat`](./static_format.md) powers. Bounding on that trait directly is for the narrow
  case where a value to format is missing.
- **[`ConcatPath`](./concat_path.md)** when the thing being composed is a path rather than a string.

Two things this is not for. It is **not a general string facility**: [`Symbol!`](../../macros/symbol.md)
exists to key field and variant lookups, and building programs out of type-level strings is not what the
encoding is for. And it is **not how you read a field**: recovering a name tells you what a field is
called, while [`HasField`](../field-access/has_field.md) reads its value.

## Under the hood

`StaticString` is const evaluation rather than a recursion at run time. It is a blanket impl over a
private `StaticBytes` trait, which only `Symbol<LEN, Chars>` and `Nil` implement:

```rust
impl<T> StaticString for T
where
    T: StaticBytes,
{
    const VALUE: &'static str = const {
        match str::from_utf8(T::BYTES) {
            Ok(value) => value,
            Err(_) => panic!("error const decoding &[u8] to &str"),
        }
    };
}
```

`Symbol<LEN, Chars>`'s `BYTES` is a `[u8; LEN]` computed in a `const fn` by walking the character
list and UTF-8-encoding each character into the array, and `VALUE` then validates those bytes as
UTF-8 and exposes the `&'static str`.

**That array is why `Symbol` carries a `LEN` at all.** A const-evaluated array's length must be a
constant known from the type, and stable Rust cannot compute it from the character list inside a
generic, so the macro precomputes it. It is a *byte* length, which makes multi-byte Unicode
round-trip correctly and makes `LEN` disagree with the visible character count for any non-ASCII
name.

## Common Mistakes

**It is not in the prelude** while [`ConcatPath`](./concat_path.md) is, an asymmetry between two traits
that do neighbouring jobs. Import `StaticString` from `cgp::core::field::traits`.

**`VALUE` is a constant, so it is named through the trait.** Write
`<Symbol!("name") as StaticString>::VALUE`; the trait lacks a method.

**`LEN` on a `Symbol` is bytes, not characters.** They coincide for ASCII, so the distinction only
surfaces on a non-ASCII name, where the number will not match the visible character count.

**`Display` reconstructs the string rather than reading a stored one.** A `Symbol` does not store a
`&str`, so `to_string()` walks the type. For a name used repeatedly, `VALUE` is the cheaper choice.

**It decodes a symbol, not a path or a bare character list.** A [`Path!`](../../macros/path.md) is a
list of segments; decoding one means decoding each segment's symbol. And a `Chars` list on its own,
without the `Symbol` wrapper that carries its byte length, lacks the impl:

```rust
pub const NAME: &str = <Chars<'a', Nil> as StaticString>::VALUE;
```

```text
error[E0277]: the trait bound `cgp::prelude::Chars<'a', cgp::prelude::Nil>: StaticString` is not satisfied
...
   = help: the trait `cgp::cgp_core::cgp_field::traits::static_string::StaticBytes` is not implemented for `cgp::prelude::Chars<'a', cgp::prelude::Nil>`
```

## Related constructs

- [`StaticFormat`](./static_format.md): the lazy counterpart, behind the `Display` impls.
- [`ConcatPath`](./concat_path.md): the same recovery idea one level up, for paths.
- [`Symbol!`](../../macros/symbol.md): the type-level string this decodes, and where the `LEN` comes from.
- [Type-level lists](../../types/index.md): the `Chars` chain being walked.
- [`HasField`](../field-access/has_field.md): where the names being decoded are used as keys.
- [`FinalizeOptional`](../optional/finalize_optional.md): a real consumer, reporting a missing field by name.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): where field-name types are put to work at
  scale.

## Source

- [`static_string.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/static_string.rs):
  `StaticString` and its const-evaluated UTF-8 decoding

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
