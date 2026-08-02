---
sidebar_label: 'StaticFormat'
---

# `StaticFormat`

Writing a type-level string into a formatter — the trait behind `Display` on `Symbol`.

:::info

### Generated machinery

**You are not expected to name `StaticFormat`.** It backs the `Display` impls on
`Symbol` and `Chars`, which is how a type-level string prints at all — so reach for `Display`, or for
[`StaticString`](./static_string.md) when you want a constant. This page explains the trait behind them,
and the narrow case where bounding on it is the answer.

:::

## What it's for

CGP encodes field and variant names as *types*, so a name can drive trait resolution. Printing one means
turning that type back into characters, and `StaticFormat` is the trait that does it **lazily**, by
writing into a formatter rather than producing a value:

```rust
pub trait StaticFormat {
    fn fmt(f: &mut Formatter<'_>) -> Result<(), fmt::Error>;
}
```

Note the absent `self`: there is no runtime value, only the type. It is implemented by recursion over the
character spine — each node writes its own character and defers to the tail, and the terminator writes
nothing.

**It is what backs the `Display` impls on `Symbol` and `Chars`**, which is how you usually meet it: a
type-level string can be printed with `{}` or `to_string()` because of this trait.

## Using it

**It is not in the prelude.** Import it from `cgp::core::base::traits`:

```rust
use cgp::core::base::traits::StaticFormat;
```

That module is where the base type-level crate is re-exported, and it is a different home from its
neighbours: [`StaticString`](./static_string.md) comes from `cgp::core::field::traits`, and
[`ConcatPath`](./concat_path.md) — which lives in the same crate as this trait — is in the prelude.
Three neighbouring traits, three different imports.

Most of the time you reach the effect rather than the trait. Any type-level string can be interpolated
or turned into an owned `String` with no import at all:

```rust
use cgp::prelude::*;

let s = <Symbol!("hello")>::default();

assert_eq!(s.to_string(), "hello");
assert_eq!(format!("field: {s}"), "field: hello");
```

The impls exist for `Chars` — the character spine — and for `Nil`, which terminates it, with `Symbol`
delegating to its inner list. Every type-level string therefore formats, including the empty one.

## Examples

Formatting is what a diagnostic or a log line wants, where the name appears once. A `Display` bound is
usually enough, and needs no import:

```rust
use cgp::prelude::*;

fn describe<Tag: Default + core::fmt::Display>(_tag: core::marker::PhantomData<Tag>) -> String {
    format!("missing field `{}`", Tag::default())
}

let message = describe(core::marker::PhantomData::<Symbol!("height")>);

assert_eq!(message, "missing field `height`");
```

Bounding on the trait itself is what you reach for when there is no *value* to call `Display` on — the
method is an associated function, so it writes a type's characters without one:

```rust
use cgp::core::base::traits::StaticFormat;

fn write_name<Tag: StaticFormat>(f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    Tag::fmt(f)
}
```

For a name used more than once, the eager form is cheaper and is bounded differently again:

```rust
use cgp::core::field::traits::StaticString;

assert_eq!(<Symbol!("height") as StaticString>::VALUE, "height");
```

## When to reach for it, and when not

**Reach for `Display` for one-off formatting, [`StaticString`](./static_string.md) for a name you use
more than once, and this trait only when you need to write characters without a value to hand.**

- **`Display` / `to_string()`** when the name goes straight into a message. This is what `StaticFormat`
  exists to power, and it is the form to prefer.
- **[`StaticString`](./static_string.md)** for a constant, a key, or a comparison. It is computed at
  compile time, so there is no per-call work.
- **`StaticFormat`** when a formatter has to be written into from a type with no value — implementing
  `Display` for a wrapper over a type-level string, say. This is the narrow case, and it is why the
  method takes no `self`.
- **[`ConcatPath`](./concat_path.md)** when the thing being composed is a path rather than a string.

## Under the hood

:::note

### Advanced

This section shows the recursion, which is the shorter of the two string recoveries.

:::

Each character node writes itself and defers to the tail:

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
interpolated or turned into an owned `String` — and the string is *reconstructed* on each call rather
than read out of storage, since there is no `&str` inside a `Symbol` to read.

That per-call reconstruction is the difference from [`StaticString`](./static_string.md), which does the
same decoding once, at compile time, into a `&'static str` constant.

## Gotchas

**It is not in the prelude, and its import differs from both its neighbours'.** This trait comes from
`cgp::core::base::traits`, [`StaticString`](./static_string.md) from `cgp::core::field::traits`, and
[`ConcatPath`](./concat_path.md) — defined in the same crate as this one — is in the prelude. Reaching for
the wrong module is the usual first failure.

**Prefer a `Display` bound where one will do.** Any code that only needs to *format* a type-level string
should require `Display`, which needs no import and is what the trait produces.

**`Display` reconstructs the string on every call.** For a name used repeatedly,
[`StaticString`](./static_string.md)'s `VALUE` is the cheaper choice.

**There is no `self`.** The trait's method is an associated function, because a type-level string has no
value — which is why the `Display` impl goes through a `Default`-constructed marker, and why a bound on
this trait is what you need when there is no value to construct.

## Related constructs

- [`StaticString`](./static_string.md) — the eager counterpart, and the one to reach for.
- [`ConcatPath`](./concat_path.md) — path composition, its reachable sibling in the same group.
- [`Symbol!`](../macros/symbol.md) — the type-level string being formatted.
- [Type-level spines](../types/type_level_spines.md) — the `Chars` chain being walked.
- [`HasField`](./has_field.md) — where the names being decoded are used as keys.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — where field-name types are put to work at
  scale.

## Source

- [`static_format.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/traits/static_format.rs)
  — `StaticFormat` and its character-list impls

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
