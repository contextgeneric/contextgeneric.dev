---
sidebar_label: '#[extend_where]'
sidebar_position: 6
---

# `#[extend_where]`

Add `where` predicates to a generated trait's own definition, not just to its implementation.

## Overview

A [`#[cgp_fn]`](../macros/cgp_fn.md) treats the `where` clause you write on the function as an
implementation detail: the bounds land on the generated implementation and never appear on the generated
trait. That default keeps a capability's requirements out of sight of its callers, and it is right almost
always.

The cost is that an unsatisfiable requirement becomes invisible. A bound on the implementation only
decides which **contexts** the implementation covers, a context being the type the capability runs
against, which supplies the values it needs as its fields. So naming the capability for a type that can
never satisfy it is not an error. It is a bound nobody can prove, accepted quietly, and the complaint
arrives later and somewhere else. `#[extend_where]` moves the predicate onto the trait, where it becomes a
condition of naming the trait at all:

```rust
#[extend_where(Scalar: Clone)]
```

The distinction is worth being exact about, because it is easy to get backwards. Promoting a predicate
does **not** hand it to callers. A trait's `where` clause is a precondition they must prove, not a
guarantee they receive, so a caller naming the trait still writes the bound themselves. Promotion changes
two things: callers are now *made* to write the bound, and getting it wrong is reported against the trait
rather than deferred.

It is the `where`-clause sibling of [`#[extend]`](extend.md). Both put a requirement into the trait's
public interface; they differ in position, and in what the reader gets. `#[extend]` adds a **supertrait**,
a bound on `Self`, which callers do receive by elaboration. `#[extend_where]` adds a **predicate**, which
can bound anything, most usefully one of the trait's own generic parameters, which a supertrait cannot
reach.

## Usage

`#[extend_where(...)]` takes a comma-separated list of full `where`-clause predicates:

```rust
#[extend_where(Scalar: Clone)]
```

Unlike [`#[uses]`](uses.md) and [`#[extend]`](extend.md), whose entries are trait bounds attached to `Self`,
these are arbitrary predicates: a bound on any type in scope, including an associated-type equality, a
higher-ranked bound, or a lifetime bound. The macro adds each to the generated trait's `where` clause
verbatim, and keeps it on the implementation as well.

**`#[extend_where]` is supported only on [`#[cgp_fn]`](../macros/cgp_fn.md).** It has no meaning on
[`#[cgp_impl]`](../macros/cgp_impl.md) or [`#[cgp_component]`](../macros/cgp_component.md), because in
those the `where` clause you write is already part of the definition. There is nothing to promote, so
write the bound as an ordinary `where` clause directly.

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

Two bounds, two destinations. The macro promotes `Scalar: Clone` onto the trait, so the compiler checks
it wherever `Scale<Scalar>` is named. `Scalar: Mul<Output = Scalar>` stays on the implementation, because
multiplication is a detail of how *this* body computes a scale and no use site needs to know it.

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

**Leave `Clone` on the implementation instead and that same function compiles**, with no diagnostic at
all: `Ctx: Scale<NoClone>` is simply a bound no type can ever prove, and nothing says so until somebody
tries to call `scale_it` with a concrete context. The attribute exists to remove that silence.

A caller who *can* satisfy the predicate states it as usual, and there is no way around stating it:

```rust
pub fn scale_it<Ctx, Scalar>(ctx: &Ctx) -> Scalar
where
    Ctx: Scale<Scalar>,
    Scalar: Clone,
{
    ctx.scale()
}
```

## When to reach for it, and when not

**Reach for `#[extend_where]` when a predicate is part of what the capability means**, and you want it
enforced where the trait is named rather than silently narrowing which contexts the implementation covers.
That is a real but uncommon need, and the default of leaving bounds on the implementation is right for
almost everything.

The useful test is who the bound is *about*. A bound describing how the body computes its answer belongs on
the implementation. A bound describing what the capability requires of its own type parameters, something
that would be part of the signature if you were writing the trait by hand, belongs on the trait.

Three neighbours cover what this attribute should not be used for.

- **A bound on `Self`** is a supertrait, so use [`#[extend]`](extend.md). `#[extend_where]` can express it,
  but a supertrait reads as what it is, and unlike a predicate it *is* handed to callers by elaboration.
- **A private requirement**, the overwhelmingly common case, belongs in the function's own `where` clause,
  or in [`#[uses]`](uses.md) when it is a capability.
- **An abstract type pinned to a concrete one** is [`#[use_type]`](use_type.md)'s equality form, which adds
  the bound and lets the signature name the type as a bare word.

Do not reach for it to spare callers a bound: it has the opposite effect. If the goal is that holding the
capability should imply something, a supertrait does that, and the bound has to be on `Self` for it to work.

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

The trait carries only the promoted predicate; the function's own bound is absent from it, which is the
default this attribute overrides.

The implementation carries three, and their **order is fixed**: the function's own `where` clause first,
then whatever the attributes contribute, then the [`HasField`](../traits/has_field.md) bounds from
[`#[implicit]`](implicit.md) arguments, which are always appended last. That order is worth knowing when
reading a long `where` clause in an expansion, since it tells you where each bound came from.

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
bound, so a caller generic over the parameter must state the predicate too:

```text
error[E0277]: the trait bound `Scalar: Clone` is not satisfied
   |
   |     Ctx: Scale<Scalar>,
   |          ^^^^^^^^^^^^^ the trait `Clone` is not implemented for `Scalar`
   |
note: required by a bound in `Scale`
```

Adding `Scalar: Clone` to the caller's own `where` clause is the fix, and is expected rather than a
workaround.

**On any host other than `#[cgp_fn]` the attribute is not recognized**, rather than accepted and ignored.
Nothing consumes it, so it reaches the compiler as an unknown attribute:

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
