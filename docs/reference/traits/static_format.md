---
sidebar_label: 'StaticFormat, StaticString & ConcatPath'
---

# `StaticFormat`, `StaticString` & `ConcatPath`

Recovering type-level strings and paths back into runtime data.

## What it's for

CGP encodes field and variant names as *types* — a [`Symbol!`](../macros/symbol.md) is a length plus a character
list, one node per character — so that names can drive trait resolution. But a program eventually needs those
names as ordinary strings: to name a missing field in an error, to build a key, to render a path.

These three traits close that loop, and they divide by *when* the recovery happens.

**`StaticString`** recovers the string **eagerly**, as a compile-time `&'static str` constant computed by const
evaluation. This is the one to reach for: the decoded string is available wherever a `const` is, and it costs
nothing at run time.

**`StaticFormat`** recovers it **lazily**, by writing into a formatter. It is what backs the `Display` impl on
`Symbol` and `Chars`, so a type-level string can be printed with `{}` or `to_string()`.

**`ConcatPath`** works one level up, on paths rather than strings: a [`Path!`](../macros/path.md) is a type-level
list of segments, and joining two of them is splicing one list onto another. It is a pure type-level operation
with no runtime side at all.

## Using it

The three differ sharply in how reachable they are, and that is the first thing to know.

**`ConcatPath` is in the prelude.** `use cgp::prelude::*;` names it.

**`StaticString` is not** — import it from `cgp::core::field::traits`:

```rust
pub trait StaticString {
    const VALUE: &'static str;
}
```

`VALUE` is the decoded string as a constant. `Symbol<LEN, Chars>` computes a `[u8; LEN]` byte array at
const-evaluation time by walking the character list and UTF-8-encoding each character, then validates those bytes
as UTF-8 and exposes the result. The `LEN` on a `Symbol` is the precomputed **byte** length that sizes that array
— which is why `Symbol!` records a byte length rather than a character count.

**`StaticFormat` cannot be named through the `cgp` crate at all.** See the [Gotchas](#gotchas) below; in practice
you reach its effect through `Display` and never the trait.

```rust
pub trait StaticFormat {
    fn fmt(f: &mut Formatter<'_>) -> Result<(), fmt::Error>;
}
```

Note the absent `self`: there is no runtime value, only the type. It is implemented by recursion over the
character spine — each node writes its own character and defers to the tail, and the terminator writes nothing.

**`ConcatPath`** joins two paths:

```rust
pub trait ConcatPath<Other: ?Sized> {
    type Output: ?Sized;
}
```

Both sides may be unsized, since path types are markers. It recurses over the path spine exactly as
[`ConcatProduct`](./product_ops.md) does over a product: each node keeps its head and concatenates onto the tail,
and the terminator becomes the other path — so the result is the first path's segments followed by the second's.

## Examples

A type-level string recovers both ways, and the two are worth seeing together because the choice between them is
the page's main decision:

```rust
use cgp::core::field::traits::StaticString;
use cgp::prelude::*;

// lazily, through Display — reconstructed at the point of formatting
let s = <Symbol!("hello")>::default();
assert_eq!(s.to_string(), "hello");

// eagerly, as a compile-time constant
assert_eq!(<Symbol!("hello") as StaticString>::VALUE, "hello");
```

Both round-trip multi-byte Unicode faithfully, and the empty symbol decodes to the empty string:

```rust
assert_eq!(<Symbol!("世界你好") as StaticString>::VALUE, "世界你好");
assert_eq!(<Symbol!("") as StaticString>::VALUE, "");
```

`ConcatPath` composes two paths at the type level, which is the operation behind chaining nested accessors:

```rust
type Outer = Path!(a.b);
type Inner = Path!(c.d);

type Joined = <Outer as ConcatPath<Inner>>::Output;   // the path a.b.c.d
```

Where `StaticString` shows up in real code is recovering a name for an error. The
[optional-field layer](./optional_fields.md)'s `finalize_optional` reports its missing field by returning
`Tag::VALUE` — the field's own name, as a static string, with no allocation.

## When to reach for it, and when not

**Reach for `StaticString` when you need a name as data, and for `Display` when you need it in a message.** That
is the whole decision.

- **`StaticString`** for a constant, a key, a comparison, or anywhere the name is used more than once. It is
  computed at compile time, so there is no per-call work, and it is the only one of the three that a
  `cgp`-dependent crate can bound on.
- **`Display` / `to_string()`** for one-off formatting. Reaching for `StaticFormat` by name is not an option, and
  is not needed — the `Display` impls are what it exists to power.
- **`ConcatPath`** when composing paths in generic code, which is nested-accessor territory. If you are writing a
  path literally, [`Path!`](../macros/path.md) already gives you the whole thing and there is nothing to
  concatenate.

Two things none of these is for. They are **not a general string facility**: `Symbol!` exists to key field and
variant lookups, and building programs out of type-level strings is not what the encoding is for. And they are
**not how you read a field** — recovering a name tells you what a field is called, while
[`HasField`](./has_field.md) is what reads its value.

## Under the hood

:::note

### Advanced

This section shows the two recursions. The `StaticString` one is the more interesting, because it explains the
`LEN` parameter that otherwise looks redundant on every `Symbol`.

:::

`StaticFormat` is the straightforward one. Each character node writes itself and defers:

```rust
impl<const CHAR: char, Tail> StaticFormat for Chars<CHAR, Tail>
where
    Tail: StaticFormat,
{
    fn fmt(f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{CHAR}")?;
        Tail::fmt(f)
    }
}

impl StaticFormat for Nil { /* writes nothing */ }
```

Because both `Symbol` and `Chars` implement `Display` by delegating to this, any type-level string can be
interpolated or turned into an owned `String`.

`StaticString` is const evaluation rather than recursion at run time. It is a blanket impl over an internal
`StaticBytes` trait: `Symbol<LEN, Chars>` computes a `[u8; LEN]` in a `const` block by walking the character list
and UTF-8-encoding each character into the array, and `StaticString::VALUE` then validates those bytes as UTF-8
and exposes the `&'static str`.

**That array is why `Symbol` carries a `LEN` at all.** A const-evaluated byte array must have a known size, and the
size cannot be computed from inside the const context by walking the list — so the macro precomputes it. It is a
*byte* length, which is what makes multi-byte Unicode round-trip correctly and why `LEN` disagrees with the
character count for any non-ASCII name.

`ConcatPath` is a pure type-level computation evaluated during trait resolution, with the same two-impl shape as
[`ConcatProduct`](./product_ops.md). It never touches a value; it only names the combined path type, which a getter
then uses to descend.

## Gotchas

**`StaticFormat` cannot be named from the `cgp` crate.** It is `pub` in its own crate, but nothing re-exports it
onto a reachable path — `cgp::core` re-exports a different types crate, `cgp-base` is not a dependency of `cgp`,
and the prelude carries only its sibling `ConcatPath`. So a crate depending on `cgp` can use its *effect* through
`Display` but cannot bound on it, implement it, or name it. Treat it as an implementation detail of those `Display`
impls, and reach for `StaticString` when you want the decoded name.

**`StaticString` is not in the prelude** while `ConcatPath` is — an asymmetry between two traits that do
neighbouring jobs. Import `StaticString` from `cgp::core::field::traits`.

**`LEN` on a `Symbol` is bytes, not characters.** They coincide for ASCII, so the distinction only surfaces on a
non-ASCII name, where the number will not match the visible character count.

**`VALUE` is a constant, so it is named through the trait.** Write `<Symbol!("name") as StaticString>::VALUE`; there
is no method to call and no value to have.

**`Display` reconstructs the string rather than reading a stored one.** There is no `&str` inside a `Symbol` —
`to_string()` walks the type. For a name used repeatedly, `StaticString::VALUE` is the cheaper choice.

**`ConcatPath` produces a type, not a joined string.** It is path composition for trait resolution; if you want the
segments as text, decode the symbols.

## Related constructs

- [`Symbol!`](../macros/symbol.md) — the type-level string these decode, and where the `LEN` comes from.
- [`Path!`](../macros/path.md) — the path `ConcatPath` joins.
- [Type-level spines](../types/type_level_spines.md) — the `Chars` and path chains being walked.
- [`ConcatProduct`](./product_ops.md) — the product-level analogue of `ConcatPath`.
- [`HasField`](./has_field.md) — where the names being decoded are used as keys.
- [Optional fields](./optional_fields.md) — a real consumer, reporting a missing field by name.
- [`ChainGetters`](../providers/chain_getters.md) — nested accessors, the setting path composition serves.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — where field-name types are put to work at scale.

## Source

- [`static_string.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/static_string.rs)
  — `StaticString` and its const-evaluated UTF-8 decoding
- [`static_format.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/traits/static_format.rs)
  — `StaticFormat` and its character-list impls
- [`concat_path.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/traits/concat_path.rs)
  — `ConcatPath`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
