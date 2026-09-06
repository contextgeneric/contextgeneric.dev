---
sidebar_label: '#[impl_generics]'
sidebar_position: 7
---

# `#[impl_generics(...)]`

Declare a generic parameter on the generated implementation alone, so a context supplies the type
through a field and callers never name it.

## Overview

`#[impl_generics(...)]` declares a generic parameter on the implementation that a
[`#[cgp_fn]`](../macros/cgp_fn.md) generates, and keeps it off the generated trait. By default,
`#[cgp_fn]` puts every generic parameter you write on the function onto both the trait and its
implementation. That is right for a type the caller chooses. But a body often needs a type that nobody
chooses: a database handle, a name that only has to be printable, a scalar read from a field. The
**context**, the type the capability runs against and the owner of the fields the body reads, already
fixes that type through the field. A parameter on the trait would then make every caller, and every
capability built on this one, declare a parameter and repeat its bounds for a type they never touch.

With `#[impl_generics]`, the trait stays free of parameters, and the compiler infers the parameter from
the field the body reads:

```rust
#[cgp_fn]
#[impl_generics(Name: Display)]
pub fn greet(&self, #[implicit] name: &Name) -> String {
    format!("Hello, {name}!")
}
```

The `Greet` trait is not generic. A context qualifies by carrying a `name` field of some type that
implements `Display`, and the compiler resolves `Name` from that field's type. The context does not need
wiring or any declaration beyond the field itself.

The cost is that the type is hidden rather than named. It exists only where a value of it flows
through an [`#[implicit]`](./implicit.md) argument, so nothing else can refer to it: not this
capability's own signature, not another capability, and not a second provider. When the type must be
named, promote it to an abstract type with [`#[cgp_type]`](../macros/cgp_type.md) and import it with
[`#[use_type]`](./use_type.md). [When to use it](#when-to-use-it) says where that boundary lies.

## Usage

Write the attribute beneath `#[cgp_fn]` and pass a comma-separated list of generic parameters, each
written as it would be inside an `impl<...>` list:

```rust
#[cgp_fn]
#[impl_generics(Name: Display, Count: Display)]
pub fn describe(&self, #[implicit] name: &Name, #[implicit] count: &Count) -> String {
    format!("{count} × {name}")
}
```

Each parameter joins the implementation's generic list, after the context parameter and after any
generics the function declares itself. A bound written inline, as `Name: Display` above, stays with
the parameter. A bound may also go in the function's own `where` clause, which lands on the
implementation as well. You may repeat the attribute, and the macro concatenates the lists, but one
attribute with a comma-separated list reads as one declaration and is the form to prefer.

**Every parameter must be pinned by the type of an implicit argument.** The compiler accepts a
parameter on an impl only where the impl determines it, and the only place a `#[cgp_fn]`
implementation determines one is the field bound an `#[implicit]` argument produces. `&Name` pins
`Name`, and a nested position such as `&Pool<Db>` pins `Db`. The compiler rejects a parameter that does
not appear in any implicit argument with `E0207`. See [Common Mistakes](#common-mistakes).

The list accepts a lifetime or a const parameter as well as a type parameter, because the macro reads
ordinary generic parameters. Type parameters are the case the attribute exists for.

**Only [`#[cgp_fn]`](../macros/cgp_fn.md) reads `#[impl_generics]`.** A
[`#[cgp_impl]`](../macros/cgp_impl.md) block's own generic list is already impl-only, so a provider
declares such a parameter there directly. A [`#[cgp_component]`](../macros/cgp_component.md) does not
have an implementation of its own to carry one.

## Examples

A capability whose one type dependency each context fixes through a field:

```rust
use cgp::prelude::*;
use core::fmt::Display;

#[cgp_fn]
#[impl_generics(Name: Display)]
pub fn greet(&self, #[implicit] name: &Name) -> String {
    format!("Hello, {name}!")
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

#[derive(HasField)]
pub struct Robot {
    pub name: u32,
}

pub fn greet_both(person: &Person, robot: &Robot) {
    println!("{}", person.greet()); // Hello, Alice!
    println!("{}", robot.greet()); // Hello, 42!
}
```

`Person` and `Robot` both implement `Greet` through the one blanket implementation, without any
wiring. For `Person` the compiler resolves `Name` to `String`, and for `Robot` to `u32`. Neither type
appears anywhere except in the field.

A capability built on `greet` never learns that `Name` exists:

```rust
#[cgp_fn]
#[uses(Greet)]
pub fn announce(&self) -> String {
    format!("{} Welcome aboard.", self.greet())
}
```

Had `greet` taken `Name` as a function generic instead, `announce` would have to declare `<Name>`,
repeat `Name: Display`, and pass the parameter on to everything that calls it.

## When to use it

**Start with `#[impl_generics]` for a type that only ever flows through implicit arguments.** It is
the shortest form, it does not need wiring, and it reads as "this works with any `name` field of a
compatible type". You must move to an abstract type once the type has to be *named* somewhere the
inferred form cannot reach.

- **The type appears in the capability's signature.** An impl-only parameter is not in scope on the
  trait, so a return type or an explicit parameter cannot mention it. This condition applies as soon as
  a capability returns a value of the type to its caller.
- **Two capabilities must agree on the type.** A transaction type only means something relative to
  its database, so the capability that opens one and the capability that commits it must mean the
  same type. Each implementation infers its own parameter, so inferred parameters cannot state the
  agreement.

In both cases declare the type with [`#[cgp_type]`](../macros/cgp_type.md), import it with
[`#[use_type]`](./use_type.md), and let the context supply it by wiring. Do not answer either
condition with a generic parameter on the function. Such a parameter lands on the trait, so every
caller and every intermediate capability must declare it and repeat its bounds whether they touch it
or not. It also puts the decision in the wrong place: `<Db>` on a trait says the caller chooses the
database type, though the application determines it.

Some neighbours cover what this attribute is not for.

- **A bound on `Self`** is a capability dependency; write it with [`#[uses]`](./uses.md).
- **A bound on one of the trait's own parameters** that callers must see is
  [`#[extend_where]`](./extend_where.md)'s job, on the trait side.
- **A type the caller should choose per call site** is a plain generic parameter on the function.

## Under the hood

The macro emits the trait without the parameter and the implementation with it. From the `greet`
input above:

```rust
pub trait Greet {
    fn greet(&self) -> String;
}

impl<__Context__, Name: Display> Greet for __Context__
where
    Self: HasField<Symbol!("name"), Value = Name>,
{
    fn greet(&self) -> String {
        let name: &Name = self.get_field(PhantomData::<Symbol!("name")>);
        format!("Hello, {name}!")
    }
}
```

The `Value = Name` binding makes the implementation legal. Rust accepts a parameter on an impl only
when the self type, the trait, or an associated-type binding in the `where` clause determines it,
and here the field bound from the implicit argument is that binding. The bound on `Name` travels with
the parameter, exactly as written.

The parameter list is ordered. The context comes first, then any generics the function declared,
then the `#[impl_generics]` parameters, so a function `fn scale<Scalar>` with `#[impl_generics(Db)]`
emits `impl<__Context__, Scalar, Db>`. Rust requires lifetimes to lead a generic list, and the
emitted list keeps that rule whatever order the parameters were declared in.

The `where` clause is ordered too: the function's own predicates first, then the bounds the
companion attributes contribute, then the `HasField` bounds from the implicit arguments, always last.
The parameter's name is yours. The context, by contrast, is literally `__Context__` in the emitted
code and appears as `Self` inside the implementation.

## Formal grammar

The attribute argument is a comma-separated list of generic parameters, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
ImplGenericsArgs -> GenericParam ( `,` GenericParam )* `,`?
```

`GenericParam` is the Rust grammar's own production for one entry of a generic parameter list: a
lifetime, a type parameter with optional bounds, or a const parameter. The list may be empty, and the
attribute may be repeated. Because the production is Rust's, a parameter default such as `T = u32`
parses, and the compiler then rejects it. See [Common Mistakes](#common-mistakes).

## Common Mistakes

**A parameter must be pinned by an implicit argument, or it is unconstrained.** The trait does not
mention it and neither does the self type, so only a field bound can determine it. Declaring `Name`
without reading a field whose type mentions `Name` fails at the implementation:

```rust
#[cgp_fn]
#[impl_generics(Name: Display)]
pub fn greet(&self) -> String {
    "Hello!".to_owned()
}
```

```text
error[E0207]: the type parameter `Name` is not constrained by the impl trait, self type, or predicates
```

Either read a field whose type mentions the parameter, or, if the caller should choose the type, make
it a generic parameter on the function instead.

**A parameter cannot appear in the capability's signature.** Only the implementation declares it, so a
return type or an explicit parameter that names it refers to nothing on the trait:

```rust
#[cgp_fn]
#[impl_generics(Db: Database)]
pub fn fetch_row(&self, #[implicit] database: &Pool<Db>) -> Db::Row { /* ... */ }
```

```text
error[E0433]: cannot find type `Db` in this scope
   |
   | pub fn fetch_row(&self, #[implicit] database: &Pool<Db>) -> Db::Row {
   |                                                             ^^ use of undeclared type `Db`
```

The `&Pool<Db>` argument is fine, because the macro strips it into a field bound on the
implementation, where `Db` is in scope. The `Db::Row` return type stays on the trait, and fails. A
bare `Db` in the signature reports the same headline under `E0425` instead, so search for both
codes. You must decide the fix, because the macro cannot: promote the type to an
[abstract type](../macros/cgp_type.md) so it can be named everywhere, or keep `#[impl_generics]` and
stop naming it. The [compile errors](../errors.md#a-name-the-generated-code-cannot-see) page reads
the full diagnostic.

**A parameter default parses and then fails.** `#[impl_generics(T = u32)]` is a valid
`GenericParam`, so the macro accepts it and copies it onto the implementation, where Rust forbids
defaults:

```text
error: defaults for generic parameters are not allowed here
```

Drop the default. An implementation infers its parameters and cannot use one.

**A parameter named after the function shadows the generated trait.** The trait takes the function's
name in PascalCase, so `fn count` with `#[impl_generics(Count: Display)]` puts a trait `Count` and a
parameter `Count` in the same scope, and inside the generated impl the trait's name resolves to the
parameter:

```text
error[E0404]: expected trait, found type parameter `Count`
   |
   | #[impl_generics(Count: Display)]
   |                 ----- found this type parameter
   | pub fn count(&self, #[implicit] count: &Count) -> String {
   |        ^^^^^ not a trait
```

Rename the parameter, or give the trait another name with `#[cgp_fn(CanCount)]`.

**On any host other than `#[cgp_fn]` the compiler does not recognize the attribute**, rather than
accepting and ignoring it. The macro does not consume it, so it reaches the compiler as an attribute
that does not exist:

```rust
#[cgp_impl(new GreetHello)]
#[impl_generics(Name: Display)]
impl Greeter {
    fn greet(&self, #[implicit] name: &Name) -> String {
        format!("Hello, {name}!")
    }
}
```

```text
error: cannot find attribute `impl_generics` in this scope
```

On a [`#[cgp_component]`](../macros/cgp_component.md) the error repeats once per item the macro
generates. On a [`#[cgp_impl]`](../macros/cgp_impl.md) block, delete the attribute and declare the
parameter in the block's own generic list, which is already impl-only.

## Related constructs

- [`#[implicit]`](./implicit.md) — the argument whose field type pins the parameter.
- [`#[cgp_fn]`](../macros/cgp_fn.md) — the only host, and where the generics split is explained.
- [`#[extend_where]`](./extend_where.md) — the trait-side sibling: a predicate callers must see.
- [`#[uses]`](./uses.md) — a capability bound on `Self`, the other kind of private requirement.
- [`#[use_type]`](./use_type.md) and [`#[cgp_type]`](../macros/cgp_type.md) — the abstract-type
  form to move to when the type must be named.
- [`HasField`](../traits/field-access/has_field.md) — the bound that carries the inference.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — why a requirement carried on the
  implementation never reaches a caller.
- [Abstract types](/docs/concepts/abstract-types) — the other home for a type dependency.

## Source

- Parsing: [`types/attributes/function.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/function.rs)
- Insertion into the implementation's generic list: [`types/cgp_fn/preprocessed.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_fn/preprocessed.rs)
- Expansion snapshot: [`generic_components/fn_impl_generics.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/tests/cgp-tests/tests/generic_components/fn_impl_generics.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
