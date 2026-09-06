---
sidebar_label: '#[cgp_fn]'
sidebar_position: 3
---

# `#[cgp_fn]`

Define a single-implementation capability as a blanket-impl trait, straight from a function.

## Overview

`#[cgp_fn]` is the simplest CGP construct that does anything useful. You write a plain function and
mark the values it needs from the **context**. The context is the type the capability runs against,
and it supplies those values as its own fields. The macro turns the function into a capability that
every type with those fields gets automatically:

```rust
#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}
```

Any struct with a `width` and a `height` can now call `.rectangle_area()`. There is no component to
define, no provider to name, and no wiring table anywhere. The macro emits a trait and one
[blanket implementation](https://blog.implrust.com/posts/2025/09/blanket-implementation-in-rust/)
covering every context that satisfies the field requirements.

That is the tradeoff it makes, and it is worth being explicit about. A
[`#[cgp_component]`](./cgp_component.md) supports many interchangeable implementations, one chosen per
context, and this costs extra boilerplate. `#[cgp_fn]` supports exactly one implementation, the function
body, and costs nothing. For the large share of capabilities that have one natural definition, that is
the better choice, which is why `#[cgp_fn]` is the recommended place to start rather than a lesser form
of the real thing.

It is also the easiest introduction to CGP, because nothing in it is unfamiliar. A reader who
understands functions and arguments can write a working capability without first meeting trait bounds,
blanket implementations, or type-level field names. An [`#[implicit]`](../attributes/implicit.md)
parameter hides all of that and looks like an ordinary parameter.

## Usage

Apply the attribute to a free function. The function name becomes the method name, and its PascalCase
form becomes the trait name: `rectangle_area` generates a `RectangleArea` trait with a
`rectangle_area` method.

Almost every `#[cgp_fn]` takes `self` as its first parameter, because reading a field from the context
is most of what these functions do, and a receiver is *required* the moment any parameter is
`#[implicit]`. Omitting it is accepted, and produces a trait whose item is an associated function
rather than a method; it is rarely what you want here, since a function that reads nothing from its
context computes the same answer for every context that gets it.

Pass an identifier to override the trait name, which is useful when a verb-style name reads better
than the function's:

```rust
#[cgp_fn(CanCalculateRectangleArea)]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}
```

The receiver may be `&self` or `&mut self`. A `&mut self` function can take a mutable implicit
argument and write through it.

### Implicit arguments

An [`#[implicit]`](../attributes/implicit.md) parameter is removed from the method's signature and
filled from a same-named field on the context instead. The parameter's *name* is the field name, and
its *type* decides how the field is read: an owned type is cloned out, a `&str` is borrowed from a
`String` field, and a plain `&T` is borrowed with no conversion. The full set of forms, including
options, slices, and the mutable variants, is on the [`#[implicit]`](../attributes/implicit.md) page.

Because these parameters disappear from the signature, `rectangle_area` above is called with no
arguments at all: `rect.rectangle_area()`.

### Generics, and where each bound lands

A `#[cgp_fn]` splits its generics and its `where` clause deliberately, and knowing which goes where is
most of what there is to learn about the macro.

| You write | It lands on |
|---|---|
| A generic parameter on the function | Both the trait and the impl |
| A `where` clause on the function | The impl only |
| [`#[impl_generics(T: Bound)]`](#a-type-the-caller-should-not-name) | The impl only |
| [`#[extend_where(...)]`](../attributes/extend_where.md) | Both the trait and the impl |

By default, generics land on both the trait and the impl, and the `where` clause lands on the impl
alone. This keeps a capability's requirements out of sight of its callers: a caller bounds on the
clean trait, and the constraints the body actually needs stay one level down on the implementation:

```rust
#[cgp_fn]
pub fn scale<Scalar>(&self, #[implicit] factor: Scalar) -> Scalar
where
    Scalar: Mul<Output = Scalar> + Copy,
{
    factor * factor
}
```

`Scale<Scalar>` is the trait; `Scalar: Mul<Output = Scalar> + Copy` appears only on the impl.

#### A type the caller should not name

When the function needs a type that should be inferred rather than chosen, such as a database handle
or a displayable value, declare it with `#[impl_generics(...)]`. The parameter goes on the impl alone, so
the trait stays unparameterized and no caller mentions it:

```rust
#[cgp_fn]
#[impl_generics(Name: Display)]
pub fn describe(&self, #[implicit] name: &Name) -> String {
    format!("{name}")
}
```

`Describe` has no type parameter. A context becomes eligible purely by carrying a `name` field of some
`Display` type, and the type is resolved from that field.

The cost is that the type is concealed rather than named. It exists only where a value of it flows
through an implicit argument, so nothing else can refer to it, including this function's own
signature. Naming it in a return type or an explicit parameter does not compile, as the
[Common Mistakes](#common-mistakes) explain. When the type must be nameable, or when two capabilities have to agree
that they mean the same type, promote it to an abstract type with [`#[cgp_type]`](./cgp_type.md) and
import it with [`#[use_type]`](../attributes/use_type.md).

### Companion attributes

Several attributes shape what the macro generates, and together they are how a `#[cgp_fn]` states what
it depends on.

- [`#[uses(...)]`](../attributes/uses.md) imports the capabilities the body calls on `self`, reading
  like a `use` statement rather than a hand-written `where Self: Trait` bound. It accepts ordinary Rust
  traits as readily as CGP ones.
- [`#[use_type(Trait.Type)]`](../attributes/use_type.md) imports an abstract type so the signature can
  name it bare, and adds the owning trait as a supertrait.
- [`#[extend(...)]`](../attributes/extend.md) adds a capability supertrait to the generated trait. Here
  it is the *only* way to add one, since a `where` clause in a `#[cgp_fn]` is an implementation detail
  rather than part of the interface.
- [`#[extend_where(...)]`](../attributes/extend_where.md) puts a predicate on the generated trait's own
  `where` clause, for a bound callers need to see.
- [`#[use_provider(...)]`](../attributes/use_provider.md) completes an inner provider's bound when the
  function delegates to one.
- [`#[async_trait]`](./async_trait.md) goes directly beneath `#[cgp_fn]` on an `async fn`. It is copied
  onto both generated items, so the trait ends up declaring a lint-clean `-> impl Future`.

## Examples

Two capabilities, the second built on the first, and a context that gets both without wiring anything:

```rust
use cgp::prelude::*;

#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}

#[cgp_fn]
#[uses(RectangleArea)]
pub fn scaled_rectangle_area(&self, #[implicit] scale_factor: f64) -> f64 {
    self.rectangle_area() * scale_factor * scale_factor
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

pub fn report(rect: &Rectangle) {
    println!("base area   = {}", rect.rectangle_area());
    println!("scaled area = {}", rect.scaled_rectangle_area());
}
```

`scaled_rectangle_area` calls `self.rectangle_area()` because `#[uses(RectangleArea)]` declared the
dependency; it does not know or care how that capability is implemented. `Rectangle` derives
[`HasField`](../derives/derive_has_field.md) and happens to carry the three fields the two functions
read, and that is its entire qualification. There is no `delegate_components!` anywhere in this
program, and adding one would change nothing.

## When to use it

**Use `#[cgp_fn]` first.** When a capability has one natural definition, this is the form to
write, and starting here costs nothing if that changes later: the trait keeps its name and its method,
so promoting it to a [`#[cgp_component]`](./cgp_component.md) leaves every call site untouched. What
you add at that point is the component, a named provider, and a line of wiring per context.

Use something else in these cases:

- **The capability needs a second implementation, chosen per context.**
  [`#[cgp_component]`](./cgp_component.md) is for that, and no amount of `#[cgp_fn]` will get you
  there: its blanket impl already covers every context, so there is nowhere for an alternative to live.
- **The dependencies are traits rather than fields.** [`#[blanket_trait]`](./blanket_trait.md) builds
  the same kind of single-implementation, no-wiring capability from a trait with supertraits and
  default method bodies. Use it when the body needs other capabilities; use `#[cgp_fn]`
  when it needs values.
- **A generic must vary per call.** A generic parameter here goes on the trait rather than the method,
  so it is fixed by whatever satisfies the bounds for a given context rather than chosen at each call
  site. A capability that needs a per-call type parameter wants a hand-written blanket impl
  or a component.

One decision inside the macro is worth making deliberately rather than by default: **where a type the
body needs should live.** Start with `#[impl_generics]` while the type only ever flows through implicit
arguments. It is shorter, needs no wiring, and reads as "this works with any `database` field of a
compatible type". Move up to an [abstract type](./cgp_type.md) when the type must appear in the
capability's own signature, or when two capabilities must agree that they mean the same one. Avoid a
plain generic parameter on the function in both cases: it lands on the trait and makes every caller,
and every intermediate capability built on it, declare the parameter and repeat its bounds whether
they touch it or not.

## Under the hood

`#[cgp_fn]` emits exactly two items: the trait carrying the method, and a blanket impl of it for a
generic context. From this input:

```rust
#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}
```

the macro produces the trait with the implicit parameters stripped from the signature, and the impl
with those parameters turned into [`HasField`](../traits/field-access/has_field.md) bounds and `let` bindings at the
top of the body:

```rust
pub trait RectangleArea {
    fn rectangle_area(&self) -> f64;
}

impl<__Context__> RectangleArea for __Context__
where
    Self: HasField<Symbol!("width"), Value = f64>
        + HasField<Symbol!("height"), Value = f64>,
{
    fn rectangle_area(&self) -> f64 {
        let width: f64 = self
            .get_field(PhantomData::<Symbol!("width")>)
            .clone();
        let height: f64 = self
            .get_field(PhantomData::<Symbol!("height")>)
            .clone();

        width * height
    }
}
```

A few details of the real output are worth recognizing. The context type parameter is literally
`__Context__`, a reserved name chosen so it cannot collide with one of yours, and it is referred to as
`Self` inside the impl. [`Symbol!("width")`](./symbol.md) is a type-level string standing for the field
name. The compiler prints its expanded `Symbol<5, Chars<'w', …>>` form in errors, and
`cargo cgp expand` resugars it back to this. An *owned* implicit argument compiles to a trailing
`.clone()`; a `&str` argument ends in `.as_str()` instead, and a plain `&T` in nothing at all.

The generics split shows up in the same impl. Given the `scale` function above, the generic goes on
both items while the function's `where` bound stays on the impl, ordered *before* the implicit field
bounds. Attribute-contributed predicates always come first, and the implicit ones are appended last:

```rust
pub trait Scale<Scalar> {
    fn scale(&self) -> Scalar;
}

impl<__Context__, Scalar> Scale<Scalar> for __Context__
where
    Scalar: Mul<Output = Scalar> + Copy,
    Self: HasField<Symbol!("factor"), Value = Scalar>,
{
    fn scale(&self) -> Scalar { /* ... */ }
}
```

The companion attributes layer into these same two items. `#[uses(Trait)]` adds a `Self: Trait`
predicate to the impl; `#[extend(Trait)]` adds it to the impl *and* to the trait's supertraits;
`#[extend_where(P)]` adds `P` to both `where` clauses; `#[impl_generics(T: Bound)]` inserts `T: Bound`
into the impl's generic list alone (its argument is a comma-separated list of ordinary generic
parameters, so a lifetime or a const parameter is accepted there too); and
`#[use_type(Trait.Type)]` adds the supertrait and rewrites every bare mention of the type into its
fully qualified form. The macro appends the implicit-argument bounds last, after whatever the
attributes contributed.

A couple of smaller placements are worth knowing because neither is visible in the source you wrote. **The
function's visibility becomes the trait's**, and the impl's method keeps inherited visibility. So
`pub fn rectangle_area` yields `pub trait RectangleArea`, and a private `fn` yields a private trait
usable only in its own module. And **the macro copies an attribute it does not recognize onto both
items**, so `#[allow(...)]`, `#[doc]`, or a doc comment carries onto the trait and the impl alike.

## Formal grammar

The attribute argument is a single optional trait name, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CgpFnArgs -> TraitName?

TraitName -> IDENTIFIER
```

When the argument is omitted, the trait name defaults to the function name converted to PascalCase. The
`#[implicit]` markers on parameters and the companion attributes are separate attributes with grammars
of their own, documented on their own pages.

## Common Mistakes

**A function with implicit arguments must take `self` first.** Without a receiver there is no context
to read a field from, and the macro says so rather than emitting a bound that fails later:

```text
error: The first argument of a function with implicit arguments must be `self`
```

**A mutable implicit argument must be the only one.** Reading a field mutably borrows the whole context
exclusively, so it cannot coexist with any other field read. Immutable implicit arguments carry no such
restriction and combine freely, in any number:

```text
error: a `&mut` implicit argument must be the only implicit argument, since its mutable
       borrow of the context conflicts with reading any other field
```

**An `#[impl_generics]` parameter cannot appear in the capability's own signature.** Only the generated
impl declares it, so naming it in a return type or an explicit parameter leaves it unresolved in the
trait. Both spellings report the same headline:

```text
error[E0425]: cannot find type `Db` in this scope
```

The error code differs, and that is the part worth noticing, since you are likely to search for it. A
bare `Db` reports `E0425` as above; a qualified path such as `Db::Row` reports `E0433`, and the
compiler labels its span *use of undeclared type `Db`* instead. Both mean the same thing: either the
type belongs out of the signature, or it needs to be an [abstract type](./cgp_type.md) rather than an
inferred impl parameter.

## Related constructs

- [`#[implicit]`](../attributes/implicit.md) — fills a parameter from a context field.
- [`#[uses]`](../attributes/uses.md), [`#[use_type]`](../attributes/use_type.md),
  [`#[extend]`](../attributes/extend.md), [`#[extend_where]`](../attributes/extend_where.md), and
  [`#[use_provider]`](../attributes/use_provider.md) — the attributes that shape what it generates.
- [`#[cgp_component]`](./cgp_component.md) — the step up, for when one implementation is not enough.
- [`#[blanket_trait]`](./blanket_trait.md) — the same idea built from a trait rather than a function.
- [`#[cgp_impl]`](./cgp_impl.md) — shares the `#[implicit]` mechanism, for writing a component's provider.
- [`#[derive(HasField)]`](../derives/derive_has_field.md) — what a context derives to satisfy the bounds.
- [`#[async_trait]`](./async_trait.md) — how an `async fn` capability is declared.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — the `where`-clause injection
  this macro is built on.
- [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) — the hierarchy this sits at the bottom
  of.

## Source

- Entry point: [`cgp_fn.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_fn.rs)
- Implementation: [`types/cgp_fn/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_fn/)
- Implicit arguments: [`types/implicits/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/implicits/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
