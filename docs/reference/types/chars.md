---
sidebar_label: 'Chars'
sidebar_position: 9
---

# `Chars`

One character of a type-level string: the list that spells a field name out one character at a time so
the name can be a type.

:::info

### Generated machinery

**You are not expected to write `Chars` by hand.** The [`Symbol!`](../macros/symbol.md) macro folds a
string literal into a `Chars` chain, and a `Symbol` wraps that chain. You meet `Chars` in a
missing-field error, where a field name prints as its expanded character list, and this page explains how
to read it.

:::

## Overview

`Chars<const CHAR: char, Tail>` encodes a string as a type, so that a field name can take part in trait
resolution. CGP keys field access by a tag type: to read a field called `name`, something must stand in
for the string `"name"` at the type level, so the compiler can match one
[`HasField`](../traits/field-access/has_field.md) impl against another purely from the tag. A `Chars`
chain is how the string becomes a type, and a `Symbol` wrapping that chain is the tag itself.

The reason the encoding is a *list of characters* is a limit in stable Rust: a `String` or `&str` cannot
be a const-generic parameter, but a single `char` can. So CGP spells the string out one character at a
time through a recursive `Chars` list, terminated by [`Nil`](nil.md), the same way the product list
spells out its elements through [`Cons`](cons.md). `Chars` is the specialized form of `Cons` in which the
head is a `const char` rather than a type.

You write this through [`Symbol!`](../macros/symbol.md) rather than by hand. `Symbol!("abc")` folds
into `Symbol<3, Chars<'a', Chars<'b', Chars<'c', Nil>>>>`, and that macro's page carries the fold, the
byte-length parameter, and the rest of the wrapper. This page is the runtime `Chars` type underneath.

## Definition

`Chars` is a zero-sized struct carrying one character as a const parameter and the rest of the string as
its tail:

```rust
pub struct Chars<const CHAR: char, Tail>(pub PhantomData<Tail>);
```

`CHAR` is the character at this position, and `Tail` is the rest of the string, expected to be either the
next `Chars` node or [`Nil`](nil.md) at the end. The character lives in the const parameter and the tail
in a [`PhantomData<Tail>`](phantom_data.md), so a `Chars` chain carries no runtime data and is erased
to a zero-sized value. A `Symbol<const LEN: usize, Chars>` then wraps such a chain together with the
string's byte length; the length is stored explicitly because stable Rust cannot compute it inside a
const-generic context, and [`Symbol!`](../macros/symbol.md) covers why.

## Behavior

A `Chars` chain reconstructs its original string on demand through the
[`StaticFormat`](../traits/formatting/static_format.md) trait, which formats a type-level string into a
`Formatter` without needing a value. `Chars<CHAR, Tail>` writes `CHAR` and then recurses into the tail,
and [`Nil`](nil.md) ends the recursion by writing nothing. A `Symbol` forwards to its inner `Chars`, and
its `Display` impl defers to `StaticFormat`, so `<Symbol!("hello")>::default().to_string()` yields
`"hello"`.

The length a `Symbol` records enables [`StaticString`](../traits/formatting/static_string.md), which
exposes the string as a `const VALUE: &'static str` rather than a formatting routine, by decoding the
characters into a byte buffer sized by that length at compile time. Code that needs the string at run
time uses `Display`; code that needs it as a const uses `StaticString`.

## Examples

A type-level string most often appears as the tag in a
[`HasField`](../traits/field-access/has_field.md) bound, where it names the field a provider reads:

```rust
use cgp::prelude::*;

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn greet(&self) {
        println!("Hello, {}!", self.get_field(PhantomData::<Symbol!("name")>));
    }
}
```

The same type can be built and inspected at run time through its `Display` impl, which walks the `Chars`
chain to rebuild the string:

```rust
use cgp::prelude::*;

let s = <Symbol!("hello")>::default();
assert_eq!(s.to_string(), "hello");
```

Because the encoding is a list, an empty string is `Symbol<0, Nil>`: a `Symbol` whose character list is
just the terminator and whose recorded length is zero.

## When to use it

**You read `Chars` in an error; you write [`Symbol!`](../macros/symbol.md).** The macro produces the
chain, and the reason to know the type is to decode a field name in a diagnostic.

- **Decode a `Chars` chain by reading off the characters.** An error mentioning
  `HasField<Symbol<5, Chars<'w', Chars<'i', ...>>>>` is telling you the field `width` is missing, and
  `cargo cgp check` resugars the common cases.
- **Use [`Symbol!`](../macros/symbol.md) for a field-name tag,** and let the ergonomic constructs
  produce it from an argument or a method name where they can.
- **Use [`Index`](index_type.md) for a tuple-field position,** which encodes a number rather than a
  string.

## Common Mistakes

**The `LEN` in a wrapping `Symbol` is bytes, not characters.** For ASCII the two coincide, so the
distinction only shows on a non-ASCII field name, where `LEN` will not match the visible character count
and the `Chars` chain will be shorter than `LEN`.

**`Chars` is zero-sized, so it holds no string at run time.** The `Display` impl rebuilds the text from
the type; there is no stored `&str` inside it.

**A `Chars` chain is the character specialization of [`Cons`](cons.md), not a general list.** Its head is
a `const char`, so it cannot carry arbitrary element types the way a product list does.

## Related constructs

- [`Nil`](nil.md) — the end marker that terminates a `Chars` chain.
- [`Cons`](cons.md) — the general product list this one specializes, with a type head rather than a
  `const char`.
- [`Symbol!`](../macros/symbol.md) — the macro that folds a string into a `Chars` chain and wraps it.
- [`StaticFormat`](../traits/formatting/static_format.md) and
  [`StaticString`](../traits/formatting/static_string.md) — recover the runtime string and the const
  from the chain.
- [`HasField`](../traits/field-access/has_field.md) — matches a field against its `Symbol!` tag.
- [`Field`](field.md) — carries a `Symbol!` tag beside its value.
- [`Index`](index_type.md) — the position tag for a tuple field, the numeric counterpart.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — where field-name tags are used at scale.

## Source

- The type:
  [`chars.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/chars.rs),
  with the `Symbol` wrapper in
  [`symbol.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/symbol.rs)
  and [`Nil`](nil.md) in
  [`nil.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/nil.rs)
- The `StaticFormat` impls behind `Display`:
  [`static_format.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/traits/static_format.rs),
  and the const-decoding `StaticString`:
  [`static_string.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/static_string.rs)
- The constructing macro is [`Symbol!`](../macros/symbol.md).

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
