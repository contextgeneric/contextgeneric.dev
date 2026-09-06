---
sidebar_label: '#[uses]'
sidebar_position: 2
---

# `#[uses]`

Import the capabilities an implementation depends on, reading like a `use` statement.

## Overview

`#[uses]` names a [consumer trait](../macros/cgp_component.md), or an ordinary Rust trait, that `Self`
must satisfy, so the body can call it. `Self` here is the **context**, the type the implementation runs
against:

```rust
#[uses(RectangleArea)]
```

That reads as *this implementation uses the `RectangleArea` capability*, the same meaning a `use`
statement has for a name. The body can then call `self.rectangle_area()` as though it had imported
the capability. `#[uses]` is the consumer-side counterpart to [`#[use_provider]`](use_provider.md).
`#[use_provider]` imports a provider trait that a named provider must satisfy for the context, whereas
`#[uses]` imports a trait that the context itself must satisfy.

The requirement stays private to the implementation. A caller who depends on the capability never sees
it and never has to repeat it. That is the point of declaring the dependency where the implementation
lives rather than on its public interface. Prefer `#[uses]` over a hand-written bound. The equivalent
`where` clause is the older form you meet in existing code, and [Under the hood](#under-the-hood)
shows that the two desugar identically.

## Usage

`#[uses(...)]` takes a comma-separated list of bounds, each becoming a `Self:` predicate on the generated
implementation:

```rust
#[uses(RectangleArea, CanCalculateArea)]
```

Each entry is a capability, optionally with type arguments: a bare `RectangleArea` becomes
`Self: RectangleArea`, and `CanCompute<Code, Input>` becomes `Self: CanCompute<Code, Input>`.

**The trait need not be a CGP construct.** `#[uses(Display)]` and `#[uses(AsRef<[u8]>)]` are accepted and
preferred over the equivalent hand-written clause. The attribute only requires that the bound is one a
context can satisfy, so an ordinary Rust trait imports exactly as a capability does.

**Prefer one attribute carrying every dependency**, as in `#[uses(RectangleArea, Display)]`, because a
single list reads as a single set of requirements. You may also split entries across several
`#[uses(...)]` attributes on the same item, and the macro merges them into one bound. Use a second
attribute only when there is a reason.

`#[uses(...)]` is accepted on [`#[cgp_fn]`](../macros/cgp_fn.md) and on
[`#[cgp_impl]`](../macros/cgp_impl.md). In both, it imports into the item being defined, and it does not
matter how the imported capability was itself produced.

### Bounds beyond the simple form

The idiomatic entry is a plain `Trait<Params>`, because the attribute is meant to read as an import. An
entry may nonetheless be any bound a `where` clause accepts: an associated-type equality such as
`HasErrorType<Error = anyhow::Error>`, a higher-ranked bound, or a lifetime bound. Each lands on the
`where` clause verbatim.

Use that generality sparingly, and prefer a more specific tool where one exists. To pin an abstract type
to a concrete one, [`#[use_type]`](use_type.md)'s equality form
`#[use_type(HasErrorType.{Error = anyhow::Error})]` adds the same bound *and* lets the signature name the
type as a bare `Error`. To put a bound on the generated trait rather than only on its implementation, use
[`#[extend]`](extend.md) for a supertrait or [`#[extend_where]`](extend_where.md) for a predicate.

## Examples

A capability built on top of a component, without knowing which provider supplies it:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_fn]
#[uses(CanCalculateArea)]
pub fn scaled_area(&self, #[implicit] scale_factor: f64) -> f64 {
    self.area() * scale_factor * scale_factor
}
```

`#[uses(CanCalculateArea)]` makes `self.area()` legal. The dependency is on the *consumer* trait rather
than on any particular provider, so every context that can calculate an area gets `scaled_area`
regardless of how it does so. A context that swaps its area provider keeps `scaled_area` working
unchanged.

The same attribute inside a provider, this time importing a plain function-style capability:

```rust
#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}

#[cgp_impl(new RectangleAreaCalculator)]
#[uses(RectangleArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        self.rectangle_area()
    }
}
```

This provider is a short adapter: it satisfies the `AreaCalculator` component by calling whatever
`rectangle_area` computes. Any context with the fields that capability needs can wire it.

## When to use it

**Use `#[uses]` for every capability dependency**, in preference to writing the `Self:` bound by hand.
You read the hand-written form in existing code rather than write it.

The choice between `#[uses]` and its neighbours turns on *what* the implementation depends on, and on
*where* the requirement should be visible.

- **A value from the context** is not a capability. Use an [`#[implicit]`](implicit.md) argument, which
  reads a field directly rather than routing through a trait.
- **A type from the context** is imported with [`#[use_type]`](use_type.md), which adds the bound and
  additionally lets the signature write the type as a bare name instead of a qualified path.
- **An inner provider** in a higher-order provider needs [`#[use_provider]`](use_provider.md), because a
  provider trait carries a context argument that `#[uses]` would not fill in.
- **A requirement callers should see** belongs on the trait rather than on the implementation:
  [`#[extend]`](extend.md) makes it a supertrait, and [`#[extend_where]`](extend_where.md) a predicate.
  `#[uses]` keeps a requirement private to the implementation, where those two make it public.

## Under the hood

Every entry becomes one `Self`-anchored predicate on the generated implementation, joined with `+`, and
nothing else changes. From this input:

```rust
#[cgp_fn]
#[uses(BaseArea, Display)]
pub fn describe_area(&self) -> String {
    format!("{} for {}", self.base_area(), self)
}
```

the macro emits the trait exactly as it would have been without the attribute, and the implementation
carries the imports:

```rust
pub trait DescribeArea {
    fn describe_area(&self) -> String;
}

impl<__Context__> DescribeArea for __Context__
where
    Self: BaseArea + Display,
{
    fn describe_area(&self) -> String { /* ... */ }
}
```

That asymmetry is the whole mechanism. Callers name `DescribeArea`, which says nothing about `BaseArea`
or `Display`, so a caller bounding on it does not inherit a requirement to pass on. Writing
`#[uses(BaseArea, Display)]` is therefore exactly equivalent to writing
`where Self: BaseArea + Display` on the function, and the two desugar identically.

The macro collects entries split across stacked `#[uses(...)]` attributes before it builds the bound, so
`#[uses(BaseArea)]` above `#[uses(Display)]` produces the same single predicate shown above. The context
parameter is literally `__Context__` in the emitted code, and appears as `Self` inside the implementation.

Inside a [`#[cgp_impl]`](../macros/cgp_impl.md) block the behaviour is the same, with the predicates
appended to that provider's `where` clause. That is also where the generated
[`IsProviderFor`](../traits/wiring/is_provider_for.md) impl picks them up, so the compiler reports an unmet
import by name rather than as a bare missing implementation.

## Formal grammar

The attribute argument is a comma-separated list of bounds, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
UsesArgs -> TypeParamBound ( `,` TypeParamBound )* `,`?
```

`TypeParamBound` is the Rust grammar's own bound production, which is wider than the plain `Trait<Args>`
this attribute is normally written with: a lifetime, a `?Sized`, and an associated-type equality such as
`HasErrorType<Error = AppError>` all parse. The list may be empty, and the attribute may be repeated,
with entries from every occurrence collected into one `Self:` predicate. The commas separate *bounds*, so
`#[uses(A, B)]` and `#[uses(A)] #[uses(B)]` are the same thing, and the single-attribute form is the one
to write.

## Common Mistakes

**Naming a provider trait instead of a consumer trait does not work**, and it is the most common way to
write `#[uses]` when you wanted [`#[use_provider]`](use_provider.md). A provider trait carries an
explicit context parameter, and `#[uses]` does not fill it in, so the bound is incomplete:

```rust
#[cgp_fn]
#[uses(AreaCalculator)]
pub fn scaled_area(&self, #[implicit] scale_factor: f64) -> f64 {
    self.area() * scale_factor * scale_factor
}
```

```text
error[E0107]: missing generics for trait `AreaCalculator`
   |
   | #[uses(AreaCalculator)]
   |        ^^^^^^^^^^^^^^ expected 1 generic argument
   |
note: trait defined here, with 1 generic parameter: `__Context__`
```

The `__Context__` in that note points to the mistake. Depend on the **consumer** trait, as in
`#[uses(CanCalculateArea)]`, when you want "whatever this context has wired", which is almost always the
case. Use [`#[use_provider]`](use_provider.md) when you mean a named implementation, and it will supply
the missing argument.

## Related constructs

- [`#[cgp_fn]`](../macros/cgp_fn.md) and [`#[cgp_impl]`](../macros/cgp_impl.md) — the two hosts.
- [`#[extend]`](extend.md) — the same syntax, but as a public supertrait rather than a private bound.
- [`#[extend_where]`](extend_where.md) — a predicate on the generated trait's own `where` clause.
- [`#[use_type]`](use_type.md) — imports a type rather than a capability, and rewrites its uses.
- [`#[use_provider]`](use_provider.md) — imports an inner provider, filling in its context argument.
- [`#[implicit]`](implicit.md) — imports a value from a field rather than a capability.
- [`#[cgp_component]`](../macros/cgp_component.md) — defines the capabilities most often imported.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — what an imported capability
  becomes, and why callers never see it.

## Source

- Parsing: [`types/attributes/uses.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/uses.rs)
- Collection: [`types/attributes/function.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/function.rs)
  and [`types/attributes/cgp_impl_attributes.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/cgp_impl_attributes.rs)
- Injection: [`types/cgp_fn/preprocessed.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_fn/preprocessed.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
