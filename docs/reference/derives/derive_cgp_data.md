---
sidebar_label: '#[derive(CgpData)], CgpRecord & CgpVariant'
---

# `#[derive(CgpData)]`, `CgpRecord` & `CgpVariant`

The extensible-data derives: the umbrella and its struct and enum faces.

## What it's for

A plain Rust struct or enum is opaque to generic code. There is no way to refer to "the `first_name` field"
or "the `Circle` variant" through a type parameter, so anything that must work across several data types ends
up written once per type.

`#[derive(CgpData)]` is the one-line answer: it turns a struct or an enum into **extensible data**, a type
whose fields or variants generic code can name, read, construct, and take apart without ever mentioning the
concrete type. It is the umbrella derive, and what it generates depends entirely on what it is applied to.

On a **struct** it produces the record machinery: per-field access, the whole-shape field list, and an
incremental builder that assembles a value one field at a time. On an **enum** it produces the variant
machinery: the whole-shape variant list, a constructor per variant, and an incremental extractor that peels
variants off one at a time.

The two things this buys are worth naming separately, because together they are what "extensible" means
here. The **representation** view describes the type as a list of named entries, which is what lets one
implementation serialize any record or dispatch over any enum. The **incremental** view adds a companion type
that tracks, in its own type parameters, which fields are present or which variants are still possible — so a
half-built value and a finished one are *different types*, and using one where the other belongs is a compile
error rather than a runtime panic.

`#[derive(CgpRecord)]` and `#[derive(CgpVariant)]` are the same thing restricted to one shape. They run the
same code and produce the same output; the only difference is that each refuses the shape it is not for.

## Using it

All three derives take no arguments and have no helper attributes. `CgpData` accepts a struct or an enum; the
other two accept exactly one:

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

### Choosing among the three

The three are interchangeable wherever the shape allows, so the choice is about what you want the code to
say.

- **`#[derive(CgpData)]`** is the default. Use it unless you have a reason not to.
- **`#[derive(CgpRecord)]`** says "this is always a struct" and rejects anything else at parse time.
- **`#[derive(CgpVariant)]`** says "this is always an enum", likewise.

There is no output difference to weigh: `CgpData` on a struct emits what `CgpRecord` emits, and `CgpData` on
an enum emits what `CgpVariant` emits. Reach for a shape-specific one when the name earns its keep as
documentation, or when you want the error for a wrong shape to arrive at the derive rather than further in.

### The struct requirements

Every struct shape is accepted. A named-field struct is keyed by field name, a tuple struct by position, and
a fieldless struct is the degenerate case rather than an error — its companion type takes no parameters at
all, so `builder()` is immediately finalizable because there is nothing to track.

### The enum requirement

**Every variant must carry exactly one unnamed payload.** This is the family's one real restriction, and it
comes from the two slices that take an enum apart: each has to name a single type per variant, and a unit,
multi-field, or struct-style variant gives it none or several. Such a variant fails with
`Expected variant to contain exactly one unnamed field`, and there is no way to opt one variant out — an enum
that mixes shapes cannot take these derives at all.

The fix is to wrap the richer payload in its own struct, so the variant's value stays a single nameable type:

```rust
// rejected: a struct-style variant has no single payload type
pub enum Shape {
    Circle { radius: f64 },
}

// accepted: the payload is one named type
pub struct Circle {
    pub radius: f64,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
}
```

That is idiomatic rather than a workaround — the payload struct usually wants to be a type in its own right,
and it can derive `CgpData` too, which is how a nested shape becomes reachable.

One escape hatch is worth knowing: [`#[derive(HasFields)]`](./derive_has_fields.md) accepts **all four**
variant shapes, because it only describes a variant rather than deconstructing it. So an enum with mixed
variants can still have a structural representation; what it cannot have is the generic constructor or the
extractor. A variantless enum, meanwhile, is accepted and degenerates the same way a fieldless struct does.

## Examples

The record side is most useful for assembling one struct out of pieces. Because both structs derive the
machinery, a builder can copy every shared field from another record in one step and fill in the rest:

```rust
use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

#[derive(CgpData)]
pub struct Employee {
    pub employee_id: u64,
    pub first_name: String,
    pub last_name: String,
}

fn promote(person: Person, id: u64) -> Employee {
    Employee::builder()                                       // every field absent
        .build_from(person)                                   // first_name + last_name now present
        .build_field(PhantomData::<Symbol!("employee_id")>, id)
        .finalize_build()                                     // all present: this is where it closes
}
```

Neither struct knows about the other. They share two field *names*, and the names are matched at the type
level, so `build_from` moves those fields across and leaves `employee_id` for the caller. Remove the
`build_field` line and this stops compiling, because `finalize_build` only exists once every field is present.

The variant side is most useful for matching generically with a guaranteed-exhaustive end. Each extraction
either yields the payload or hands back a *remainder* whose type has that variant ruled out:

```rust
use cgp::core::field::traits::FinalizeExtractResult;
use cgp::prelude::*;

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

fn area(shape: Shape) -> f64 {
    match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
        Ok(circle) => core::f64::consts::PI * circle.radius * circle.radius,
        Err(remainder) => {
            let rect = remainder
                .extract_field(PhantomData::<Symbol!("Rectangle")>)
                .finalize_extract_result();   // no variants left, so this cannot fail
            rect.width * rect.height
        }
    }
}
```

There is no wildcard arm and no `unreachable!()`. After the second extraction the remainder's type has both
variants ruled out, which makes it uninhabited, and `finalize_extract_result` is what discharges a value that
cannot exist. Add a third variant to `Shape` and this function stops compiling until it is handled — the same
guarantee a concrete `match` gives, recovered for code that never names the enum.

## When to reach for it, and when not

**Reach for one of these derives when generic code has to work over the type's own structure.** That is the
test, and it is narrower than it sounds. Most types in a CGP program want
[`#[derive(HasField)]`](./derive_has_field.md) and nothing more, because most of what implementations need
from a context — the type a capability runs against, which supplies values as its fields — is to read one
value out of it.

The cases that do earn it are specific.

- **A record assembled from independent pieces.** When several parts of a program each contribute part of a
  struct and no one place should know the whole type, that is the extensible builder pattern and this is what
  it runs on.
- **An enum handled one variant at a time by independent code.** When variants and the operations over them
  both need to grow without editing each other, this is the extensible visitor pattern.
- **A framework over any user type.** A serializer, a validator, or a mapper written once against the shape
  rather than once per type.

And the cases that do not:

- **A closed enum with fixed operations.** A `match` is clearer than any machinery, and it already gives
  exhaustiveness. Reach for the variant derives when the variant set is open or independent modules must each
  contribute one.
- **A struct whose fields are only ever read.** [`#[derive(HasField)]`](./derive_has_field.md) is the whole
  answer, and it is one impl pair per field rather than a companion type and a dozen impls.
- **A one-off structural manipulation.** A lighter generic-programming library is a better fit than adopting
  this family for a single conversion.

When you want only part of the output, the family splits along the lines you would expect, and deriving the
slice is the cheaper choice: [`#[derive(HasField)]`](./derive_has_field.md) for the getters,
[`#[derive(HasFields)]`](./derive_has_fields.md) for the representation,
[`#[derive(BuildField)]`](./derive_build_field.md) for the record builder,
[`#[derive(ExtractField)]`](./derive_extract_field.md) for the extractor, and
[`#[derive(FromVariant)]`](./derive_from_variant.md) for the variant constructors. The umbrella is the right
call once you want most of them.

One honest cost: this is the heaviest derive in CGP. It generates a companion type and, for an enum, two of
them, plus an impl per field or variant on top of the whole-type impls. That is compile-time work, and the
generated types appear by name in error messages. It buys guarantees a runtime builder cannot give, and it is
not free.

## Under the hood

:::note

### Advanced

This section shows what the derives generate. You do not need it to use them, but the companion types appear
by name in compiler errors, so recognizing one turns an intimidating error into a legible one.
`cargo cgp expand` prints the same thing for your own code, with the tags resugared.

:::

`#[derive(CgpData)]` dispatches on the shape of its input and then runs one of two fixed sequences. Nothing
about the umbrella is special: `#[derive(CgpRecord)]` enters the first sequence directly and
`#[derive(CgpVariant)]` the second, which is why all three agree on their output.

### The record expansion

For a struct, the derive emits three groups in order. From:

```rust
#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
```

it first emits the **per-field access** — a `HasField` and a `HasFieldMut` impl per field, exactly what
[`#[derive(HasField)]`](./derive_has_field.md) produces on its own. Then the **representation**, exposing the
struct as a product of named entries with conversions in both directions, which is
[`#[derive(HasFields)]`](./derive_has_fields.md)'s output:

```rust
impl HasFields for Person {
    type Fields = Product![
        Field<Symbol!("first_name"), String>,
        Field<Symbol!("last_name"), String>,
    ];
}

// plus HasFieldsRef, FromFields, ToFields, ToFieldsRef
```

Then the **builder**, which is [`#[derive(BuildField)]`](./derive_build_field.md)'s output and where the
companion type appears. Each field's type is wrapped in a [`MapType`](../traits/map_type.md) marker, so a
field can be present (`IsPresent`, holding the value) or absent (`IsNothing`, holding `()`):

```rust
pub struct __PartialPerson<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub last_name: <__F1__ as MapType>::Map<String>,
}

impl HasBuilder for Person {
    type Builder = __PartialPerson<IsNothing, IsNothing>;   // the empty builder
    // ...
}

impl FinalizeBuild for __PartialPerson<IsPresent, IsPresent> {
    // ...                                                     only at all-present
}
```

That pair of impls is the whole safety argument: `builder()` starts at all-absent, each `build_field` flips
one marker, and `finalize_build` exists only at all-present, so finalizing early is a missing impl rather than
a runtime check. The per-field `UpdateField` impls that move a marker, and the `HasField` impls on the
companion that let a set field be read back, follow.

### The variant expansion

For an enum, the derive emits three groups too, but a different three. From:

```rust
#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

it first emits the **representation** over a sum rather than a product:

```rust
impl HasFields for Shape {
    type Fields = Sum![
        Field<Symbol!("Circle"), Circle>,
        Field<Symbol!("Rectangle"), Rectangle>,
    ];
}

// plus HasFieldsRef, FromFields, ToFields, ToFieldsRef
```

Then the **constructors**, one per variant, which is
[`#[derive(FromVariant)]`](./derive_from_variant.md)'s output:

```rust
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;

    fn from_variant(_tag: PhantomData<Symbol!("Circle")>, value: Circle) -> Self {
        Self::Circle(value)
    }
}
```

Then the **extractor**, which is [`#[derive(ExtractField)]`](./derive_extract_field.md)'s output and where two
companion enums appear — `__PartialShape` for owned extraction and `__PartialRefShape` for borrowed. A
variant's payload is wrapped in a marker that is either `IsPresent` or `IsVoid`, the latter mapping it to the
uninhabited `Void`:

```rust
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Rectangle(<__F1__ as MapType>::Map<Rectangle>),
}

impl HasExtractor for Shape {
    type Extractor = __PartialShape<IsPresent, IsPresent>;   // every variant still possible
    // ...
}

impl FinalizeExtract for __PartialShape<IsVoid, IsVoid> {    // nothing left: uninhabited
    // ...
}
```

Note the asymmetry with the record side, which is the detail most worth carrying away: **a record uses
`IsNothing` for a field that is not there, and a variant uses `IsVoid` for one that has been ruled out.**
`IsNothing` maps a type to `()`, which is inhabited; `IsVoid` maps it to `Void`, which is not. That is exactly
why extraction ends the way it does — once every variant is `IsVoid` the whole companion enum is uninhabited,
so a value of it cannot exist and can be discharged to produce anything.

The companion names are reserved: `__Partial{Name}` and, for the borrowed extractor, `__PartialRef{Name}`.
They keep the original type's visibility, so a `pub` struct or enum yields a `pub` companion.

Each generated impl is aimed at the token it came from — a per-field impl at its field, a per-variant impl at
its variant, a whole-type impl at the type name — so a conflict with a hand-written impl underlines that token
rather than the whole derive.

## Gotchas

**Every variant needs exactly one unnamed payload.** A unit, multi-field, or struct-style variant fails, with
no per-variant opt-out. [`#[derive(HasFields)]`](./derive_has_fields.md) is the one derive in the family that
accepts all four shapes, so it is what an enum with mixed variants gets.

**The companion type carries none of your attributes.** The derive clears them, so a
`#[derive(Debug, Clone)]` on the record does not reach `__Partial{Name}` and a partially-built value can be
neither printed nor cloned. Read a set field back through the companion's `HasField` impl instead.

**A tuple struct's builder is keyed by position.** Its companion exposes `UpdateField<Index<0>, _>` rather
than symbol-keyed impls, so `build_field` takes `PhantomData::<Index<0>>`.

**A single-field tuple struct's representation is the inner type directly**, not a one-element product — the
newtype special case described on the [`HasFields`](./derive_has_fields.md) page, which the record path
inherits.

**Absence is spelled differently on the two sides.** `IsNothing` for a missing record field, `IsVoid` for a
ruled-out variant, and they are not interchangeable. An error mentioning the wrong one usually means record
and variant machinery have been crossed.

**Seven variant names are reserved on an enum.** Because the generated impls name their associated types
through `Self::…`, a variant called `Fields`, `FieldsRef`, `Value`, `Remainder`, `Extractor`, `ExtractorRef`,
or `ExtractorMut` makes that path ambiguous and the derive fails with `ambiguous associated item`, headlined at
the derive. Whether the message also names the variant depends on which part collided: the representation and
constructor impls are generated for your enum and point a note at the real variant, while the extractor's are
generated for the companion enums and point back at the derive. Renaming the variant is the fix. Record field names are unaffected,
since a field is not in the same namespace as an associated type.

**The degenerate shapes compile.** A fieldless struct yields a parameterless companion whose `builder()` is
immediately finalizable, and a variantless enum yields bare companion enums. Neither is an error, and neither
does anything useful.

**These are the heaviest derives in CGP.** One `#[derive(CgpData)]` on a five-field struct generates a
companion type and roughly twenty impls. If only part of the output is wanted, derive the slice.

## Related constructs

- [`#[derive(HasField)]`](./derive_has_field.md) — the per-field access slice, on its own.
- [`#[derive(HasFields)]`](./derive_has_fields.md) — the representation slice, on its own, and the only one
  that accepts every variant shape.
- [`#[derive(BuildField)]`](./derive_build_field.md) — the record builder slice, on its own.
- [`#[derive(ExtractField)]`](./derive_extract_field.md) — the extractor slice, on its own.
- [`#[derive(FromVariant)]`](./derive_from_variant.md) — the variant constructors, on their own.
- [`HasBuilder`](../traits/has_builder.md) — the builder trait family the record side generates impls for.
- [`ExtractField`](../traits/extract_field.md) — the extractor trait family the variant side generates impls
  for.
- [`MapType`](../traits/map_type.md) — the `IsPresent`/`IsNothing`/`IsVoid` markers the companion types are
  parameterized by.
- [`CanUpcast`](../traits/cast.md) — converting between two types whose shapes overlap, built on this
  machinery.
- [Dispatch combinators](../providers/dispatch_combinators.md) — the providers that route a record or a
  variant to per-entry handlers.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — the struct half, and the extensible builder
  pattern.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum half, and the extensible visitor
  pattern.
- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to a handler per field or
  variant.

## Source

- Entry points: [`cgp_data.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_data.rs),
  [`cgp_record.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_record.rs),
  and [`cgp_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_variant.rs)
- Shape dispatch: [`cgp_data/item.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/item.rs)
- Record and variant codegen: [`cgp_data/record.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/record.rs)
  and [`cgp_data/variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/variant.rs)
- Runtime traits: [`cgp-field/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-field/src/traits)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
