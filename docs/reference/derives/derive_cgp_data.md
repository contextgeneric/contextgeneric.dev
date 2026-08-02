---
sidebar_label: '#[derive(CgpData)]'
---

# `#[derive(CgpData)]`

The umbrella extensible-data derive, for a struct or an enum.

## What it's for

A plain Rust struct or enum is opaque to generic code. There is no way to refer to "the `first_name`
field" or "the `Circle` variant" through a type parameter, so anything that must work across several
data types ends up written once per type.

`#[derive(CgpData)]` is the one-line answer: it turns a struct or an enum into **extensible data**, a
type whose fields or variants generic code can name, read, construct, and take apart without ever
mentioning the concrete type. It is the umbrella derive, and what it generates depends entirely on what
it is applied to — on a struct it emits the record machinery, on an enum the variant machinery.

The two things this buys are worth naming separately, because together they are what "extensible" means
here. The **representation** view describes the type as a list of named entries, which is what lets one
implementation serialize any record or dispatch over any enum. The **incremental** view adds a companion
type that tracks, in its own type parameters, which fields are present or which variants are still
possible — so a half-built value and a finished one are *different types*, and using one where the other
belongs is a compile error rather than a runtime panic.

## Using it

The derive takes no arguments and has no helper attributes, and it accepts either shape:

```rust
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

Naming follows the family's rules: a named struct field or a variant is keyed by
[`Symbol!`](../macros/symbol.md), and a positional field of a tuple struct by
[`Index<N>`](../types/index.md). Generic parameters, lifetimes, and a `where` clause are carried onto
everything generated, including the companion types.

### What each shape emits, and where it is documented

The derive dispatches on the shape of its input and then runs one of two fixed sequences. Those two
sequences are what the shape-specific derives run, so each is documented on its own page rather than
twice here:

| Applied to | It emits | Documented on |
|---|---|---|
| a struct | per-field access, the representation, and the builder | [`#[derive(CgpRecord)]`](./derive_cgp_record.md) |
| an enum | the representation, the constructors, and the extractor | [`#[derive(CgpVariant)]`](./derive_cgp_variant.md) |

Read the matching page for the shapes each accepts, the expansion in full, and the corner cases. The one
restriction worth knowing before you get there is on the enum side: **every variant must carry exactly
one unnamed payload**, and there is no per-variant opt-out.

### Choosing among the three

The three derives are interchangeable wherever the shape allows, so the choice is about what you want
the code to say. There is no output difference to weigh — `CgpData` on a struct emits exactly what
`CgpRecord` emits, and on an enum exactly what `CgpVariant` emits.

- **`#[derive(CgpData)]`** is the default. Use it unless you have a reason not to.
- **[`#[derive(CgpRecord)]`](./derive_cgp_record.md)** says "this is always a struct" and rejects
  anything else at parse time.
- **[`#[derive(CgpVariant)]`](./derive_cgp_variant.md)** says "this is always an enum", likewise.

Reach for a shape-specific one when the name earns its keep as documentation, or when you want the error
for a wrong shape to arrive at the derive rather than further in.

## Examples

One derive covers both halves of a program that builds records and takes enums apart:

```rust
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Circle {
    pub radius: f64,
}

#[derive(CgpData)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

`Circle` and `Rectangle` gain field access, a shape, and a builder; `Shape` gains a variant list,
constructors, and an extractor. Worked examples of each half are on the
[record](./derive_cgp_record.md#examples) and [variant](./derive_cgp_variant.md#examples) pages.

## When to reach for it, and when not

**Reach for one of these derives when generic code has to work over the type's own structure.** That is
the test, and it is narrower than it sounds. Most types in a CGP program want
[`#[derive(HasField)]`](./derive_has_field.md) and nothing more, because most of what implementations
need from a context — the type a capability runs against, which supplies values as its fields — is to
read one value out of it.

The cases that do earn it are specific.

- **A record assembled from independent pieces.** When several parts of a program each contribute part
  of a struct and no one place should know the whole type, that is the extensible builder pattern and
  this is what it runs on.
- **An enum handled one variant at a time by independent code.** When variants and the operations over
  them both need to grow without editing each other, this is the extensible visitor pattern.
- **A framework over any user type.** A serializer, a validator, or a mapper written once against the
  shape rather than once per type.

And the cases that do not:

- **A closed enum with fixed operations.** A `match` is clearer than any machinery, and it already gives
  exhaustiveness.
- **A struct whose fields are only ever read.** [`#[derive(HasField)]`](./derive_has_field.md) is the
  whole answer, and it is one impl pair per field rather than a companion type and a dozen impls.
- **A one-off structural manipulation.** A lighter generic-programming library is a better fit than
  adopting this family for a single conversion.

When you want only part of the output, the family splits along the lines you would expect, and deriving
the slice is the cheaper choice: [`#[derive(HasField)]`](./derive_has_field.md) for the getters,
[`#[derive(HasFields)]`](./derive_has_fields.md) for the representation,
[`#[derive(BuildField)]`](./derive_build_field.md) for the record builder,
[`#[derive(ExtractField)]`](./derive_extract_field.md) for the extractor, and
[`#[derive(FromVariant)]`](./derive_from_variant.md) for the variant constructors. The umbrella is the
right call once you want most of them.

One honest cost: this is the heaviest derive in CGP. It generates a companion type and, for an enum, two
of them, plus an impl per field or variant on top of the whole-type impls. That is compile-time work,
and the generated types appear by name in error messages. It buys guarantees a runtime builder cannot
give, and it is not free.

## Under the hood

:::note

### Advanced

This section covers only what the umbrella itself does. The expansions are on the two shape pages,
because that is where they belong once the derives are documented separately.

:::

Nothing about the umbrella is special. `#[derive(CgpData)]` inspects the item, dispatches on whether it
is a struct or an enum, and then runs exactly the same code path the matching shape-specific derive
runs — [`#[derive(CgpRecord)]`](./derive_cgp_record.md#under-the-hood) enters the first directly and
[`#[derive(CgpVariant)]`](./derive_cgp_variant.md#under-the-hood) the second. That shared dispatch is
why all three agree on their output, and why a claim about what `CgpData` emits is always a claim about
one of the other two.

Applied to a union, all three fail: the family models products and sums, and a union is neither.

## Gotchas

**Which gotchas apply depends on the shape**, and the shape pages carry them: the
[record ones](./derive_cgp_record.md#gotchas) — the companion's cleared attributes, position-keyed tuple
builders, the newtype special case — and the [variant ones](./derive_cgp_variant.md#gotchas) — the
one-payload rule and the seven reserved variant names. Two are worth repeating here because they catch
people who reached for the umbrella without reading either.

**Every enum variant needs exactly one unnamed payload.** A unit, multi-field, or struct-style variant
fails, with no per-variant opt-out. [`#[derive(HasFields)]`](./derive_has_fields.md) is the one derive in
the family that accepts all four shapes, so it is what an enum with mixed variants gets.

**Absence is spelled differently on the two sides.** `IsNothing` for a missing record field, `IsVoid` for
a ruled-out variant, and they are not interchangeable. An error mentioning the wrong one usually means
record and variant machinery have been crossed.

**These are the heaviest derives in CGP.** One `#[derive(CgpData)]` on a five-field struct generates a
companion type and roughly twenty impls. If only part of the output is wanted, derive the slice.

## Related constructs

- [`#[derive(CgpRecord)]`](./derive_cgp_record.md) and
  [`#[derive(CgpVariant)]`](./derive_cgp_variant.md) — the two faces this dispatches to.
- [`#[derive(HasField)]`](./derive_has_field.md) — the per-field access slice, on its own.
- [`#[derive(HasFields)]`](./derive_has_fields.md) — the representation slice, on its own, and the only
  one that accepts every variant shape.
- [`#[derive(BuildField)]`](./derive_build_field.md) — the record builder slice, on its own.
- [`#[derive(ExtractField)]`](./derive_extract_field.md) — the extractor slice, on its own.
- [`#[derive(FromVariant)]`](./derive_from_variant.md) — the variant constructors, on their own.
- [`HasBuilder`](../traits/has_builder.md) and [`ExtractField`](../traits/extract_field.md) — the two
  trait families the shapes generate impls for.
- [`MapType`](../traits/map_type.md) — the markers the companion types are parameterized by.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — the struct half, and the extensible builder
  pattern.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum half, and the extensible visitor
  pattern.
- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to a handler per field or
  variant.

## Source

- Entry point: [`cgp_data.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_data.rs)
- Shape dispatch: [`cgp_data/item.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/item.rs)
- Record and variant codegen: [`cgp_data/record.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/record.rs)
  and [`cgp_data/variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/variant.rs)
- Runtime traits: [`cgp-field/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-field/src/traits)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
