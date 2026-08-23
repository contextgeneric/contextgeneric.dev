---
sidebar_label: '#[derive(BuildField)]'
sidebar_position: 6
---

# `#[derive(BuildField)]`

Builder support for a record.

## Overview

A struct literal has to be written in one place that names the concrete type and supplies every field at
once. That is usually what you want. It is exactly wrong when the fields come from several independent
places, none of which should know the whole type. A hand-written constructor that grows a parameter per
subsystem becomes the one file every change has to edit.

`#[derive(BuildField)]` replaces it with a builder whose completeness is tracked **in the type**. The
derive generates a companion struct that starts out with every field absent, and each step fills one
field in and returns a value of a *different* type: one where that field is now present. Only the
all-present type can be turned back into the real struct:

```rust
let person = Person::builder()                                     // nothing set yet
    .build_field(PhantomData::<Symbol!("first_name")>, first)
    .build_field(PhantomData::<Symbol!("last_name")>, last)
    .finalize_build();                                             // every field set: closes
```

Delete either middle line and this does not compile. There is no runtime check, no `Option` per field,
and no panic path: `finalize_build` simply does not exist for a value with a field still missing. A
conventional builder catches a missing field at run time; this catches it while you are typing.

The other half of the payoff is that steps are decoupled. Because each `build_field` names its field by a
type-level tag rather than by calling a method on the concrete type, generic code can fill in a field of a
struct it does not know, and two independent implementations can each contribute part of one value.

## Usage

The macro is a plain derive on a struct. It takes no arguments and has no helper attributes:

```rust
use cgp::prelude::*;

#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

Each named field becomes a [`Symbol!`](../macros/symbol.md) tag and its declared type becomes the value that
tag holds. A tuple struct works the same way with positional tags: a field at position `N` is keyed by
[`Index<N>`](../types/index.md), so its builder step is
`build_field(PhantomData::<Index<0>>, value)`. Generic parameters, lifetimes, and a `where` clause are
carried onto the companion type and every generated impl.

A fieldless struct is the degenerate case rather than an error. Its companion type takes no parameters at
all, so there is exactly one configuration of it and `builder()` is immediately finalizable. The presence
tracking has nothing to track.

The derive parses a struct, so applying it to an enum fails at parse time. The enum counterparts are
[`#[derive(ExtractField)]`](./derive_extract_field.md) for taking one apart and
[`#[derive(FromVariant)]`](./derive_from_variant.md) for constructing one.

### What it does *not* generate

This is the point of deriving it alone, so it is worth being explicit. `#[derive(BuildField)]` emits the
builder and nothing else: **no** `HasField` accessors on your struct and **no** whole-shape representation.
Those come from [`#[derive(HasField)]`](./derive_has_field.md) and
[`#[derive(HasFields)]`](./derive_has_fields.md), and the umbrella
[`#[derive(CgpData)]`](./derive_cgp_data.md) includes all three.

So a struct that derives only `BuildField` can be *built* generically and not *read* generically. That is
occasionally exactly right, for an output type a pipeline assembles and hands back, and it is why the
slice exists.

### The three ways to fill a field

The builder is driven through methods that come from the library rather than from the derive, and there are
three of them.

**`build_field`** sets one absent field to a value, which is the step shown above.

**`build_from`** copies every field the target and a source record share, in one step. It comes from
[`CanBuildFrom`](../traits/can_build_from.md), and is imported from
`cgp::core::field::impls`:

```rust
Employee::builder()
    .build_from(person)                                        // first_name + last_name at once
    .build_field(PhantomData::<Symbol!("employee_id")>, id)
    .finalize_build()
```

**The source of a `build_from` needs [`#[derive(HasFields)]`](./derive_has_fields.md) as well**, and this
is the easiest thing on this page to get wrong. `build_from` walks the *source's* field list to know what
to copy, and that list comes from `HasFields`. A source deriving only `BuildField` has a builder of its
own and still cannot be merged into anything. The target needs only this derive.

**`take_field`** goes the other way: it removes a field that is already present, handing back the value
alongside a builder with that field absent again. It is the reverse of `build_field` and comes from
`TakeField`, which unlike the rest of the family is **not in the prelude**. Import it from
`cgp::core::field::traits` when calling it directly. Most code meets it indirectly, since `build_from`
uses it to pull each field out of the source.

Two more methods sit at the ends of the process. `builder()` produces the empty builder, and
`into_builder()` goes the other way, turning a finished struct into an all-present builder so its fields
can be redistributed.

## Examples

The clearest use is extending one record into a larger one. Both structs derive the builder, and the smaller
one's fields move across in a single step:

```rust
use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

// The source of a `build_from` needs its field list too, so it derives `HasFields` as well.
#[derive(HasFields, BuildField)]
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
}

#[derive(BuildField)]
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
}

fn extend(foo_bar: FooBar) -> FooBarBaz {
    FooBarBaz::builder()
        .build_from(foo_bar)                                    // sets foo + bar
        .build_field(PhantomData::<Symbol!("baz")>, true)        // sets baz
        .finalize_build()
}
```

`FooBarBaz` does not mention `FooBar` and `FooBar` does not mention `FooBarBaz`. They share two field names,
matched at the type level, which is the whole coupling between them.

A field that has been set can be read back out mid-build, because the companion type gains a `HasField` impl
per field that is in scope only once that field is present:

```rust
let partial = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());

assert_eq!(partial.get_field(PhantomData::<Symbol!("first_name")>), "Alice");
```

Asking for `last_name` there would not compile, since that field's accessor is not in scope until it is set.

And a finished value can be taken apart and put back together, the operation generic code performs when it
redistributes fields:

```rust
use cgp::core::field::traits::TakeField;

let builder = person.into_builder();                            // all fields present

let (first_name, remainder) = builder.take_field(PhantomData::<Symbol!("first_name")>);
// `remainder` is now missing first_name, so it cannot be finalized

let person = remainder
    .build_field(PhantomData::<Symbol!("first_name")>, first_name)
    .finalize_build();                                          // all present again
```

## When to reach for it, and when not

**Derive `BuildField` when a struct is assembled by code that does not know the whole struct.** That is the
case it exists for, and it is a narrower case than "this struct has several fields".

- **Reach for it when independent parts of a program each contribute fields.** This is the extensible
  builder pattern: one implementation per subsystem, each producing a small struct, merged into a target
  whose fields the compiler checks are all supplied.
- **Reach for it when the type being built is itself generic.** Code that builds "whatever record these
  contributions add up to" needs a builder addressed by tags rather than by method names.
- **Do not reach for it for a struct you construct with a literal.** A struct literal is already checked for
  completeness, reads better, and generates nothing. This derive buys the *decoupling*, and if there is
  nothing to decouple it is pure cost.
- **Do not reach for it in place of a conventional builder** with defaults and validation. It tracks presence
  and nothing else: it has no notion of a default value, no way to run a check at finalize time, and no
  optional field unless you make the field's own type optional. The
  [optional-field extensions](../traits/has_optional_builder.md) cover the defaulted and optional cases,
  and a hand-written builder remains the better fit when the logic is the point.

Between this derive and its neighbours the choice is about how much of the machinery you want.

- **[`#[derive(CgpData)]`](./derive_cgp_data.md) or [`#[derive(CgpRecord)]`](./derive_cgp_record.md)** if the
  struct also needs per-field reads or a whole-shape representation, which is the common case. Those include
  this output.
- **`#[derive(BuildField)]` alone** when the struct is only ever built generically and never read
  generically.

## Under the hood

The derive centres on a companion struct named `__Partial{Name}`. It is your struct with one
[`MapType`](../traits/map_type.md) parameter added per field, and each field's type wrapped in that
parameter's projection. The marker decides how the field is stored: `IsPresent` maps `T` to `T`, `IsNothing`
maps it to `()`, and `IsVoid` maps it to the uninhabited `Void`. From:

```rust
#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

it emits:

```rust
pub struct __PartialPerson<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub last_name: <__F1__ as MapType>::Map<String>,
}
```

Around that struct come the entry and exit points, and the pair of them is the whole safety argument:

```rust
impl HasBuilder for Person {
    type Builder = __PartialPerson<IsNothing, IsNothing>;      // the empty builder
    fn builder() -> Self::Builder {
        __PartialPerson { first_name: (), last_name: () }
    }
}

impl IntoBuilder for Person {
    type Builder = __PartialPerson<IsPresent, IsPresent>;      // a finished value, as a builder
    // ...
}

impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialPerson<__F0__, __F1__> {
    type Target = Person;                                      // any configuration targets Person
}

impl FinalizeBuild for __PartialPerson<IsPresent, IsPresent> {  // only at all-present
    fn finalize_build(self) -> Self::Target {
        Person { first_name: self.first_name, last_name: self.last_name }
    }
}
```

Then, per field, an `UpdateField` impl: the primitive everything else is built from. It moves one field's
marker to a new state, returning the old value alongside the rebuilt companion:

```rust
impl<__M1__: MapType, __M2__: MapType, __F1__: MapType>
    UpdateField<Symbol!("first_name"), __M2__> for __PartialPerson<__M1__, __F1__>
{
    type Value = String;
    type Mapper = __M1__;                          // the field's marker before the change
    type Output = __PartialPerson<__M2__, __F1__>; // and after: only this field moved
    // ...
}
```

And, per field, a `HasField` impl on the companion that is in scope only when that field's marker is
`IsPresent`, which lets a set field be read back mid-build:

```rust
impl<__F1__: MapType> HasField<Symbol!("first_name")> for __PartialPerson<IsPresent, __F1__> {
    type Value = String;
    // ...
}
```

**Neither `BuildField` nor `TakeField` is generated.** Both are blanket impls in the library over the
`UpdateField` above, in opposite directions. That is why the derive actually writes `UpdateField`, the
general form, parameterized by the marker to move *to*:

- `BuildField<Tag>` covers `UpdateField<Tag, IsPresent, Mapper = IsNothing>`, the absent-to-present move. So
  `build_field` is `update_field` in that one direction.
- `TakeField<Tag>` covers `UpdateField<Tag, IsNothing, Mapper = IsPresent>`, the present-to-absent move.

`FinalizeBuild` is likewise a library trait; the derive supplies only the all-present impl of it.

Two properties of the companion follow from how it is built: the derive clones your struct and renames it.
It **keeps your visibility**, yours and each field's, so a `pub struct` yields a `pub struct __PartialPerson`
with `pub` fields. But it **carries none of your attributes**: those are cleared, so a
`#[derive(Debug, Clone)]` on the record does not reach the companion. That is not an oversight. A field's
type is the projection `<__F0__ as MapType>::Map<String>`, so a derived `Debug` would need bounds the macro
has no way to state. But it does mean a half-built value cannot be printed.

Each generated impl is aimed at the token it came from: a per-field impl at its field, a whole-struct impl at
the struct name.

## Common Mistakes

**`finalize_build` missing is the expected error, not a bug.** A field left unset means the all-present impl
does not apply, and the compiler reports a missing method rather than a missing field. Read it as "some field
is still absent" and check the companion type in the error for which marker is `IsNothing`.

**The companion cannot be printed or cloned.** Its attributes are cleared, so no `Debug`, no `Clone`, no
`PartialEq`, whatever the original struct derives. Read a set field through the companion's `HasField` impl
instead.

**`build_from` needs `HasFields` on the source, not just `BuildField`.** Deriving only this macro on both
structs looks symmetric and does not work: `build_from` reads the source's field list, and only
[`#[derive(HasFields)]`](./derive_has_fields.md) provides one. The error names an unsatisfied `HasFields`
bound on the source type.

**`TakeField` is not in the prelude.** Import it from `cgp::core::field::traits` to call `take_field`
directly. `CanBuildFrom`, which provides `build_from`, comes from `cgp::core::field::impls`.

**There are no defaults and no validation.** Presence is all that is tracked. A field with a sensible default
still has to be set explicitly, unless you reach for
[`CanFinalizeWithDefault`](../traits/can_finalize_with_default.md).

**A tuple struct's steps are keyed by position.** Use `PhantomData::<Index<0>>`, not a `Symbol!` of the
number.

**Field order does not matter, but field *names* are the whole interface.** `build_from` matches by name, so
renaming a field in one struct silently stops it being copied from the other. The result is a
`finalize_build` that no longer resolves, reported at the finalize rather than at the rename.

**It does not accept an enum.** The enum counterparts are
[`#[derive(ExtractField)]`](./derive_extract_field.md) and
[`#[derive(FromVariant)]`](./derive_from_variant.md).

## Related constructs

- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella derive, which includes this slice.
- [`#[derive(HasField)]`](./derive_has_field.md) — per-field access on the original struct, which this derive
  does not generate.
- [`#[derive(HasFields)]`](./derive_has_fields.md) — the whole-shape representation, which this derive does
  not generate either.
- [`#[derive(ExtractField)]`](./derive_extract_field.md) — the enum analogue: incremental deconstruction
  rather than construction.
- [`HasBuilder`](../traits/has_builder.md) — the builder trait family, including `UpdateField`, `BuildField`,
  `TakeField`, and `FinalizeBuild`.
- [`MapType`](../traits/map_type.md) — the `IsPresent`/`IsNothing`/`IsVoid` markers the companion is
  parameterized by.
- [`CanBuildFrom`](../traits/can_build_from.md) — where `build_from` is documented.
- [`HasOptionalBuilder`](../traits/has_optional_builder.md) and
  [`CanFinalizeWithDefault`](../traits/can_finalize_with_default.md) — optional and defaulted fields,
  which the plain builder does not model.
- [Dispatch combinators](../providers/dispatch_combinators.md) — the providers that run several builder
  implementations and merge their outputs.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records, and the extensible builder
  pattern this derive is the foundation of.

## Source

- Entry point: [`derive_build_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_build_field.rs)
- Codegen: [`cgp_data/derive_builder/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_builder)
- Traits: [`build_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/build_field.rs),
  [`update_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/update_field.rs),
  [`take_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/take_field.rs),
  and [`has_builder.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_builder.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
