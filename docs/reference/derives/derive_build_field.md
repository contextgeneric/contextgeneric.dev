---
sidebar_label: '#[derive(BuildField)]'
sidebar_position: 6
---

# `#[derive(BuildField)]`

`#[derive(BuildField)]` generates a struct builder that tracks field presence in its type.

## Overview

A struct literal requires one place to name the concrete type and supply every field. When independent
parts of a program contribute fields, a shared constructor can force each contribution to depend on
the whole struct.

`#[derive(BuildField)]` lets generic code assemble fields independently while the compiler checks
completeness. It generates a companion struct whose type records which fields are present. Each step
sets a field and returns a new type reflecting that state. The builder can produce the original
struct only once every field is present:

```rust
let person = Person::builder()                                     // nothing set yet
    .build_field(PhantomData::<Symbol!("first_name")>, first)
    .build_field(PhantomData::<Symbol!("last_name")>, last)
    .finalize_build();                                             // every field set: closes
```

Finalizing an incomplete builder fails to compile. Removing either `build_field` call above leaves a
field absent, so `finalize_build` is unavailable for the resulting type. The presence check does not
require runtime `Option` values or a panic path.

Field tags let independent implementations contribute to the same builder. Each `build_field` call
addresses a field by its type-level tag, so generic code can set that field without knowing the
concrete struct.

## Usage

Apply `BuildField` to a struct without arguments or helper attributes:

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
[`Index<N>`](../types/index_type.md), so its builder step is
`build_field(PhantomData::<Index<0>>, value)`. Generic parameters, lifetimes, and a `where` clause are
carried onto the companion type and every generated impl.

A fieldless struct produces a builder that can be finalized immediately. Its companion does not need
field-state parameters because it has nothing to track.

The derive parses a struct, so applying it to an enum fails at parse time. The enum counterparts are
[`#[derive(ExtractField)]`](./derive_extract_field.md) for taking one apart and
[`#[derive(FromVariant)]`](./derive_from_variant.md) for constructing one.

### What it does *not* generate

`BuildField` generates the builder without adding accessors or a structural representation to the
original struct. Derive [`HasField`](./derive_has_field.md) for per-field access and
[`HasFields`](./derive_has_fields.md) for the representation. [`CgpData`](./derive_cgp_data.md)
includes all of these outputs.

A struct deriving only `BuildField` supports generic construction without generic reads from the
finished value. This suits an output type that a pipeline assembles and returns.

### The three ways to fill a field

The library provides methods to set a field, merge source fields, and remove a field. The derive
supplies the implementations these methods require.

**`build_field`** sets an absent field to a value, as shown above.

**`build_from`** moves fields from a source record into the target builder. It comes from
[`CanBuildFrom`](../traits/casting/can_build_from.md), imported from `cgp::core::field::impls`:

```rust
Employee::builder()
    .build_from(person)                                        // first_name + last_name at once
    .build_field(PhantomData::<Symbol!("employee_id")>, id)
    .finalize_build()
```

The source of `build_from` must also derive [`HasFields`](./derive_has_fields.md).
`build_from` traverses the source's field list, which `BuildField` alone does not provide. The target
needs only `BuildField`.

**`take_field`** removes a present field and returns its value alongside a builder with that field
absent. It reverses `build_field` and comes from `TakeField`, which is not in the prelude. Import it
from `cgp::core::field::traits` for direct calls. `build_from` uses it internally to remove source
fields.

`builder()` creates an empty builder, while `into_builder()` turns a finished struct into an
all-present builder. Use `into_builder()` when generic code needs to redistribute existing fields.

## Examples

A builder can extend a source record with additional fields. Both types below derive `BuildField`,
and the source also derives `HasFields` so `build_from` can traverse its fields:

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

The conversion relates `FooBar` and `FooBarBaz` through their field names and types. Neither struct
names the other in its definition.

The companion's `HasField` implementation allows a field to be read once it has been set:

```rust
let partial = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());

assert_eq!(partial.get_field(PhantomData::<Symbol!("first_name")>), "Alice");
```

Reading `last_name` at this point fails to compile because its field is still absent.

`into_builder()` and `take_field()` let generic code remove fields from a finished value and rebuild
it:

```rust
use cgp::core::field::traits::TakeField;

let builder = person.into_builder();                            // all fields present

let (first_name, remainder) = builder.take_field(PhantomData::<Symbol!("first_name")>);
// `remainder` is now missing first_name, so it cannot be finalized

let person = remainder
    .build_field(PhantomData::<Symbol!("first_name")>, first_name)
    .finalize_build();                                          // all present again
```

## When to use it

Use `BuildField` when independent code must assemble a struct without knowing its concrete type.
Choose it for the generic field interface, rather than for the number of fields:

- **Independent field contributions:** merge outputs from separate implementations and let the
  compiler check that the target is complete.
- **A generic target type:** address fields by tags when the builder cannot name the concrete struct.
- **Ordinary construction:** prefer a struct literal when one place already knows the type and fields.
  Rust checks its completeness without a generated companion.
- **Defaults or validation:** use the [optional-field extensions](../traits/optional/has_optional_builder.md)
  for defaulted and optional fields, or a hand-written builder for custom validation. This derive
  tracks presence and does not supply defaults or validation at finalization. An `Option<T>` field
  still needs to be set in the plain builder, even when its value is `None`.

Choose the derive according to the operations the struct needs:

- **[`CgpData`](./derive_cgp_data.md) or [`CgpRecord`](./derive_cgp_record.md)**: include per-field
  access and a structural representation alongside the builder.
- **`BuildField` alone**: provide generic construction when the finished value does not need generic
  field access or a structural representation.

## Under the hood

The companion struct `__Partial{Name}` stores each field through a
[`MapType`](../traits/type-level/map_type.md) parameter. `IsPresent` maps `T` to `T`, `IsNothing` maps
it to `()`, and `IsVoid` maps it to the uninhabited `Void`. For this input:

```rust
#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

The generated companion wraps each field in its corresponding marker:

```rust
pub struct __PartialPerson<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub last_name: <__F1__ as MapType>::Map<String>,
}
```

The builder implementations define the initial state, conversion from a finished value, target type,
and finalization. Only the all-present companion implements `FinalizeBuild`:

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

Each field gets an `UpdateField` implementation that changes its marker and returns its old value
alongside the updated companion:

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

Each field also gets a `HasField` implementation on the companion, available when that field's marker
is `IsPresent`:

```rust
impl<__F1__: MapType> HasField<Symbol!("first_name")> for __PartialPerson<IsPresent, __F1__> {
    type Value = String;
    // ...
}
```

`BuildField` and `TakeField` are library blanket implementations over `UpdateField`. The derive
generates `UpdateField`, which supports changing a field to a chosen marker. The library supplies
these specific transitions:

- `BuildField<Tag>` covers `UpdateField<Tag, IsPresent, Mapper = IsNothing>`, the absent-to-present move. So
  `build_field` is `update_field` in that one direction.
- `TakeField<Tag>` covers `UpdateField<Tag, IsNothing, Mapper = IsPresent>`, the present-to-absent move.

`FinalizeBuild` is likewise a library trait; the derive supplies only the all-present impl of it.

The companion preserves the original struct's visibility and each field's visibility. The derive
creates it by cloning and renaming the struct, so a `pub` struct with `pub` fields produces a public
companion with public fields.

The companion does not inherit the original attributes. A `#[derive(Debug, Clone)]` on the record
therefore does not make the builder printable or cloneable. Its fields use projections such as
`<__F0__ as MapType>::Map<String>`, whose trait bounds depend on the marker.

Each generated impl is aimed at the token it came from: a per-field impl at its field, a whole-struct impl at
the struct name.

## Common Mistakes

**An incomplete builder cannot call `finalize_build`.** The all-present implementation does not apply
while any field is unset. Check the companion type in the diagnostic for `IsNothing` to identify the
missing field.

**The companion does not inherit `Debug`, `Clone`, or `PartialEq`.** The derive clears the original
attributes. Read a set field through the companion's `HasField` implementation when inspecting a
partial value.

**`build_from` needs `HasFields` on its source in addition to `BuildField`.** Deriving only
`BuildField` on both structs leaves the source without the field list that the conversion traverses.
The error reports an unsatisfied [`HasFields`](./derive_has_fields.md) bound on the source.

**`TakeField` is not in the prelude.** Import it from `cgp::core::field::traits` to call `take_field`
directly. `CanBuildFrom`, which provides `build_from`, comes from `cgp::core::field::impls`.

**The plain builder tracks presence without defaults or validation.** Set each field explicitly or
use [`CanFinalizeWithDefault`](../traits/optional/can_finalize_with_default.md) to fill unset fields
from their default values.

**A tuple struct's steps are keyed by position.** Use `PhantomData::<Index<0>>`, not a `Symbol!` of the
number.

**`build_from` uses field names, regardless of declaration order.** Renaming a source field breaks
the correspondence with the target field. If the target field remains unset, `finalize_build` fails
to resolve, so the error appears at finalization rather than at the rename.

**It does not accept an enum.** The enum counterparts are
[`#[derive(ExtractField)]`](./derive_extract_field.md) and
[`#[derive(FromVariant)]`](./derive_from_variant.md).

## Related constructs

These references cover the related derives, generated traits, and supporting types:

- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella derive, which includes this slice.
- [`#[derive(HasField)]`](./derive_has_field.md) — per-field access on the original struct, which this derive
  does not generate.
- [`#[derive(HasFields)]`](./derive_has_fields.md) — the whole-shape representation, which this derive does
  not generate either.
- [`#[derive(ExtractField)]`](./derive_extract_field.md) — the enum analogue: incremental deconstruction
  rather than construction.
- [`HasBuilder`](../traits/builder/has_builder.md) — the builder trait family, including `UpdateField`, `BuildField`,
  `TakeField`, and `FinalizeBuild`.
- [`MapType`](../traits/type-level/map_type.md) — the `IsPresent`/`IsNothing`/`IsVoid` markers the companion is
  parameterized by.
- [`CanBuildFrom`](../traits/casting/can_build_from.md) — where `build_from` is documented.
- [`HasOptionalBuilder`](../traits/optional/has_optional_builder.md) and
  [`CanFinalizeWithDefault`](../traits/optional/can_finalize_with_default.md) — optional and defaulted fields,
  which the plain builder does not model.
- [Dispatch combinators](../providers/dispatch/index.md) — the providers that run several builder
  implementations and merge their outputs.

The ideas behind it are explained on these concept pages:

- [Extensible records](/docs/concepts/extensible-records) — partial records, and the extensible builder
  pattern this derive is the foundation of.

## Source

The implementation is defined in these source files:

- Entry point: [`derive_build_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_build_field.rs)
- Codegen: [`cgp_data/derive_builder/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_builder)
- Traits: [`build_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/build_field.rs),
  [`update_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/update_field.rs),
  [`take_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/take_field.rs),
  and [`has_builder.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_builder.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
