---
sidebar_label: '#[extend_where]'
sidebar_position: 6
---

# `#[extend_where]`

Add `where` predicates to a generated trait's own definition, not just to its implementation.

## Overview

`#[extend_where]` moves a `where` predicate onto the trait that a [`#[cgp_fn]`](../macros/cgp_fn.md)
generates, so the predicate becomes a condition of naming the trait at all. By default, `#[cgp_fn]`
treats the `where` clause you write on the function as an implementation detail: the bounds land on the
generated implementation and never appear on the generated trait. That default keeps a capability's
requirements private to its implementation, and it is almost always right.

The cost of that default is that an unsatisfiable requirement becomes invisible. A bound on the
implementation only decides which **contexts** the implementation covers. (A context is the type the
capability runs against, and it supplies the values the capability needs as its fields.) So the compiler
accepts a caller that names the capability for a type that can never satisfy it. The bound stays
unproven, and the error appears later, at a different place. `#[extend_where]` puts the predicate where
the compiler checks it as soon as the trait is named:

```rust
#[extend_where(Scalar: Clone)]
```

Promoting a predicate does **not** give it to callers as a guarantee, and this distinction is easy to
reverse. A trait's `where` clause is a precondition callers must prove, not a guarantee they receive. So a
caller that names the trait still writes the bound itself, and the compiler reports a missing bound
against the trait instead of deferring it.

`#[extend_where]` is the `where`-clause sibling of [`#[extend]`](extend.md). Both put a requirement into
the trait's public interface. They differ in position, and in what the reader gets. `#[extend]` adds a
**supertrait**, a bound on `Self`, which callers do receive, because Rust elaborates a supertrait bound
automatically. `#[extend_where]` adds a **predicate**, which can bound anything, most usefully one of the
trait's own generic parameters, which a supertrait cannot reach.

## Usage

`#[extend_where(...)]` takes a comma-separated list of full `where`-clause predicates:

```rust
#[extend_where(Scalar: Clone)]
```

Unlike [`#[uses]`](uses.md) and [`#[extend]`](extend.md), whose entries are trait bounds attached to `Self`,
these are arbitrary predicates: a bound on any type in scope, including an associated-type equality, a
higher-ranked bound, or a lifetime bound. The macro adds each to the generated trait's `where` clause
verbatim, and keeps it on the implementation as well.

**Only [`#[cgp_fn]`](../macros/cgp_fn.md) supports `#[extend_where]`.** On
[`#[cgp_impl]`](../macros/cgp_impl.md) or [`#[cgp_component]`](../macros/cgp_component.md) the `where`
clause you write is already part of the definition, so there is nothing to promote. Write the bound as
an ordinary `where` clause there.

## Examples

A generic capability whose parameter carries a bound that belongs to the contract:

```rust
use cgp::prelude::*;
use core::ops::Mul;

#[cgp_fn]
#[extend_where(Scalar: Clone)]
pub fn scale<Scalar>(&self, #[implicit] factor: Scalar) -> Scalar
where
    Scalar: Mul<Output = Scalar>,
{
    factor.clone() * factor
}
```

Each bound has its own destination. The macro promotes `Scalar: Clone` onto the trait, so the compiler
checks it wherever `Scale<Scalar>` is named. `Scalar: Mul<Output = Scalar>` stays on the implementation,
because multiplication is a detail of how *this* body computes a scale and no use site needs to know it.

The promotion's effect is visible at the boundary. The compiler rejects a caller that names the
capability for a type it cannot satisfy, at the place the bound is written, and names the trait that
demanded it:

```rust
pub struct NoClone;

pub fn scale_it<Ctx>(ctx: &Ctx) -> NoClone
where
    Ctx: Scale<NoClone>,   // error[E0277]: the trait bound `NoClone: Clone` is not satisfied
{
    ctx.scale()
}
```

**With `Clone` left on the implementation instead, that same function compiles** without a diagnostic.
`Ctx: Scale<NoClone>` is then a bound that no type can prove, and the compiler says so only when somebody
calls `scale_it` with a concrete context. The attribute exists to report that error earlier.

A caller who *can* satisfy the predicate states it as usual, and cannot avoid stating it:

```rust
pub fn scale_it<Ctx, Scalar>(ctx: &Ctx) -> Scalar
where
    Ctx: Scale<Scalar>,
    Scalar: Clone,
{
    ctx.scale()
}
```

## When to use it

**Reach for `#[extend_where]` when a predicate is part of what the capability means**, and you want the
compiler to enforce it where the trait is named, rather than letting it narrow which contexts the
implementation covers without a report. That is a real but uncommon need, and the default of leaving
bounds on the implementation is right for almost everything.

The useful test is who the bound is *about*. A bound describing how the body computes its answer belongs on
the implementation. A bound describing what the capability requires of its own type parameters, something
that would be part of the signature if you were writing the trait by hand, belongs on the trait.

Other constructs carry the requirements that belong elsewhere.

- **A bound on `Self`** is a supertrait, so use [`#[extend]`](extend.md). `#[extend_where]` can express it,
  but a supertrait reads as what it is, and unlike a predicate Rust *does* hand it to callers by
  elaboration.
- **A private requirement**, the overwhelmingly common case, belongs in the function's own `where` clause,
  or in [`#[uses]`](uses.md) when it is a capability.
- **An abstract type pinned to a concrete one** is [`#[use_type]`](use_type.md)'s equality form, which adds
  the bound and lets the signature name the type as a bare word.

Do not use it to spare callers a bound, because it has the opposite effect. If holding the capability
should imply something, use a supertrait, which requires the bound to be on `Self`.

## Under the hood

The macro adds each predicate to the generated trait's `where` clause, and also keeps it on the
implementation so the body can rely on it. From the `scale` example above:

```rust
pub trait Scale<Scalar>
where
    Scalar: Clone,
{
    fn scale(&self) -> Scalar;
}

impl<__Context__, Scalar> Scale<Scalar> for __Context__
where
    Scalar: Mul<Output = Scalar>,
    Scalar: Clone,
    Self: HasField<Symbol!("factor"), Value = Scalar>,
{
    fn scale(&self) -> Scalar { /* ... */ }
}
```

The trait carries only the promoted predicate. The function's own bound is absent from it, which is the
default this attribute overrides.

The implementation carries all of them, and their **order is fixed**: the function's own `where` clause
first, then whatever the attributes contribute, then the
[`HasField`](../traits/field-access/has_field.md) bounds from [`#[implicit]`](implicit.md) arguments,
which are always appended last. That order helps when you read a long `where` clause in an expansion,
because it tells you where each bound came from.

The context parameter is literally `__Context__` in the emitted code and appears as `Self` inside the
implementation.

## Formal grammar

The attribute argument is a comma-separated list of `where` predicates, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
ExtendWhereArgs -> WherePredicate ( `,` WherePredicate )* `,`?
```

`WherePredicate` is the Rust grammar's own production, the thing that appears between the commas of a
`where` clause, and it separates this attribute from [`#[uses]`](uses.md) and [`#[extend]`](extend.md).
Those take a *bound* and always attach it to `Self`, whereas a predicate names its own subject, so
`#[extend_where(Self::Output: Clone)]` and `#[extend_where(for<'a> &'a T: IntoIterator)]` are both
expressible here and neither is expressible there. The list may be empty and the attribute may be
repeated.

## Common Mistakes

**A promoted predicate does not reach callers as a guarantee.** This is the mistake the attribute most
invites: because `#[extend]` gives callers its supertrait, it is natural to assume `#[extend_where]` gives
them its predicate. It does not. Naming a trait whose `where` clause is unproven is an error against the
bound, so a caller generic over the parameter must state the predicate too. This caller omits it:

```rust
pub fn scale_it<Ctx, Scalar>(ctx: &Ctx) -> Scalar
where
    Ctx: Scale<Scalar>,
{
    ctx.scale()
}
```

```text
error[E0277]: the trait bound `Scalar: Clone` is not satisfied
   |
   |     Ctx: Scale<Scalar>,
   |          ^^^^^^^^^^^^^ the trait `Clone` is not implemented for `Scalar`
   |
note: required by a bound in `Scale`
```

The fix is to add `Scalar: Clone` to the caller's own `where` clause. This is the expected use, not a
workaround.

**On any host other than `#[cgp_fn]` the compiler does not recognize the attribute**, rather than
accepting and ignoring it. The macro does not consume it, so it reaches the compiler as an unknown
attribute:

```rust
#[cgp_component(Scaler)]
#[extend_where(Scalar: Clone)]
pub trait CanScale<Scalar> {
    fn scale(&self) -> Scalar;
}
```

```text
error: cannot find attribute `extend_where` in this scope
```

On a [`#[cgp_component]`](../macros/cgp_component.md) or [`#[cgp_impl]`](../macros/cgp_impl.md) the fix is
to delete the attribute and write the predicate in the definition's own `where` clause, where it already
has the effect this attribute exists to produce elsewhere.

## Related constructs

- [`#[extend]`](extend.md) — the supertrait sibling, and the one callers do receive.
- [`#[uses]`](uses.md) — a private capability bound on the implementation.
- [`#[use_type]`](use_type.md) — for pinning an abstract type, which this could express but should not.
- [`#[cgp_fn]`](../macros/cgp_fn.md) — the only host, and the reason the attribute exists.
- [`#[implicit]`](implicit.md) — contributes the bounds that always sort last.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — why a `#[cgp_fn]`'s own `where`
  clause stays off its trait by default.

## Source

- Parsing: [`types/attributes/function.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/function.rs)
- Injection into both `where` clauses: [`types/cgp_fn/preprocessed.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_fn/preprocessed.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
