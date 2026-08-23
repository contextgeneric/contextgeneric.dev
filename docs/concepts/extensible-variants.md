---
sidebar_label: 'Extensible variants'
sidebar_position: 14
---

# Extensible variants

Constructing and matching an enum through the names of its variants, with exhaustiveness still
checked by the compiler.

This page answers *how does code handle an enum whose variants it does not know?* It is the dual of
[extensible records](./extensible-records.md) and assumes it. It shows the shape of an enum as a type,
the technique that keeps exhaustiveness checked without a wildcard, and the conversions between enums
that share variants. It closes on the one restriction the derive imposes.

## The problem a `match` builds in

A concrete `enum` plus a `match` puts the full set of variants into every function that consumes it. Add
a variant and every `match` has to change; that is Rust's exhaustiveness check doing its job, and it is
usually what you want.

It stops being what you want at exactly two moments. When new variants should be addable **without
editing the code that handles the others**: a plugin adding a message kind, or a downstream crate
extending an expression language. And when a variant is defined **upstream**, where handling it means
forking rather than extending.

This is the [expression problem](https://en.wikipedia.org/wiki/Expression_problem), and the usual
trade is that you can add variants easily or add operations easily, never both. Extensible variants
decouple each variant's handling from the enum itself so that both are additions.

## An enum as a list of named variants

The move is the same as for a record: give the shape a type with
[`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data). Where a record is a *product* of named
fields, an enum is a *sum* of named variants:

```rust
#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

```rust
Sum![Field<Symbol!("Circle"), Circle>, Field<Symbol!("Rectangle"), Rectangle>]
```

Each entry pairs the variant's name, lifted into a type, with its payload. Generic code can now be
written against "an enum containing a `Circle` variant" rather than against `Shape`.

## Exhaustiveness without a wildcard

Taking a value apart runs through a **partial variant**, and this is where the design earns its keep.

The value is first converted into a form in which every variant is still possible. Then each variant is
tried in turn. If the value is that variant, its payload comes out. If not, the result is a *remainder*:
the same value, with that one variant ruled out **in the type**.

Each miss therefore narrows the type. After every variant has been tried, the remainder is a type that
rules out everything, and a type that rules out everything is uninhabited: a value that cannot exist.
The final step discharges it with a `match` that has no arms, which is sound precisely because there is
nothing to handle. No wildcard arm, no `unreachable!()`.

And the guarantee is the one a concrete `match` gives. Add a variant to the enum without handling it and
the final remainder becomes inhabited again, so the code stops compiling until the new variant is
covered. The guarantee is recovered for generic code that never named the enum.

## One handler, many enums

Put together, a per-variant handler set is written once and applied to any enum whose variants it
covers:

```rust
#[cgp_computer]
pub fn field_to_string<Tag, Value>(Field { value, .. }: Field<Tag, Value>) -> String
where
    Value: Display,
{
    value.to_string()
}

delegate_components! {
    App {
        ComputerComponent: MatchWithFieldHandlers<FieldToString>,
    }
}
```

`App` can now stringify a two-variant `Reading` and a three-variant `ExtendedReading` through the same
wiring, with no arm added anywhere, because the handler is written against a variant rather than against
an enum. That is the **extensible visitor**: the logic for each variant lives on its own, and a dispatcher
routes a value to whichever one matches. [Dispatching](./dispatching.md) is the page for the dispatcher
itself.

## Enums that share variants convert for free

Because variants are matched by name, two enums with overlapping variants interconvert with no
hand-written conversion.

**Widening always succeeds**, because every variant of the narrow enum has a place in the wide one:

```rust
let wide: ExtendedReading = narrow.upcast(PhantomData::<ExtendedReading>);
```

**Narrowing may not**, so it returns a `Result`, with the failure carrying a remainder that can be tried
against another candidate:

```rust
let narrow: Result<Reading, _> = wide.downcast(PhantomData::<Reading>);
```

Widening is also how a provider constructs a value using only the variants it knows about: build a small
local enum, lift it into the full one. That is the variant-side counterpart of reading a field through a
getter: code touching part of a shape without naming the whole.

## What it costs

**One unnamed field per variant.** A derivable enum must be a sum of products: each variant holds
exactly one unnamed payload. A richer variant wraps its data in a dedicated struct, which is why the
example above is `Circle(Circle)` rather than `Circle { radius: u64 }`. It is a real constraint on how
the enum is written, and it is the first thing anyone hits. The constraint comes from constructing and
deconstructing rather than from the shape itself:
[`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields) accepts every variant shape, so an
enum that cannot take the full family can still have a structural representation, with no generic
constructor and no extractor.

**Only types that opted in**, as with records. An enum from a crate that has not derived the shape is
invisible to all of this.

**The errors are long, and this is where they are longest.** An unhandled variant surfaces as a bound on
a partial-variant type spelling out every variant and its presence marker. It is the accurate report of
"you did not cover everything", and it does not read like one.

**And a closed enum with fixed operations should stay one.** A `match` is clearer, faster to read, and
free. This pays when the variant set is genuinely open, or when independent modules must each contribute
one, not when an enum merely has several variants.

## Where to go next

[Extensible records](./extensible-records.md) is the other half, and worth reading first if the shape
type above was unfamiliar. [Dispatching](./dispatching.md) is the machinery that runs a handler per
variant, and [Type-level DSLs](./type-level-dsls.md) is the largest thing this pattern builds: a
language whose terms are types and whose interpreter is the trait system.

For the constructs, [`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data) is the umbrella
derive and its slices are [`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields) for the
whole shape, [`#[derive(ExtractField)]`](/docs/reference/derives/derive_extract_field) for the
narrowing, and [`#[derive(FromVariant)]`](/docs/reference/derives/derive_from_variant) for construction
by name. [`ExtractField`](/docs/reference/traits/variant/extract_field) is the extractor family those produce,
[`FromVariant`](/docs/reference/traits/variant/from_variant) constructs a variant generically, and
[the casts](/docs/reference/traits/casting/can_upcast) are `CanUpcast` and
[`CanDowncast`](/docs/reference/traits/casting/can_downcast).

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
