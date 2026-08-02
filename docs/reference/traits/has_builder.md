---
sidebar_label: 'HasBuilder'
---

# `HasBuilder`

The incremental-builder trait family behind the extensible builder pattern.

## What it's for

A struct literal supplies every field at once, in one place that names the type. This family is for the case
where that is impossible: the fields come from several independent places, none of which should know the whole
struct, and the result must still be checked at compile time.

The family's defining idea is that **field presence lives in the type**. A half-built value is a *different
type* from a finished one, and the trait that turns a partial value back into the concrete struct is
implemented only for the fully-present configuration. So finalizing early is a missing impl rather than a
runtime panic:

```rust
let person = Person::builder()                                      // every field absent
    .build_field(PhantomData::<Symbol!("first_name")>, first)
    .build_field(PhantomData::<Symbol!("last_name")>, last)
    .finalize_build();                                              // all present: this resolves
```

Delete a middle line and this does not compile.

Everything here reduces to **one primitive**, `UpdateField`, which changes one field's storage from one state
to another. `BuildField` and `TakeField` are the two directions of that change, and `HasBuilder`/`IntoBuilder`
and `FinalizeBuild` are the entry and exit points. The impls come from
[`#[derive(BuildField)]`](../derives/derive_build_field.md), which also generates the partial companion type
they all operate on.

## Using it

Most of the family is in the prelude — `HasBuilder`, `IntoBuilder`, `UpdateField`, `BuildField`,
`PartialData`, `FinalizeBuild`. The exception is **`TakeField`, which is not**; import it from
`cgp::core::field::traits` when you call `take_field` directly.

### The entry points

Two traits obtain a partial value to build into, differing in where they start:

```rust
pub trait HasBuilder {
    type Builder;
    fn builder() -> Self::Builder;
}

pub trait IntoBuilder {
    type Builder;
    fn into_builder(self) -> Self::Builder;
}
```

`HasBuilder::builder()` gives a partial value with **every field absent**. `IntoBuilder::into_builder(self)`
goes the other way, turning an existing value into a partial one with **every field present** — which is what
you want when redistributing a struct's fields rather than assembling one.

### The primitive

`UpdateField<Tag, M>` is what everything else is built from. It moves the field named by `Tag` from its
current marker to the new marker `M`, both [`MapType`](./map_type.md) markers, and returns the old value
alongside the rebuilt partial value:

```rust
pub trait UpdateField<Tag, M: MapType> {
    type Value;
    type Mapper: MapType;   // the field's marker before the update
    type Output;            // the partial value with the field now in state M

    fn update_field(
        self,
        _tag: PhantomData<Tag>,
        value: M::Map<Self::Value>,
    ) -> (<Self::Mapper as MapType>::Map<Self::Value>, Self::Output);
}
```

The `Map<…>` wrappers are the field's value *as stored* under each marker: `IsPresent` stores the value
itself, `IsNothing` stores `()`. So setting an absent field takes the real value in and hands `()` back, and
removing a present one takes `()` in and hands the real value back.

### The two directions

`BuildField` and `TakeField` are blanket impls over that primitive, each pinning one transition:

```rust
pub trait BuildField<Tag> {
    type Value;
    type Output;
    fn build_field(self, _tag: PhantomData<Tag>, value: Self::Value) -> Self::Output;
}

pub trait TakeField<Tag> {
    type Value;
    type Remainder;
    fn take_field(self, _tag: PhantomData<Tag>) -> (Self::Value, Self::Remainder);
}
```

`BuildField` is the `IsNothing → IsPresent` direction — set a currently-absent field — and `TakeField` is
`IsPresent → IsNothing` — remove a currently-present one. Because each requires a specific starting marker, the
compiler rejects building a field that is already set or taking one that is absent.

### The exit

```rust
pub trait PartialData {
    type Target;
}

pub trait FinalizeBuild: PartialData {
    fn finalize_build(self) -> Self::Target;
}
```

`PartialData::Target` names the struct being built and is implemented for **every** configuration, which is how
generic builder code knows the destination before the build is complete. `FinalizeBuild` is implemented for
**only** the all-present configuration, which is the safety argument in one line.

## Examples

The everyday shape is `builder()`, some `build_field` calls, and `finalize_build`, with `build_from` copying
every shared field from another record in one step:

```rust
use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

// `build_from` walks the *source's* field list, so the source needs `HasFields` too.
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
    FooBarBaz::builder()                                     // all absent
        .build_from(foo_bar)                                 // foo, bar now present
        .build_field(PhantomData::<Symbol!("baz")>, true)     // baz now present
        .finalize_build()                                    // only the all-present impl applies
}
```

Every line changes the partial *type*, and the last one type-checks only because every marker has reached
present. Reorder them so `finalize_build` runs before `baz` is set and it is a compile error.

The reverse direction takes a finished value apart and puts it back, which is what generic code
redistributing fields does:

```rust
use cgp::core::field::traits::TakeField;

let builder = person.into_builder();                         // all present

let (first_name, remainder) = builder.take_field(PhantomData::<Symbol!("first_name")>);
// `remainder` is missing first_name, so it cannot be finalized

let person = remainder
    .build_field(PhantomData::<Symbol!("first_name")>, first_name)
    .finalize_build();                                       // all present again
```

A field that has been set can also be read back mid-build, because the derive emits a
[`HasField`](./has_field.md) impl on the partial type gated on presence:

```rust
let partial = Person::builder()
    .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());

assert_eq!(partial.get_field(PhantomData::<Symbol!("first_name")>), "Alice");
```

Asking for `last_name` there would not compile.

## When to reach for it, and when not

**Use the methods; bound on the traits only in generic code.** In concrete code you call `builder()`,
`build_field`, and `finalize_build` and never name a trait. The traits matter when you write code that is
generic over the record being built — which is the extensible builder pattern, and the reason the family exists.

- **Bound on `HasBuilder` + `BuildField` + `FinalizeBuild`** to write a routine that assembles some record it
  does not name.
- **Bound on `PartialData`** when you need the destination type mid-build, before the value is complete.
- **Bound on `UpdateField` directly** only for an operation neither `BuildField` nor `TakeField` covers — such
  as a transition to or from `IsOptional`, which is what the
  [optional-field extensions](./optional_fields.md) do.
- **Do not reach for any of it for a struct you build with a literal.** A literal is already checked for
  completeness, reads better, and generates nothing. This family buys *decoupling*, and with nothing to
  decouple it is pure cost.

Two boundaries are worth stating plainly. This is **not a conventional builder**: it tracks presence and
nothing else — no defaults, no validation at finalize, no optional field unless the field's own type is
optional. The [optional-field extensions](./optional_fields.md) cover the defaulted and optional cases, and a
hand-written builder remains better when the *logic* is the point. And the enum counterparts are a different
family: [`ExtractField`](./extract_field.md) for taking a value apart and
[`FromVariant`](./from_variant.md) for constructing one.

## Under the hood

:::note

### Advanced

This section shows how presence is encoded and why finalizing early fails. You do not need it to use the
builder, but the partial type appears by name in every builder error, so recognizing it turns "no method named
`finalize_build`" into a legible message.

:::

The derive generates a companion struct — `__Partial{Name}` — that is your struct with one
[`MapType`](./map_type.md) parameter added per field and each field's type wrapped in that parameter's
projection:

```rust
pub struct __PartialPerson<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub last_name: <__F1__ as MapType>::Map<String>,
}
```

The marker in each position decides the storage: `IsPresent` holds the value, `IsNothing` holds `()`, `IsVoid`
holds the uninhabited `Void`. Then the two impls that make the whole thing safe:

```rust
impl HasBuilder for Person {
    type Builder = __PartialPerson<IsNothing, IsNothing>;      // the empty builder
    // ...
}

impl FinalizeBuild for __PartialPerson<IsPresent, IsPresent> {  // only at all-present
    // ...
}
```

`builder()` starts at all-absent, each `build_field` flips one marker, and `finalize_build` exists only at
all-present. There is no check to run — the impl simply is not there for an incomplete value.

The per-field `UpdateField` impl is what does the flipping, and note that only the named field's marker moves
while the rest stay generic — which is why fields can be built in any order:

```rust
impl<__M1__: MapType, __M2__: MapType, __F1__: MapType>
    UpdateField<Symbol!("first_name"), __M2__> for __PartialPerson<__M1__, __F1__>
{
    type Value = String;
    type Mapper = __M1__;                          // the marker before
    type Output = __PartialPerson<__M2__, __F1__>; // and after
    // ...
}
```

**Neither `BuildField` nor `TakeField` is generated.** Both are library blanket impls over that one
`UpdateField`, in opposite directions — `BuildField` over
`UpdateField<Tag, IsPresent, Mapper = IsNothing>` and `TakeField` over
`UpdateField<Tag, IsNothing, Mapper = IsPresent>` — which is why `UpdateField`, the general form, is what the
derive actually writes. `FinalizeBuild` is likewise a library subtrait of `PartialData`; the derive supplies
only its all-present impl.

`TakeField` is also the trait behind bulk merging: `CanBuildFrom`'s `build_from` recurses over the source's
field list, taking each field out and building it into the target.

## Gotchas

**`TakeField` is not in the prelude.** Import it from `cgp::core::field::traits`. `CanBuildFrom`, which
provides `build_from`, comes from `cgp::core::field::impls`.

**`build_from` needs [`HasFields`](./has_fields.md) on the *source*.** It walks the source's field list to know
what to copy, so a source deriving only `BuildField` cannot be merged into anything. The target needs only the
builder. This is the easiest thing here to get wrong, because deriving the same thing on both looks symmetric.

**"No method named `finalize_build`" is the expected error for an incomplete build.** A missing field means the
all-present impl does not apply, so the compiler reports a missing method rather than a missing field. Read the
partial type in the message to see which marker is still `IsNothing`.

**The partial type cannot be printed or cloned.** The derive clears the original's attributes, so no `Debug`,
no `Clone`, whatever the record derives. Read a set field through the partial type's
[`HasField`](./has_field.md) impl instead.

**There are no defaults and no validation.** Presence is all that is tracked. A field with a sensible default
still has to be set, unless you reach for the [optional-field extensions](./optional_fields.md).

**Field *names* are the whole interface for merging.** `build_from` matches by name, so renaming a field in one
struct silently stops it being copied — and the failure surfaces at `finalize_build`, not at the rename.

## Related constructs

- [`#[derive(BuildField)]`](../derives/derive_build_field.md) — generates the partial type and every impl here.
- [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) — bundles this family with the field access and the
  shape.
- [`MapType`](./map_type.md) — the `IsPresent`/`IsNothing`/`IsVoid` markers presence is encoded in.
- [`HasField`](./has_field.md) — how a set field is read back off a partial value.
- [`HasFields`](./has_fields.md) — the shape `build_from` walks, and what a merge source needs.
- [`CanUpcast`](./cast.md) — where `CanBuildFrom` and `build_from` are documented.
- [Optional fields](./optional_fields.md) — defaulted and optional finalization, layered on `UpdateField`.
- [`ExtractField`](./extract_field.md) and [`FromVariant`](./from_variant.md) — the enum counterparts.
- [Dispatch combinators](../providers/dispatch_combinators.md) — the providers that run several builder
  implementations and merge their outputs.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — partial records and the extensible builder
  pattern.

## Source

- [`has_builder.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_builder.rs)
  — `HasBuilder`, `IntoBuilder`
- [`update_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/update_field.rs)
  — `UpdateField`, the primitive
- [`build_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/build_field.rs)
  — `BuildField`, `FinalizeBuild`
- [`take_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/take_field.rs)
  — `TakeField`
- [`partial_data.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/partial_data.rs)
  — `PartialData`
- [`build_from.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/build_from.rs)
  — `CanBuildFrom`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
