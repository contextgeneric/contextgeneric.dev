---
sidebar_label: '#[uses]'
---

# `#[uses]`

Import the capabilities an implementation depends on, reading like a `use` statement.

## What it's for

An implementation usually calls capabilities defined elsewhere, and to do so it has to require that the
**context** — the type it runs against — provides them. Written out, that requirement is a bound on
`Self`:

```rust
where
    Self: RectangleArea,
```

Bounding `Self` is unusual in everyday Rust and reads as machinery rather than as intent. `#[uses]` says
the same thing in the register the code is actually in:

```rust
#[uses(RectangleArea)]
```

That reads as *this implementation uses the `RectangleArea` capability*, which is what a `use` statement
means for a name, and the body can then call `self.rectangle_area()` as though the capability had been
imported.

The two forms generate exactly the same bound, so nothing is gained or lost mechanically. What changes is
that the dependency list stops looking like a constraint the author had to satisfy and starts looking like
a list of what the code needs — which is the reason `#[uses]` is the recommended form and a hand-written
`where Self: Trait` is what you meet in older code.

One property of that bound is worth naming, because it is the whole point of putting the dependency on
the implementation. It lands on the implementation *only*, never on the trait, so a caller who bounds on
the capability never sees it and never has to repeat it. What an implementation needs is its own business.

## Using it

`#[uses(...)]` takes a comma-separated list of bounds, each becoming a `Self:` predicate on the generated
implementation:

```rust
#[uses(RectangleArea, CanCalculateArea)]
```

Each entry is a capability, optionally with type arguments — a bare `RectangleArea` becomes
`Self: RectangleArea`, and `CanCompute<Code, Input>` becomes `Self: CanCompute<Code, Input>`.

**The trait need not be a CGP construct.** `#[uses(Display)]` and `#[uses(AsRef<[u8]>)]` are accepted and
preferred over the equivalent hand-written clause: the attribute only cares that the bound is one a
context can satisfy, so an ordinary Rust trait imports exactly as a capability does.

**Prefer one attribute carrying every dependency** — `#[uses(RectangleArea, Display)]` — since a single
list reads as a single set of requirements. Entries may also be split across several `#[uses(...)]`
attributes on the same item and they accumulate into one bound, but reach for a second attribute only
when there is a reason rather than by default.

`#[uses(...)]` is accepted on [`#[cgp_fn]`](../macros/cgp_fn.md) and on
[`#[cgp_impl]`](../macros/cgp_impl.md). In both it imports into the thing being defined, and it does not
care how the imported capability was itself produced.

### Bounds beyond the simple form

The idiomatic entry is a plain `Trait<Params>`, because the attribute is meant to read as an import. An
entry may nonetheless be any bound a `where` clause accepts — an associated-type equality such as
`HasErrorType<Error = anyhow::Error>`, a higher-ranked bound, a lifetime bound — and it lands on the
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

`#[uses(CanCalculateArea)]` is what makes `self.area()` legal. The dependency is on the *consumer* trait
rather than on any particular provider, so every context that can calculate an area gets `scaled_area`
regardless of how it does so — and a context that swaps its area provider keeps `scaled_area` working
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

This provider is a two-line adapter: it satisfies the `AreaCalculator` component by deferring to whatever
`rectangle_area` computes. Any context with the fields that capability needs can wire it.

## When to reach for it, and when not

**Use `#[uses]` for every capability dependency**, in preference to writing the `Self:` bound by hand.
That is the recommendation, and the hand-written form is what to read rather than write.

The distinction that decides between `#[uses]` and its neighbours is *what* is being depended on and
*where the requirement should be visible*.

- **A value from the context** is not a capability. Use an [`#[implicit]`](implicit.md) argument, which
  reads a field directly rather than routing through a trait.
- **A type from the context** is imported with [`#[use_type]`](use_type.md), which adds the bound and
  additionally lets the signature write the type as a bare name instead of a qualified path.
- **An inner provider** in a higher-order provider needs [`#[use_provider]`](use_provider.md), because a
  provider trait carries a context argument that `#[uses]` would not fill in.
- **A requirement callers should see** belongs on the trait rather than on the implementation:
  [`#[extend]`](extend.md) makes it a supertrait, and [`#[extend_where]`](extend_where.md) a predicate.
  `#[uses]` is for what the implementation needs privately, which is the `use` to their `pub use`.

## Under the hood

:::note

### Advanced

This section shows what the attribute injects. You do not need it to use `#[uses]`, but it is a short
rewrite and seeing it once explains why the bound never appears on the trait. `cargo cgp expand` prints
the same thing for your own code.

:::

Every entry becomes one `Self`-anchored predicate on the generated implementation, joined with `+`, and
nothing else changes. From this input:

```rust
#[cgp_fn]
#[uses(BaseArea, Display)]
pub fn describe_area(&self) -> String {
    format!("{} for {}", self.base_area(), self)
}
```

the trait is emitted exactly as it would have been without the attribute, and the implementation carries
the imports:

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

That asymmetry is the mechanism, not a detail: `DescribeArea` is what callers name, and it says nothing
about `BaseArea` or `Display`, so a caller bounding on it inherits no requirement to pass on. Writing
`#[uses(BaseArea, Display)]` is therefore exactly equivalent to writing
`where Self: BaseArea + Display` on the function, and the two desugar identically.

Entries split across stacked `#[uses(...)]` attributes are collected before the bound is built, so
`#[uses(BaseArea)]` above `#[uses(Display)]` produces the same single predicate shown above. The context
parameter is literally `__Context__` in the emitted code, and appears as `Self` inside the implementation.

Inside a [`#[cgp_impl]`](../macros/cgp_impl.md) block the behaviour is the same, with the predicates
appended to that provider's `where` clause — which is also where they are picked up by the generated
[`IsProviderFor`](../traits/is_provider_for.md) impl, so an unmet import is reported by name rather than
as a bare missing implementation.

## Gotchas

**Naming a provider trait instead of a consumer trait does not work**, and this is the most likely way to
reach for `#[uses]` when [`#[use_provider]`](use_provider.md) is what you wanted. A provider trait carries
an explicit context parameter, and `#[uses]` does not fill it in, so the bound is incomplete:

```text
error[E0107]: missing generics for trait `AreaCalculator`
   |
   | #[uses(AreaCalculator)]
   |        ^^^^^^^^^^^^^^ expected 1 generic argument
   |
note: trait defined here, with 1 generic parameter: `__Context__`
```

The `__Context__` in that note is the giveaway. Depend on the **consumer** trait —
`#[uses(CanCalculateArea)]` — when what you want is "whatever this context has wired", which is almost
always the case. Use [`#[use_provider]`](use_provider.md) when you genuinely mean a named implementation,
and it will supply the missing argument.

## Related constructs

- [`#[cgp_fn]`](../macros/cgp_fn.md) and [`#[cgp_impl]`](../macros/cgp_impl.md) — the two hosts.
- [`#[extend]`](extend.md) — the same syntax, but as a public supertrait rather than a private bound.
- [`#[extend_where]`](extend_where.md) — a predicate on the generated trait's own `where` clause.
- [`#[use_type]`](use_type.md) — imports a type rather than a capability, and rewrites its uses.
- [`#[use_provider]`](use_provider.md) — imports an inner provider, filling in its context argument.
- [`#[implicit]`](implicit.md) — imports a value from a field rather than a capability.
- [`#[cgp_component]`](../macros/cgp_component.md) — defines the capabilities most often imported.

## Source

- Parsing: [`types/attributes/uses.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/uses.rs)
- Collection: [`types/attributes/function.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/function.rs)
  and [`types/attributes/cgp_impl_attributes.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/cgp_impl_attributes.rs)
- Injection: [`types/cgp_fn/preprocessed.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_fn/preprocessed.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
