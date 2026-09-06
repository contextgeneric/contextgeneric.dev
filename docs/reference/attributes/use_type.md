---
sidebar_label: '#[use_type]'
sidebar_position: 3
---

# `#[use_type]`

Import an abstract associated type and write it as a bare name.

## Overview

`#[use_type]` imports a type from the **context**, the type the code runs against. In CGP a type
supplied this way is called an [*abstract type*](/docs/concepts/abstract-types): the code names it,
and the context chooses what it actually is, at compile time, through its
[`delegate_components!`](../macros/delegate_components.md) wiring.

The type most often imported this way is a context's
[error type](/docs/concepts/modular-error-handling). One attribute lets a fallible signature name it as a
bare `Error`:

```rust
#[use_type(HasErrorType.Error)]
```

A method can then return `Result<String, Error>`, and that method compiles unchanged whether the
context chose `anyhow::Error`, `Box<dyn core::error::Error>`, or a plain `String` as its error type.
The method never names any of those. The context names one, in its wiring.

`#[use_type]` reads like a `use` statement for a type, and [Under the hood](#under-the-hood) shows how
the bare name resolves. It also has advanced forms that a bare identifier could not express, covered
[below](#importing-from-another-type): pinning the type to a concrete one, importing it from a
parameter rather than the context, and tying two abstract types together.

## Usage

Apply `#[use_type]` beside the macro it modifies, naming a trait and the associated type to import:

```rust
#[use_type(HasScalarType.Scalar)]
```

The part before the `.` is the trait and the identifier after it is the type. The **separator is a dot,
not `::`**, and that is deliberate. It leaves `::` free for the trait's own path, so
`#[use_type(errors::HasErrorType.Error)]` imports from a trait named by path and
`#[use_type(HasFooType<X>.Foo)]` imports from a particular generic instantiation.

By default the macro projects the type from `Self`, so `Scalar` expands to `<Self as HasScalarType>::Scalar`.

The attribute is accepted on [`#[cgp_fn]`](../macros/cgp_fn.md),
[`#[cgp_impl]`](../macros/cgp_impl.md), and [`#[cgp_component]`](../macros/cgp_component.md).

### Importing several types

Separate entries with commas, and **prefer one attribute carrying the whole list**, because it reads as a
single import list:

```rust
#[use_type(HasUserIdType.UserId, HasCurrencyType.Currency, HasErrorType.Error)]
```

Several types from the same trait go in a braced group, and any entry may be renamed with `as` to give it
a local alias:

```rust
#[use_type(HasFooType.{Foo, Bar as Baz})]
```

Stacking several `#[use_type]` attributes behaves identically, because the macro collects every
attribute's entries into one list before it resolves any of them. So an alias declared in one is
available to another regardless of which comes first. Use a second attribute only when there is a
reason.

### Pinning a type to a concrete one

An `= Type` clause imports the type *and* constrains it. A provider uses it to say "this only works when
the error type is `anyhow::Error`":

```rust
#[use_type(HasErrorType.{Error = anyhow::Error})]
```

That emits `Self: HasErrorType<Error = anyhow::Error>` in place of the plain bound. The macro substitutes
the right-hand side too, so it may name another import. One import can therefore tie two abstract types
together:

```rust
#[use_type(HasPasswordType.Password, HasHashedPasswordType.{HashedPassword = Password})]
```

The pin is an implementation-side constraint, so the macro **rejects it on
[`#[cgp_component]`](../macros/cgp_component.md)**, whose trait definition has nowhere to put it.

### Importing from another type

A trailing `in Context` clause changes what the type is projected from, so an import can reach a type on a
parameter rather than on `Self`:

```rust
#[use_type(HasScalarType.Scalar in Types)]
```

`Scalar` now expands to `<Types as HasScalarType>::Scalar`. The clause scopes over the whole entry, so a
braced group projects every type in it against the same target. And because `in` is a reserved word, the
parser can never mistake it for a type name.

This form also lets an import parameterize the trait it comes from. An alias may appear in
another entry's `in` clause *or* in its trait arguments, and the macro resolves the chain for you:

```rust
#[use_type(HasDbType.Db, HasPoolType<Db>.Pool)]
```

Here `Pool` is projected from `HasPoolType<<Self as HasDbType>::Db>`. Such chains may be written in any
order (see [Under the hood](#under-the-hood)), provided they do not form a cycle.

## Examples

One abstract scalar shared by a component and a provider, with neither writing `Self::` anywhere:

```rust
use cgp::prelude::*;
use core::ops::Mul;

#[cgp_type]
pub trait HasScalarType {
    type Scalar: Clone + Mul<Output = Self::Scalar>;
}

#[cgp_component(AreaCalculator)]
#[use_type(HasScalarType.Scalar)]
pub trait CanCalculateArea {
    fn area(&self) -> Scalar;
}

#[cgp_impl(new RectangleArea)]
#[use_type(HasScalarType.Scalar)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: Scalar, #[implicit] height: Scalar) -> Scalar {
        width * height
    }
}
```

The component gains `HasScalarType` as a supertrait, and the provider gains it as a bound. Every
`Scalar` in both becomes the same qualified projection, so the fields the provider reads and the value it
returns are guaranteed to agree on whatever scalar the context chose.

A context supplies the concrete type by wiring, and nothing above changes:

```rust
#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        ScalarTypeProviderComponent: UseType<f64>,
        AreaCalculatorComponent: RectangleArea,
    }
}
```

The most common use of all is an
[error type](/docs/concepts/abstract-types#the-canonical-one-a-contexts-error-type), where the import
saves a projection in every fallible signature:

```rust
#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}
```

The error type is one of three decisions a context makes about failure, and
[modular error handling](/docs/concepts/modular-error-handling#why-the-error-type-is-the-hard-one)
separates them.

## When to use it

**Use `#[use_type]` whenever a definition names an abstract type another component supplies.** You read
a hand-written supertrait plus `Self::`-qualified paths in existing code rather than write them.

Its neighbours handle the requirements that are not types, and one of them declares the type this
attribute imports.

- **A capability** is [`#[uses]`](uses.md) for a private bound or [`#[extend]`](extend.md) for a
  supertrait. Use `#[extend]` specifically when the supertrait's methods matter and its associated
  types are not named in your signatures, the case where `#[use_type]` has nothing to rewrite.
- **A value from a field** is an [`#[implicit]`](implicit.md) argument.
- **The type itself** is declared with [`#[cgp_type]`](../macros/cgp_type.md) and bound to a concrete type
  by wiring the component to [`UseType<T>`](../providers/use_type.md). The `UseType` *provider* and this
  *attribute* are [two different things](/docs/concepts/abstract-types#two-things-called-usetype) despite
  the shared name: the provider supplies a type to a context, and the attribute imports one into a
  definition.

`#[use_type]` does not import a construct's **own** associated type, so it stays qualified as
`Self::Output`. This rule is worth stating plainly, because breaking it produces a confusing error. The
attribute rewrites only the names it was given, so a local type written bare resolves to nothing. A
mixed signature such as `Result<Self::Output, Error>` is therefore correct and idiomatic: the local type
qualified, the imported one bare.

Prefer an inferred parameter over an abstract type when the type only ever flows through values the
body reads: [`#[impl_generics]`](./impl_generics.md) on a `#[cgp_fn]` is shorter and does not need
wiring. Move to an abstract type when the type must be named in the capability's own signature, or when
two capabilities have to
[agree that they mean the same one](/docs/concepts/abstract-types#one-type-agreed-on-by-everything-that-needs-it).

## Under the hood

`#[use_type]` runs before the surrounding macro: it **grounds** each import's own type positions,
**substitutes** every matching bare identifier, then **adds the bound**.

From this input:

```rust
#[cgp_fn]
#[use_type(HasScalarType.Scalar)]
pub fn area(&self, #[implicit] width: Scalar, #[implicit] height: Scalar) -> Scalar {
    width * height
}
```

the effective result is:

```rust
pub trait Area: HasScalarType {
    fn area(&self) -> <Self as HasScalarType>::Scalar;
}

impl<__Context__> Area for __Context__
where
    Self: HasField<Symbol!("width"), Value = <Self as HasScalarType>::Scalar>
        + HasField<Symbol!("height"), Value = <Self as HasScalarType>::Scalar>,
    Self: HasScalarType,
{
    fn area(&self) -> <Self as HasScalarType>::Scalar { /* ... */ }
}
```

**Where the bound lands depends on the target.** When the type is projected from `Self` it becomes a
*supertrait* of the generated trait, and also a `where` bound on the implementation. When an `in Context`
clause names some other type, a supertrait of `Self` would be wrong, so the macro adds a plain
`Context: Trait` predicate. That predicate goes on the implementation. On `#[cgp_fn]` and
`#[cgp_component]` it goes on the generated trait's own `where` clause as well, because the trait's
signatures now mention the projection and would not otherwise be well-formed. A plain unbounded `<Types>`
on your function is therefore enough:

```rust
pub trait AreaOf<Types>
where
    Types: HasScalarType,
{
    fn area_of(&self) -> <Types as HasScalarType>::Scalar;
}
```

An equality pin is the exception: it stays on the implementation, and the macro never adds it to the trait.

**What substitution touches.** The rewrite matches a single-segment type path with no arguments whose
identifier is an imported name or alias, anywhere it appears: a return type, an argument, a `where`
predicate, a local binding. It also reaches an alias used to *qualify* a path, so `Transaction::begin()`
becomes `<<Self as HasTransactionType>::Transaction>::begin()`, because `<Self as Trait>::Assoc::method` is
not valid syntax.

The rewrite leaves one position alone: a **bare alias in expression position**, which names a value,
something an abstract type can never be. So an alias sharing its name with a unit struct still constructs
the struct:

```rust
let value = Marker;            // stays the unit struct `Marker`
let typed: Marker = todo!();   // becomes <Self as HasMarkerType>::Marker
```

**Grounding, and why order does not matter.** The macro resolves each import against the imports it
*depends on* rather than the ones written before it, so every arrangement of the same entries produces
the same substitutions and the same bounds. Only the order the bounds are listed in follows the source. A
chain like `#[use_type(HasC<B>.C in B, HasB<A>.B in A, HasA.A)]` grounds exactly as its front-to-back
spelling does, and two imports may share one target as readily as chain through it. A **cycle** is the
only arrangement without a valid order. See [Common Mistakes](#common-mistakes).

## Formal grammar

The tokens inside `#[use_type(...)]`, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
UseTypeArgs  -> UseTypeSpec ( `,` UseTypeSpec )* `,`?

UseTypeSpec  -> TraitPath `.` TypeItems ( `in` ContextPath )?

ContextPath  -> TypePath
TraitPath    -> TypePath

TypeItems    -> UseTypeIdent
              | `{` UseTypeIdent ( `,` UseTypeIdent )* `,`? `}`

UseTypeIdent -> IDENTIFIER ( `as` IDENTIFIER )? ( `=` Type )?
```

`TraitPath` and `ContextPath` are ordinary Rust `TypePath`s, whose final segment may carry generic
arguments. Their `::` segments belong to the path, while the `.` after the trait begins the
associated-type list. An omitted `in ContextPath` defaults the target to `Self`. In each `UseTypeIdent`
the leading `IDENTIFIER` is the associated type's own name, `as` gives it a local alias to write in
signatures, and `= Type` pins it with an equality bound. The pin is accepted on `#[cgp_fn]` and
`#[cgp_impl]` and rejected on `#[cgp_component]`.

## Common Mistakes

**Two imports may not share a name or alias.** The substitution could only pick one, so the macro
rejects a collision rather than resolving it:

```text
error: Multiple abstract types cannot share the same identifier or alias
```

**Imports may not resolve through one another in a cycle.** An `in` clause or a trait argument may name
another import only if the resulting chain is acyclic, and the message names the loop:

```text
error: cannot ground `#[use_type]` imports: they resolve through one another in a cycle `B` -> `A` -> `B`.
An `in Context` clause or a trait argument may name another import's alias only if the resulting chain is
acyclic, since a cycle has no valid grounding order.
```

A self-referential import such as `#[use_type(HasAType.A in A)]` is the degenerate case and reports the
same way.

**An equality pin is rejected on a component trait**, because a trait definition cannot carry an
implementation-side constraint:

```text
error: Type equality constraints cannot be used in component trait definition
```

Pin the type on the provider with [`#[cgp_impl]`](../macros/cgp_impl.md) instead, or on a
[`#[cgp_fn]`](../macros/cgp_fn.md).

**A construct's own associated type must stay qualified.** Writing a local `type Output` as a bare
`Output` leaves an identifier that the substitution has no entry for, so the compiler cannot resolve it.
Write `Self::Output`, and do not list it in a `#[use_type]` attribute.

## Related constructs

- [`#[cgp_type]`](../macros/cgp_type.md) — declares the abstract-type component this imports from.
- [`UseType`](../providers/use_type.md) — the provider a context wires to supply the concrete type; a
  different thing from this attribute despite the name.
- [`HasType`](../components/has_type.md) — the built-in abstract-type component underneath.
- [`#[uses]`](uses.md) — imports a capability rather than a type.
- [`#[extend]`](extend.md) — adds a supertrait without rewriting any type names.
- [`#[implicit]`](implicit.md) — imports a value from a field.
- [`#[cgp_fn]`](../macros/cgp_fn.md), [`#[cgp_impl]`](../macros/cgp_impl.md), and
  [`#[cgp_component]`](../macros/cgp_component.md) — the three hosts.

The ideas behind it:

- [Abstract types](/docs/concepts/abstract-types) — the idea behind the types this attribute imports,
  from [choosing the type by wiring](/docs/concepts/abstract-types#choosing-the-type-by-wiring) to
  [what an abstract type costs](/docs/concepts/abstract-types#what-it-costs).
- [Modular error handling](/docs/concepts/modular-error-handling) — the error type is the abstract type
  this attribute imports most, treated there as one of three independent wiring choices.

## Source

- Parsing: [`types/attributes/use_type/attribute.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/use_type/attribute.rs)
- Grounding, substitution and bounds: [`types/attributes/use_type/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/attributes/use_type/)
- The substitution pass: [`visitors/substitute_abstract_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/visitors/substitute_abstract_type.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
