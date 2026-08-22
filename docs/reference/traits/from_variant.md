---
sidebar_label: 'FromVariant'
---

# `FromVariant`

Generic construction of an enum from a named variant.

## Overview

`Shape::Circle(circle)` names two things: the enum and the variant. That is fine where both are known and
useless to code that knows neither — a routine handed a payload and told which variant to wrap it in cannot
write that expression.

`FromVariant<Tag>` gives it a way to. The variant is selected by its *name as a type*, so the choice becomes a
parameter rather than syntax:

```rust
Shape::from_variant(PhantomData::<Symbol!("Circle")>, circle)
```

That call is exactly `Shape::Circle(circle)`. The difference is that the tag can come from a type parameter, so
one function can build whichever variant it was asked for, on whatever enum implements the trait for that tag.

**This is the smallest trait in the extensible-data family.** There is no companion type, no state, and nothing
to track — building a single variant has no intermediate states. It is the construction counterpart to
[`ExtractField`](./extract_field.md)'s deconstruction, and the impls come from
[`#[derive(FromVariant)]`](../derives/derive_from_variant.md), one per variant.

## Usage

The trait is in the prelude. It carries the payload type as an associated `Value` and one associated function:

```rust
pub trait FromVariant<Tag> {
    type Value;

    fn from_variant(_tag: PhantomData<Tag>, value: Self::Value) -> Self;
}
```

`Tag` is the variant's name as a [`Symbol!`](../macros/symbol.md) type-level string, `Value` is that variant's
payload type, and `from_variant` wraps a payload into the enum. The `PhantomData<Tag>` argument carries no
data; it exists so a caller can pick which variant to build when several impls — one per variant — are in scope
on the same enum.

The trait is implemented once per variant, each impl fixing its own `Tag` and `Value`, so **choosing the impl
*is* choosing the variant**.

### Naming the payload type generically

The associated `Value` is what makes a generic signature possible: a function that does not know the variant
still needs to name the type it takes. Project it through the trait:

```rust
fn wrap<Tag>(tag: PhantomData<Tag>, value: <Shape as FromVariant<Tag>>::Value) -> Shape
where
    Shape: FromVariant<Tag>,
{
    Shape::from_variant(tag, value)
}
```

`<Shape as FromVariant<Tag>>::Value` is the payload type *derived from the tag*. Without it there would be
nothing to write in the parameter position, which is the whole reason the trait carries an associated type
rather than taking the payload as a second parameter.

## Examples

One `wrap` builds either variant, and its payload type follows the tag:

```rust
use cgp::prelude::*;

#[derive(FromVariant)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

fn wrap<Tag>(tag: PhantomData<Tag>, value: <Shape as FromVariant<Tag>>::Value) -> Shape
where
    Shape: FromVariant<Tag>,
{
    Shape::from_variant(tag, value)
}

let circle = wrap(PhantomData::<Symbol!("Circle")>, Circle { radius: 2.0 });
let rect = wrap(
    PhantomData::<Symbol!("Rectangle")>,
    Rectangle { width: 3.0, height: 4.0 },
);
```

No hand-written function can do that, because `Shape::Circle` and `Shape::Rectangle` are different expressions
taking different types.

Where this earns its keep in practice is **building through a narrow enum and widening**. A routine that only
produces some of a large enum's variants declares a small local enum, constructs into that, and lifts the
result:

```rust
use cgp::core::field::impls::CanUpcast;

let ident = LispSubExpr::Ident(Ident("+".to_owned())).upcast(PhantomData::<LispExpr>);
```

The upcast always succeeds, because every variant of the smaller enum has a home in the larger one — and it is
`FromVariant` that rebuilds each variant into the target. That is the construction-side counterpart of reading
a field through a getter: the implementation names only what it needs, and the widening is checked.
Upcasting is documented with the other [structural casts](./can_upcast.md).

## When to reach for it, and when not

**Bound on `FromVariant` when the variant to build is decided by a type parameter.** That is the whole test,
and it is narrower than the extractor's, because most code decides which variant to build at a site that can
simply name it.

- **Bound on it in a routine parameterized over the variant it produces.** There is no alternative.
- **Derive it to make a smaller enum upcastable into a larger one.** Casting between enums is built on these
  constructors, so this is what lets an implementation work in a narrow local enum and widen the result.
- **Do not reach for it for an ordinary constructor call.** `Shape::Circle(circle)` is shorter, clearer, and
  generates nothing. The trait adds a *second* way to do the same thing, for callers that cannot use the first.
- **Do not derive it alone if you also take the enum apart**, which is the usual case — reach for
  [`#[derive(CgpData)]`](../derives/derive_cgp_data.md), which bundles construction, deconstruction, and the
  shape.

The split with its counterpart is exactly what the names say: this trait puts a value *into* an enum,
[`ExtractField`](./extract_field.md) gets one *out*. For structs, the analogous field-setting primitive is
[`BuildField`](./has_builder.md).

## Under the hood

:::note

### Advanced

This section shows what the derive emits. It is the shortest expansion in the family, and worth reading once
because the impls appear by name in errors about generic construction.

:::

The derive emits **one impl per variant and nothing else**:

```rust
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;

    fn from_variant(_tag: PhantomData<Symbol!("Circle")>, value: Self::Value) -> Self {
        Self::Circle(value)
    }
}

impl FromVariant<Symbol!("Rectangle")> for Shape {
    type Value = Rectangle;

    fn from_variant(_tag: PhantomData<Symbol!("Rectangle")>, value: Self::Value) -> Self {
        Self::Rectangle(value)
    }
}
```

Each body is the plain constructor call, and there is no intermediate type, no marker, and no validation beyond
the type system's own check that the payload matches. Because the impls are distinguished *only* by their `Tag`
parameter, resolving a `from_variant` call comes down to which `Symbol!` the caller names — the compiler picks
the matching impl and inlines it to the corresponding constructor. So the generic call costs exactly what the
concrete one does.

The trait itself is defined in the library; the derive supplies only these per-variant impls. Each is aimed at
the variant it came from, so a conflict with a hand-written impl underlines that variant rather than the whole
derive.

## Gotchas

**The tag must be written out, not inferred.** `Shape::from_variant(PhantomData::<Symbol!("Circle")>, value)`
needs the turbofish, because nothing in the value determines the variant when two variants could share a payload
type.

**Two variants with the same payload type are distinguishable only by tag.** That is the reason for the previous
point, and it means a mistyped tag is a missing-impl error rather than a type mismatch.

**A variant name is matched exactly.** `Symbol!("Circle")` and `Symbol!("circle")` are unrelated types, so a
case slip reports as an unsatisfied `FromVariant` bound.

**A variant named `Value` does not compile**, because the generated signature names the payload as
`Self::Value`. The [derive's page](../derives/derive_from_variant.md) covers this with the family's other
reserved names.

**Every variant needs exactly one unnamed payload**, which is the derive's requirement rather than the trait's —
a unit, multi-field, or struct-style variant cannot be given a single `Value`. Wrap the payload in its own
struct.

## Related constructs

- [`#[derive(FromVariant)]`](../derives/derive_from_variant.md) — generates the per-variant impls; what you
  write.
- [`ExtractField`](./extract_field.md) — the reverse operation, and the trait most often derived alongside.
- [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) — bundles this with the extractor and the shape.
- [`HasBuilder`](./has_builder.md) — the struct analogue: setting one field rather than choosing one variant.
- [`Symbol!`](../macros/symbol.md) — the tag that names a variant.
- [`CanUpcast`](./can_upcast.md) — widening a narrow enum into a wider one, built on these constructors.
- [`HasFields`](./has_fields.md) — the variant shape a cast walks while rebuilding.
- [Dispatch combinators](../providers/dispatch_combinators.md) — where variant construction meets routing.

The ideas behind it:

- [Extensible variants](/docs/concepts/extensible-variants) — construction by name, and building through a
  small local enum before widening.

## Source

- Trait: [`from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/from_variant.rs)
- Derive codegen: [`cgp_data/derive_from_variant.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_from_variant.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
