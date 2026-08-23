---
sidebar_label: 'StaticString'
sidebar_position: 1
---

# `StaticString`

Recovering a type-level string as a compile-time `&'static str`.

## Overview

CGP encodes field and variant names as *types* — a [`Symbol!`](../../macros/symbol.md) is a length plus a
character list, one node per character — so that names can drive trait resolution. But a program
eventually needs those names as ordinary strings: to report a missing field, to build a key, to compare
against input.

`StaticString` is how a name comes back out, and it does it **eagerly**.

**This is the one of the three recovery traits to reach for.** Its sibling
[`StaticFormat`](./static_format.md) recovers a name lazily by writing into a formatter, which backs
`Display`; reach for it only where there is no value to format and no constant will do.

## Definition

`StaticString` carries a single associated constant and no method:

```rust
pub trait StaticString {
    const VALUE: &'static str;
}
```

`VALUE` is the decoded string as a constant, computed by const evaluation rather than at run time, so the
string is available wherever a `const` is, costs nothing per use, and can be compared or stored freely.
The trait is implemented for the type-level string itself and carries no method, because there is no
value to call one on.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::StaticString;
```

`VALUE` is an associated constant rather than a method, so it is named through the trait:
`<Symbol!("name") as StaticString>::VALUE`. There is no value to have and nothing to call.

The impl is a blanket one over `Symbol<LEN, Chars>`, so every type-level string has it, including the
empty one.

## Examples

Decoding a symbol, and the eager-versus-lazy pair side by side:

```rust
use cgp::core::field::traits::StaticString;
use cgp::prelude::*;

// eagerly, as a compile-time constant
assert_eq!(<Symbol!("hello") as StaticString>::VALUE, "hello");

// lazily, through Display — reconstructed at the point of formatting
let s = <Symbol!("hello")>::default();
assert_eq!(s.to_string(), "hello");
```

Multi-byte Unicode round-trips faithfully, and the empty symbol decodes to the empty string:

```rust
assert_eq!(<Symbol!("世界你好") as StaticString>::VALUE, "世界你好");
assert_eq!(<Symbol!("") as StaticString>::VALUE, "");
```

Where it shows up in real code is recovering a name for an error. The
[optional-field layer](../optional/finalize_optional.md)'s `finalize_optional` reports its missing field by
returning `Tag::VALUE` — the field's own name, as a static string, with no allocation.

## When to use it

**Reach for `StaticString` when you need a name as data, and for `Display` when you need it in a
message.** That is the whole decision.

- **`StaticString`** for a constant, a key, a comparison, or anywhere the name is used more than once. It
  is computed at compile time, so there is no per-call work.
- **`Display` / `to_string()`** for one-off formatting, which is what
  [`StaticFormat`](./static_format.md) powers. Bounding on that trait directly is for the narrow case
  where there is no value to format.
- **[`ConcatPath`](./concat_path.md)** when the thing being composed is a path rather than a string.

Two things this is not for. It is **not a general string facility**: [`Symbol!`](../../macros/symbol.md)
exists to key field and variant lookups, and building programs out of type-level strings is not what the
encoding is for. And it is **not how you read a field** — recovering a name tells you what a field is
called, while [`HasField`](../field-access/has_field.md) reads its value.

## Under the hood

`StaticString` is const evaluation rather than a recursion at run time. It is a blanket impl over an
internal `StaticBytes` trait: `Symbol<LEN, Chars>` computes a `[u8; LEN]` in a `const` block by walking
the character list and UTF-8-encoding each character into the array, and `VALUE` then validates those
bytes as UTF-8 and exposes the `&'static str`.

**That array is why `Symbol` carries a `LEN` at all.** A const-evaluated byte array must have a known
size, and the size cannot be computed from inside the const context by walking the list — so the macro
precomputes it. It is a *byte* length, which makes multi-byte Unicode round-trip correctly and
why `LEN` disagrees with the visible character count for any non-ASCII name.

## Common Mistakes

**It is not in the prelude** while [`ConcatPath`](./concat_path.md) is — an asymmetry between two traits
that do neighbouring jobs. Import `StaticString` from `cgp::core::field::traits`.

**`VALUE` is a constant, so it is named through the trait.** Write
`<Symbol!("name") as StaticString>::VALUE`; there is no method to call.

**`LEN` on a `Symbol` is bytes, not characters.** They coincide for ASCII, so the distinction only
surfaces on a non-ASCII name, where the number will not match the visible character count.

**`Display` reconstructs the string rather than reading a stored one.** There is no `&str` inside a
`Symbol`, so `to_string()` walks the type. For a name used repeatedly, `VALUE` is the cheaper choice.

**It decodes a symbol, not a path.** A [`Path!`](../../macros/path.md) is a list of segments; decoding one
means decoding each segment's symbol.

## Related constructs

- [`StaticFormat`](./static_format.md) — the lazy counterpart, behind the `Display` impls.
- [`ConcatPath`](./concat_path.md) — the same recovery idea one level up, for paths.
- [`Symbol!`](../../macros/symbol.md) — the type-level string this decodes, and where the `LEN` comes from.
- [Type-level spines](../../types/type_level_spines.md) — the `Chars` chain being walked.
- [`HasField`](../field-access/has_field.md) — where the names being decoded are used as keys.
- [`FinalizeOptional`](../optional/finalize_optional.md) — a real consumer, reporting a missing field by name.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — where field-name types are put to work at
  scale.

## Source

- [`static_string.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/static_string.rs)
  — `StaticString` and its const-evaluated UTF-8 decoding

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
