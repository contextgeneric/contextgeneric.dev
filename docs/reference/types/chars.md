---
title: 'Chars — a type-level character list'
sidebar_label: 'Chars'
sidebar_position: 9
description: 'The character list behind Symbol!, which spells a field name out one character at a time so the name can be a type in trait resolution.'
---

# `Chars`

One character of a [type-level string](/docs/reference/glossary#type-level-string): the list that spells a field name out one character at a time so
the name can be a type.

## Overview

`Chars<const CHAR: char, Tail>` encodes a string as a type, so that a field name can take part in trait
resolution. CGP keys field access by a tag type. To read a field called `name`, something must stand for
the string `"name"` at the type level, so that the compiler can tell one
[`HasField`](../traits/field-access/has_field.md) impl from another by the tag alone. A `Chars` chain
turns the string into a type, and a `Symbol` that wraps the chain is the tag itself.

The encoding is a *list of characters* because of a limit in stable Rust. A `String` or `&str` cannot be
a const-generic parameter, but a single `char` can. So CGP spells the string out one character at a time
through a recursive `Chars` list terminated by [`Nil`](nil.md), in the same way the product list spells
out its elements through [`Cons`](cons.md). `Chars` is the specialized form of `Cons` whose head is a
`const char` rather than a type.

You write this through [`Symbol!`](../macros/symbol.md) rather than directly. `Symbol!("abc")` folds
into `Symbol<3, Chars<'a', Chars<'b', Chars<'c', Nil>>>>`. That macro's page covers the fold, the
byte-length parameter, and the rest of the wrapper. This page covers the `Chars` type underneath.

## Definition

`Chars` is a zero-sized struct carrying one character as a const parameter and the rest of the string as
its tail:

```rust
#[derive(Eq, PartialEq, Clone, Copy, Default)]
pub struct Chars<const CHAR: char, Tail>(pub PhantomData<Tail>);
```

`CHAR` is the character at this position, and `Tail` is the rest of the string, which is the next
`Chars` node or [`Nil`](nil.md) at the end. The character lives in the const parameter, and the tail
lives in a [`PhantomData<Tail>`](phantom_data.md), so a `Chars` chain does not carry runtime data
and compiles to a zero-sized value. A `Symbol<const LEN: usize, Chars>` then wraps such a chain
together with the string's byte length. The wrapper stores the length explicitly because stable Rust
cannot compute it inside a const-generic context, and the [`Symbol!`](../macros/symbol.md) page
explains why. Both `Chars` and `Symbol` are in the prelude.

## Behavior

A `Chars` chain reconstructs its original string on demand through the
[`StaticFormat`](../traits/formatting/static_format.md) trait, which formats a type-level string
into a `Formatter` without needing a value. `Chars<CHAR, Tail>` writes `CHAR` and then recurses into
the tail, and [`Nil`](nil.md) ends the recursion by writing nothing. A `Symbol` forwards to its
inner `Chars`, and the `Display` impls on both defer to `StaticFormat`, so
`<Symbol!("hello")>::default().to_string()` yields `"hello"`.

The length a `Symbol` records enables [`StaticString`](../traits/formatting/static_string.md), which
exposes the string as a `const VALUE: &'static str` rather than as a formatting routine. It decodes
the characters into a byte buffer sized by that length at compile time. A bare `Chars` chain lacks
the length, so `StaticString` is implemented for `Symbol` and for `Nil`, not for `Chars`. Code that
needs the string at run time uses `Display`, and code that needs it as a const uses `StaticString`.

## Examples

A type-level string most often appears as the tag in a
[`HasField`](../traits/field-access/has_field.md) bound, where it names the field a function or
provider reads. The same type rebuilds its string through `Display`:

```rust
use cgp::prelude::*;

// The tag names the field the function reads.
pub fn name_of<Context>(context: &Context) -> &str
where
    Context: HasField<Symbol!("name"), Value = String>,
{
    context.get_field(PhantomData::<Symbol!("name")>)
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

pub fn demo() {
    let person = Person {
        name: "Alice".to_owned(),
    };
    assert_eq!(name_of(&person), "Alice");

    // `Symbol!` spells the string out as a `Chars` chain, with its byte length in front.
    let _: PhantomData<Symbol<3, Chars<'a', Chars<'b', Chars<'c', Nil>>>>> =
        PhantomData::<Symbol!("abc")>;
    let _: PhantomData<Symbol<6, Chars<'世', Chars<'界', Nil>>>> = PhantomData::<Symbol!("世界")>;

    // `Display` walks the chain to rebuild the text.
    let symbol = <Symbol!("hello")>::default();
    assert_eq!(symbol.to_string(), "hello");
    assert_eq!(core::mem::size_of::<Symbol!("hello")>(), 0);
}
```

The two `PhantomData` annotations are the check on the expansion: `"世界"` records a length of `6`,
its UTF-8 byte count, over a chain of two `Chars` nodes. Because the encoding is a list, an empty
string is `Symbol<0, Nil>`: a `Symbol` whose character list is only the terminator.

## When to use it

**You read `Chars` in an error, and you write [`Symbol!`](../macros/symbol.md).** The macro produces the
chain. You need to know the type so that you can decode a field name in a diagnostic.

- **Decode a `Chars` chain by reading off the characters.** An error that mentions
  `HasField<Symbol<5, Chars<'w', Chars<'i', ...>>>>` says that the field `width` is involved.
- **Use [`Symbol!`](../macros/symbol.md) for a field-name tag,** and let the ergonomic constructs
  produce it from an argument or a method name where they can.
- **Use [`Index`](index_type.md) for a tuple-field position,** which encodes a number rather than a
  string.

## Common Mistakes

**A missing field appears in a raw error as a `Chars` chain.** A provider that reads `width` and
`height`, wired on a context with only `height`, fails its check:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}
```

The compiler's help line names the missing tag, and the tag the context does have, character by
character:

```text
help: the trait `HasField<Symbol<5, cgp::prelude::Chars<'w', cgp::prelude::Chars<'i', cgp::prelude::Chars<'d', cgp::prelude::Chars<'t', cgp::prelude::Chars<'h', Nil>>>>>>>` is not implemented for `Rectangle`
      but trait `HasField<Symbol<6, cgp::prelude::Chars<'h', cgp::prelude::Chars<'e', cgp::prelude::Chars<'i', cgp::prelude::Chars<'g', cgp::prelude::Chars<'h', cgp::prelude::Chars<'t', Nil>>>>>>>>` is implemented for it
```

[`cargo cgp check`](/docs/cargo-cgp/check) reads the chain back into the name, and reports
``[CGP-E106] missing field `width` on `Rectangle` `` as the root cause. `cargo cgp check` leads with
the root cause for the classes it recognizes, and the tool does not yet reshape every class.

**The `LEN` in a wrapping `Symbol` is bytes, not characters.** For ASCII the two counts agree, so the
difference shows only on a non-ASCII field name. There `LEN` does not match the visible character count,
and the `Chars` chain is shorter than `LEN`.

**`Chars` is zero-sized, so it does not hold a string at run time.** The `Display` impl rebuilds the text
from the type, and the value does not store a `&str`.

**A `Chars` chain is the character specialization of [`Cons`](cons.md), not a general list.** Its head is
a `const char`, so it cannot carry arbitrary element types the way a product list does.

## Related constructs

- [`Nil`](nil.md): the end marker that terminates a `Chars` chain.
- [`Cons`](cons.md): the general product list this one specializes, with a type head rather than a
  `const char`.
- [`Symbol!`](../macros/symbol.md): the macro that folds a string into a `Chars` chain and wraps it.
- [`StaticFormat`](../traits/formatting/static_format.md) and
  [`StaticString`](../traits/formatting/static_string.md): recover the runtime string and the const
  from the chain.
- [`HasField`](../traits/field-access/has_field.md): matches a field against its `Symbol!` tag.
- [`Field`](field.md): carries a `Symbol!` tag beside its value.
- [`Index`](index_type.md): the position tag for a tuple field, the numeric counterpart.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records): where field-name tags are used at scale.

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

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
