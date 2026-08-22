---
sidebar_label: '#[use_type]'
---

# `#[use_type]`

Import an abstract associated type and write it as a bare name.

## Overview

Generic CGP code constantly needs a type it does not choose — an error type, a scalar, a database handle —
supplied by the **context**, the type the code runs against. Such a type lives as an
[associated type](https://doc.rust-lang.org/reference/items/associated-items.html) on another trait, and
Rust requires every mention of it to be written out in full:

```rust
fn load(&self, path: &str) -> Result<String, <Self as HasErrorType>::Error>;
```

That is correct and almost unreadable, and it has to be repeated at every occurrence — the return type,
each argument, each local binding. `#[use_type]` imports the type once so the signature can name it
plainly:

```rust
#[use_type(HasErrorType.Error)]
```

With that in place the same method reads `-> Result<String, Error>`, and the macro expands each bare
`Error` back into the qualified path while adding the bound that makes it available. The bare name behaves
like an ordinary type in your source and resolves to the associated type in the output.

Reading better is the obvious benefit; the less obvious one is that the qualified form composes where a
bare one cannot. Because the macro always emits `<Context as Trait>::Type`, one import can be defined in
terms of another, a type can be pulled from a parameter rather than from `Self`, and two abstract types can
be constrained to be the same — none of which you can express by writing a bare identifier and hoping.
Those are the [advanced forms](#importing-from-another-type) below.

## Usage

Apply `#[use_type]` beside the macro it modifies, naming a trait and the associated type to import:

```rust
#[use_type(HasScalarType.Scalar)]
```

The part before the `.` is the trait and the identifier after it is the type. The **separator is a dot,
not `::`**, and that is deliberate: it leaves `::` free for the trait's own path, so
`#[use_type(errors::HasErrorType.Error)]` imports from a trait named by path and
`#[use_type(HasFooType<X>.Foo)]` imports from a particular generic instantiation.

By default the type is projected from `Self`, so `Scalar` expands to `<Self as HasScalarType>::Scalar`.

The attribute is accepted on [`#[cgp_fn]`](../macros/cgp_fn.md),
[`#[cgp_impl]`](../macros/cgp_impl.md), and [`#[cgp_component]`](../macros/cgp_component.md).

### Importing several types

Separate entries with commas, and **prefer one attribute carrying the whole list**, since it reads as a
single import list:

```rust
#[use_type(HasUserIdType.UserId, HasCurrencyType.Currency, HasErrorType.Error)]
```

Several types from the same trait go in a braced group, and any entry may be renamed with `as` to give it
a local alias:

```rust
#[use_type(HasFooType.{Foo, Bar as Baz})]
```

Stacking several `#[use_type]` attributes behaves identically, because every attribute's entries are
collected into one list before any of them is resolved — so an alias declared in one is available to
another regardless of which comes first. Reach for a second attribute only when there is a reason.

### Pinning a type to a concrete one

An `= Type` clause imports the type *and* constrains it, which is how a provider says "this only works when
the error type is `AppError`":

```rust
#[use_type(HasErrorType.{Error = AppError})]
```

That emits `Self: HasErrorType<Error = AppError>` in place of the plain bound. The right-hand side is
itself substituted, so it may name another import — which is how two abstract types are tied together:

```rust
#[use_type(HasPasswordType.Password, HasHashedPasswordType.{HashedPassword = Password})]
```

The pin is an implementation-side constraint, so it is **rejected on
[`#[cgp_component]`](../macros/cgp_component.md)**, whose trait definition has nowhere to put it.

### Importing from another type

A trailing `in Context` clause changes what the type is projected from, so an import can reach a type on a
parameter rather than on `Self`:

```rust
#[use_type(HasScalarType.Scalar in Types)]
```

`Scalar` now expands to `<Types as HasScalarType>::Scalar`. The clause scopes over the whole entry, so a
braced group projects every type in it against the same target, and because `in` is a reserved word it can
never be mistaken for a type name.

This is also the form that lets an import parameterize the trait it comes from. An alias may appear in
another entry's `in` clause *or* in its trait arguments, and the macro resolves the chain for you:

```rust
#[use_type(HasDbType.Db, HasPoolType<Db>.Pool)]
```

Here `Pool` is projected from `HasPoolType<<Self as HasDbType>::Db>`. Such chains may be written in any
order — see [Under the hood](#under-the-hood) — provided they do not form a cycle.

## Examples

One abstract scalar threaded through a component and a provider, with neither writing `Self::` anywhere:

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

The component gains `HasScalarType` as a supertrait and the provider gains it as a bound, and every
`Scalar` in both becomes the same qualified projection — so the fields the provider reads and the value it
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

The most common use of all is an error type, where the import saves a projection in every fallible
signature:

```rust
#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}
```

## When to reach for it, and when not

**Use `#[use_type]` whenever a definition names an abstract type another component supplies.** It is the
recommended form, and a hand-written supertrait plus `Self::`-qualified paths is what to read rather than
write.

Two neighbours cover requirements that are not types, and one construct is what `#[use_type]` imports
*from*.

- **A capability** is [`#[uses]`](uses.md) for a private bound or [`#[extend]`](extend.md) for a
  supertrait. Reach for `#[extend]` specifically when the supertrait's methods are what matter and its
  associated types are not named in your signatures — that is the case `#[use_type]` has nothing to
  rewrite.
- **A value from a field** is an [`#[implicit]`](implicit.md) argument.
- **The type itself** is declared with [`#[cgp_type]`](../macros/cgp_type.md) and bound to a concrete type
  by wiring the component to [`UseType<T>`](../providers/use_type.md). Note that the `UseType` *provider*
  and this *attribute* are different things despite the shared name: the provider supplies a type to a
  context, and the attribute imports one into a definition.

There is one boundary worth stating plainly, because getting it wrong produces a confusing error. A
construct's **own** associated type is not imported and stays qualified as `Self::Output`. `#[use_type]`
rewrites only the names it was given, so a local type written bare resolves to nothing. A mixed signature
such as `Result<Self::Output, Error>` is therefore correct and idiomatic — the local type qualified, the
imported one bare.

Finally, prefer an inferred parameter over an abstract type when the type only ever flows through values
the body reads: [`#[impl_generics]`](../macros/cgp_fn.md) on a `#[cgp_fn]` is shorter and needs no wiring.
Climb to an abstract type when the type must be named in the capability's own signature, or when two
capabilities have to agree that they mean the same one.

## Under the hood

:::note

### Advanced

This section shows the three steps the attribute performs and what each one touches. You do not need it to
use `#[use_type]`, but the substitution has edges worth knowing, and an unmet import names the generated
bound rather than your import. `cargo cgp expand` prints the same thing for your own code.

:::

`#[use_type]` runs before the surrounding macro, in three steps: it **grounds** each import's own type
positions, **substitutes** every matching bare identifier, then **adds the bound**.

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
`Context: Trait` predicate — to the implementation, and on `#[cgp_fn]` and `#[cgp_component]` to the
generated trait's own `where` clause as well, since the trait's signatures now mention the projection and
would not otherwise be well-formed. A plain unbounded `<Types>` on your function is therefore enough:

```rust
pub trait AreaOf<Types>
where
    Types: HasScalarType,
{
    fn area_of(&self) -> <Types as HasScalarType>::Scalar;
}
```

An equality pin is the exception: it stays on the implementation and is never added to the trait.

**What substitution touches.** The rewrite matches a single-segment type path with no arguments whose
identifier is an imported name or alias, anywhere it appears — a return type, an argument, a `where`
predicate, a local binding. It also reaches an alias used to *qualify* a path, so `Transaction::begin()`
becomes `<<Self as HasTransactionType>::Transaction>::begin()`, since `<Self as Trait>::Assoc::method` is
not valid syntax.

The one position left alone is a **bare alias in expression position**, which names a value — something an
abstract type can never be. So an alias sharing its name with a unit struct still constructs the struct:

```rust
let value = Marker;            // stays the unit struct `Marker`
let typed: Marker = todo!();   // becomes <Self as HasMarkerType>::Marker
```

**Grounding, and why order does not matter.** Each import is resolved against the imports it *depends on*
rather than the ones written before it, so every arrangement of the same entries produces the same
substitutions and the same bounds. Only the order the bounds are listed in follows the source. A chain like
`#[use_type(HasC<B>.C in B, HasB<A>.B in A, HasA.A)]` grounds exactly as its front-to-back spelling does,
and two imports may share one target as readily as chain through it. What has no valid order is a
**cycle** — see the Gotchas.

<details>
<summary>Formal grammar</summary>

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
arguments; their `::` segments belong to the path, while the `.` after the trait begins the
associated-type list. An omitted `in ContextPath` defaults the target to `Self`. In each `UseTypeIdent`
the leading `IDENTIFIER` is the associated type's own name, `as` gives it a local alias to write in
signatures, and `= Type` pins it with an equality bound — accepted on `#[cgp_fn]` and `#[cgp_impl]`,
rejected on `#[cgp_component]`.

</details>

## Gotchas

**Two imports may not share a name or alias.** The substitution could only pick one, so a collision is
rejected rather than resolved:

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

**An equality pin is rejected on a component trait**, since a trait definition cannot carry an
implementation-side constraint:

```text
error: Type equality constraints cannot be used in component trait definition
```

Pin the type on the provider with [`#[cgp_impl]`](../macros/cgp_impl.md) instead, or on a
[`#[cgp_fn]`](../macros/cgp_fn.md).

**A construct's own associated type must stay qualified.** Writing a local `type Output` as a bare
`Output` leaves an identifier the substitution has no entry for, so it resolves to nothing. Write
`Self::Output`, and do not list it in a `#[use_type]` attribute.

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

- [Abstract types](/docs/concepts/abstract-types) — the idea behind the types this attribute
  imports.

## Source

- Parsing: [`types/attributes/use_type/attribute.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/use_type/attribute.rs)
- Grounding, substitution and bounds: [`types/attributes/use_type/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/attributes/use_type/)
- The substitution pass: [`visitors/substitute_abstract_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/visitors/substitute_abstract_type.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
