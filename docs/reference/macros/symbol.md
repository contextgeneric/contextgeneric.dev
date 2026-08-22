---
sidebar_label: 'Symbol!'
---

# `Symbol!`

A type-level string, used as a field-name tag.

## Overview

CGP needs field *names* to be types. Reading a field goes through
[`HasField<Tag>`](../traits/has_field.md), where `Tag` identifies which field is meant — so to look up a field
called `name`, something has to stand in for the string `"name"` at the type level.

`Symbol!("name")` is that something. It produces a distinct type whose whole identity is the characters it
encodes: two invocations with the same string are the same type, and two with different strings are different
types.

```rust
HasField<Symbol!("name"), Value = String>
```

Encoding a string as a type is what lets field access take part in trait resolution. A **context** — the type
the capability runs against, which supplies the values it needs as its fields — can carry a
`HasField<Symbol!("width")>` impl and a `HasField<Symbol!("height")>` impl side by side, and the compiler
picks the right one from the tag alone. Nothing is compared at runtime, because there is nothing at runtime:
the tag exists only during compilation.

**You rarely write it by hand.** [`#[implicit]`](../attributes/implicit.md) arguments,
[`#[cgp_auto_getter]`](./cgp_auto_getter.md), and [`#[derive(HasField)]`](../derives/derive_has_field.md) all
generate the tag from a name you already wrote. Where `Symbol!` shows up explicitly is in a wiring entry that
names a field — `UseField<Symbol!("first_name")>` — and where it shows up unavoidably is in compiler errors,
which is the main reason to be able to read it.

## Using it

The macro takes a single string literal and is used wherever a type is expected — in trait bounds, in
associated-type positions, and inside a `PhantomData` tag:

```rust
Symbol!("name")
Symbol!("first_name")
Symbol!("")
```

Any valid string literal is accepted, including the empty string and multi-byte Unicode
(`Symbol!("世界")`). Most often it appears inside a field bound:

```rust
Self: HasField<Symbol!("name"), Value = String>
```

### The tag for a tuple field

A tuple-struct field has no name, so it cannot be keyed by a string.
[`#[derive(HasField)]`](../derives/derive_has_field.md) tags those with [`Index`](../types/index.md) instead —
`Index<0>`, `Index<1>` — which encodes a number at the type level the way `Symbol!` encodes a string. A field
is keyed by `Symbol!` when it has a name and by `Index` when it has only a position.

### Raw identifiers

A field written as a raw identifier is tagged by its *logical* name, with the `r#` stripped: a field `r#type`
is tagged `Symbol!("type")`. The macro itself performs no stripping — it takes the literal verbatim — so
`Symbol!("type")` is the tag that matches, and `Symbol!("r#type")` would be a different tag matching nothing.

## Examples

The everyday appearance is a wiring entry that points a getter at a particular field:

```rust
use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct Person {
    pub first_name: String,
}

delegate_components! {
    Person {
        NameGetterComponent: UseField<Symbol!("first_name")>,
    }
}
```

`Symbol!("first_name")` is what tells [`UseField`](../providers/use_field.md) which field to read. This is the
one place a reader routinely writes the macro themselves.

It also appears in a hand-written field bound, which is what the ergonomic constructs generate for you:

```rust
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

Written idiomatically that provider would use an [`#[implicit]`](../attributes/implicit.md) argument and no
`Symbol!` would be visible at all — which is the point of the ergonomic surface, and why this form is worth
recognizing rather than writing.

A type-level string can also be turned back into a runtime string, which is occasionally useful for
diagnostics:

```rust
let s = <Symbol!("hello")>::default();
assert_eq!(s.to_string(), "hello");
```

## When to reach for it, and when not

**Write `Symbol!` when a wiring entry has to name a field**, and let the macros produce it everywhere else.
That is the honest summary: the construct is load-bearing and mostly generated.

- **Use an [`#[implicit]`](../attributes/implicit.md) argument to read a field.** The parameter's name becomes
  the tag, so you never type one.
- **Use [`#[cgp_auto_getter]`](./cgp_auto_getter.md) for a named accessor.** The method name becomes the tag.
- **Write `Symbol!` explicitly with [`UseField`](../providers/use_field.md)**, where the whole point is that
  the field name is a wiring decision rather than fixed to a method name. This is its main hand-written use.
- **Use [`Index`](../types/index.md) for a tuple field**, not a `Symbol!` of `"0"`. They are different types
  and the derive generates the former.

Two things it is not. It is **not a runtime string** — there is no `&str` inside it, and the `Display` impl
above reconstructs the text from the type rather than reading a stored value. And it is **not a general
type-level string facility** to build programs out of; it exists to key field and variant lookups, and the
[`StaticFormat`](../traits/static_format.md) traits are what recover text from one when that is genuinely
needed.

## Under the hood

:::note

### Advanced

This section shows what the macro expands to. You do not need it to use `Symbol!`, but the expanded form is
exactly what a compiler error prints, so reading it once turns an intimidating error into a legible one.
`cargo cgp expand` resugars it back for your own code.

:::

`Symbol!("...")` expands to a `Symbol` type wrapping a `Chars` chain that spells the string one character at a
time:

```rust
// before
Symbol!("abc")

// after
Symbol<3, Chars<'a', Chars<'b', Chars<'c', Nil>>>>
```

Two type constructors do the work. `Chars<const CHAR: char, Tail>` is one character paired with the rest of
the string; chained through its tail and terminated by `Nil`, it forms a
[type-level list](../types/type_level_spines.md) of characters — the same shape as
[`Product!`](./product.md)'s `Cons`/`Nil` spine, specialized so the head is a `const char` rather than a type.
`Symbol<const LEN: usize, Chars>` then wraps that list together with a length.

**The `LEN` parameter is the part most likely to surprise**, and it exists to work around a limit in stable
Rust: the length of a `Chars` chain cannot be computed inside a const-generic context, so the macro
precomputes it and bakes it in. The value is the string's **byte** length, not its character count:

```rust
Symbol!("abc")       // Symbol<3,  Chars<'a', ...>>   — 3 bytes,  3 chars
Symbol!("世界你好")   // Symbol<12, Chars<'世', ...>>  — 12 bytes, 4 chars
```

The character list has one `Chars` node per Unicode scalar value, so those two numbers disagree for any
non-ASCII string. `LEN` is there so length-dependent code can read the size off the type instead of recursing
through the list.

The expansion is built by folding the characters right to left onto `Nil` and wrapping the result, so the empty
string `Symbol!("")` becomes `Symbol<0, Nil>`.

**What this means for reading errors** is the practical payoff. A missing field on a context is reported
against the expanded tag, so an error mentioning
`HasField<Symbol<5, Chars<'w', Chars<'i', ...>>>>` is telling you the field `width` is missing. Counting the
characters is enough to decode it, and `cargo cgp check` resugars the common cases.

<details>
<summary>Formal grammar</summary>

The input is a single string literal, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
SymbolInput -> STRING_LITERAL
```

`STRING_LITERAL` is the Rust string-literal token, so any valid string literal is accepted — including the
empty string and multi-byte Unicode. The macro is used in type position, and this single literal is the whole
of its input.

</details>

## Gotchas

**The `LEN` in an expanded `Symbol` is bytes, not characters.** For ASCII the two coincide, which is why the
distinction only shows up on non-ASCII field names — where the number will not match the visible character
count and the `Chars` chain will be shorter than `LEN`.

**A raw-identifier field is tagged without the `r#`.** `Symbol!("r#type")` and `Symbol!("type")` are different
types, and the derive generates the second, so the first matches nothing.

**Two spellings of the same field are different tags.** `Symbol!("first_name")` and `Symbol!("firstName")` are
unrelated types, and a mismatch reports as a missing `HasField` bound rather than as a typo. This is the usual
cause of a getter that "should" work.

**It is a type, so it goes in type position.** Writing `Symbol!("name")` where a value is expected does not
work; the `PhantomData::<Symbol!("name")>` form is how the tag is passed to `get_field`, and the tag is the
type argument rather than the value.

## Related constructs

- [`Index`](../types/index.md) — the position-keyed tag for tuple-struct fields.
- [`HasField`](../traits/has_field.md) — what a tag is looked up through.
- [`#[derive(HasField)]`](../derives/derive_has_field.md) — generates one tag per named field.
- [`#[implicit]`](../attributes/implicit.md) — generates the tag from an argument name.
- [`#[cgp_auto_getter]`](./cgp_auto_getter.md) — generates it from a method name.
- [`UseField`](../providers/use_field.md) — where the tag is written by hand, as a wiring decision.
- [Type-level spines](../types/type_level_spines.md) — the `Chars`/`Nil` chain the expansion builds.
- [`Product!`](./product.md) and [`Sum!`](./sum.md) — the record and variant lists whose entries carry these
  tags.
- [`StaticFormat`](../traits/static_format.md) — recovering runtime text from a type-level string.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — where field-name tags are used at scale.

## Source

- Entry point: [`symbol.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/symbol.rs)
- The character fold and `LEN` computation: [`types/field/symbol.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/field/symbol.rs)
- Runtime types: [`types/symbol.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/symbol.rs)
  and [`types/chars.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/chars.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
