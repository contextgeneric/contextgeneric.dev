---
sidebar_label: '#[use_provider]'
sidebar_position: 4
---

# `#[use_provider]`

Depend on another provider by name, importing the provider trait it must satisfy.

## Overview

An implementation can depend on *another* implementation: a wrapper that scales whatever an inner
calculator produces, a retry layer over whatever performs a request. This shape is a
[higher-order provider](/docs/concepts/higher-order-providers), and `#[use_provider]` declares its
dependency by importing the [provider trait](../macros/cgp_component.md) the inner provider must satisfy:

```rust
#[use_provider(InnerCalculator: AreaCalculator)]
```

That reads as *`InnerCalculator` is an `AreaCalculator` for this context*, and it is the provider-side
counterpart to [`#[uses]`](uses.md). Where `#[uses]` imports a **consumer trait** (or an ordinary Rust
trait) that the **context** must satisfy, `#[use_provider]` imports a **provider trait** that a named
provider must satisfy for the context. The context is the type the implementation runs against. Both read
as importing a dependency; they differ only in whether the thing depended on is a capability of the
context or a provider.

The inner provider is a named type rather than a method on the context, so the body calls it as an
associated function and passes the context:

```rust
let base_area = InnerCalculator::area(self);
```

`#[use_provider]` writes the bound behind that call; it does not rewrite the call. Writing `self.area()`
instead would route through whatever provider the context has itself wired for `AreaCalculator`, which is
a different choice and usually not what a higher-order provider wants.

## Usage

`#[use_provider(...)]` takes a provider, a colon, and the trait bounds it must satisfy:

```rust
#[use_provider(InnerCalculator: AreaCalculator)]
```

`InnerCalculator` is the provider, usually a generic parameter of the implementation, and
`AreaCalculator` is the provider trait it must satisfy. The trait may carry further arguments of its own,
and the macro preserves those in order.

Two forms cover more than one bound, and which you need depends on what is being multiplied.

**One provider, several traits: join them with `+`.**

```rust
#[use_provider(Inner: AreaCalculator + PerimeterCalculator)]
```

**Several providers: one attribute each, stacked.**

```rust
#[use_provider(A: AreaCalculator)]
#[use_provider(P: PerimeterCalculator)]
```

Stacking is the intended form here rather than a fallback, because a comma-separated list of
provider-and-trait pairs is *not* accepted (see [Common Mistakes](#common-mistakes)). This differs from
[`#[uses]`](uses.md) and [`#[use_type]`](use_type.md), where commas are the preferred way to carry several
entries, so it is worth remembering as the exception.

`#[use_provider]` is accepted on [`#[cgp_impl]`](../macros/cgp_impl.md) and on
[`#[cgp_fn]`](../macros/cgp_fn.md), and it works the same way on both.

## Examples

The typical use is a wrapper parameterized by whatever it wraps. Given a component and a plain
implementation of it:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}
```

a scaling wrapper takes the inner implementation as a parameter, declares it with `#[use_provider]`, and
calls it by name:

```rust
#[cgp_impl(new ScaledArea<InnerCalculator>)]
#[use_provider(InnerCalculator: AreaCalculator)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        let base_area = InnerCalculator::area(self);
        base_area * scale_factor * scale_factor
    }
}
```

A context then composes the two when it wires the component, and `ScaledArea<RectangleArea>` computes a
rectangle's area and scales it:

```rust
#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: ScaledArea<RectangleArea>,
    }
}
```

Because the wrapper never names a particular inner implementation, the same `ScaledArea` composes over any
of them. Composition is itself a type, so `type ScaledRectangle = ScaledArea<RectangleArea>;` is a
complete way to name the combination.

The attribute is also useful for depending on one *specific* implementation rather than a parameter. A
[`#[cgp_fn]`](../macros/cgp_fn.md) does exactly that when it wants a named implementation instead of
whatever the context has wired:

```rust
#[cgp_fn]
#[use_provider(RectangleArea: AreaCalculator)]
pub fn rect_area(&self) -> f64 {
    RectangleArea::area(self)
}
```

## When to use it

**Use `#[use_provider]` whenever an implementation depends on another implementation by name**, in
preference to writing the bound by hand. You read the explicit `Inner: AreaCalculator<Self>` form in
generated code and older wiring rather than write it.

The choice against its neighbours is about *what* is being depended on, and one of the distinctions is
easy to miss because both spellings compile.

- **A capability of the context** is [`#[uses]`](uses.md), which imports a consumer trait or an ordinary
  trait the context satisfies. `#[use_provider]` imports a provider trait instead, one that a separate
  provider satisfies for the context.
- **Whatever the context already chose** needs no attribute at all. Calling `self.area()` in the body
  routes through the context's own wiring, which is a *different dispatch* from
  `InnerCalculator::area(self)`: the first asks the context, the second names an implementation
  statically. Reach for `#[use_provider]` only when you want the second.
- **A choice made per type rather than fixed** is dispatch rather than parameterization, so it belongs in
  the wiring: the `open` statement of [`delegate_components!`](../macros/delegate_components.md) maps each
  type to its own implementation.

One ergonomic option is worth knowing when writing the provider struct by hand. Giving the parameter a
default of [`UseContext`](../providers/use_context.md), as in `pub struct IterSum<Inner = UseContext>(...)`,
makes an unparameterized `IterSum` fall back to the context's own wiring, so the wrapper can be dropped in
without naming a base case.

## Under the hood

A [provider trait](../macros/cgp_component.md) is not shaped like the consumer trait it came from: the
original `Self` moves into an explicit leading type parameter for the context, so the real bound on an
inner provider is `InnerCalculator: AreaCalculator<Self>`, with a context argument the consumer trait has
no counterpart for. `#[use_provider]` writes that `<Self>` for you. It inserts the context type as the
provider trait's leading argument and appends the completed bound to the `where` clause, and nothing else
changes: the macro emits the body as written. From the `ScaledArea` example:

```rust
#[cgp_impl(new ScaledArea<InnerCalculator>)]
#[use_provider(InnerCalculator: AreaCalculator)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        InnerCalculator::area(self) * scale_factor * scale_factor
    }
}
```

the macro emits the provider implementation with the argument filled in:

```rust
impl<__Context__, InnerCalculator> AreaCalculator<__Context__>
    for ScaledArea<InnerCalculator>
where
    __Context__: HasField<Symbol!("scale_factor"), Value = f64>,
    InnerCalculator: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        let scale_factor: f64 = __context__
            .get_field(PhantomData::<Symbol!("scale_factor")>)
            .clone();

        InnerCalculator::area(__context__) * scale_factor * scale_factor
    }
}
```

The `Self` you wrote in the attribute is the context, so it appears as `__Context__` here, the reserved
name the surrounding macro inserted. Writing `#[use_provider(InnerCalculator: AreaCalculator)]` is
therefore exactly equivalent to writing `where InnerCalculator: AreaCalculator<Self>` by hand.

On a [`#[cgp_fn]`](../macros/cgp_fn.md) the same insertion happens, and because that macro's
implementation is written *for* the context the bound reads with `Self` directly:

```rust
impl<__Context__> RectArea for __Context__
where
    RectangleArea: AreaCalculator<Self>,
{
    fn rect_area(&self) -> f64 {
        RectangleArea::area(self)
    }
}
```

The macro leaves the body untouched in both. The call stays `InnerCalculator::area(__context__)`, passing
the context as the first argument, because that is the shape the provider trait's method has.

## Formal grammar

The attribute argument is one provider and the provider traits it must satisfy, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
UseProviderArgs -> ProviderType `:` ProviderBound ( `+` ProviderBound )*

ProviderType    -> Type
ProviderBound   -> TypePath GenericArgs?
```

Both parts are required. `ProviderType` names the generic parameter the inner provider occupies, and each
`ProviderBound` is the provider trait to require of it, written *without* its leading context argument,
which the attribute inserts.

Two restrictions follow from those productions and account for every parse failure this attribute
produces. **`ProviderBound` is a path with plain generic arguments**, not a full `TypeParamBound`, so a
turbofish or an associated-type binding in that position does not parse; a bound of that shape belongs in
the block's own `where` clause. And **the argument holds exactly one provider**, because the
`+`-separated bound list runs to the end of the attribute, so a comma after the first pair lands where a
`+` was expected. That makes this attribute the one exception to the comma-separated convention its
siblings follow: bind several inner providers by stacking one attribute each. See [Common Mistakes](#common-mistakes).

## Common Mistakes

**A comma-separated list of pairs is not accepted.** Writing two providers in one attribute is a parse
error, because after the first trait the parser is looking for a `+` to continue that provider's bounds:

```text
error: expected `+`
   |
   | #[use_provider(A: AreaCalculator, P: PerimeterCalculator)]
   |                                 ^
```

Use one attribute per provider instead. This is the opposite of the convention for
[`#[uses]`](uses.md) and [`#[use_type]`](use_type.md), which do take comma-separated lists, so the habit
transfers wrongly.

**The provider and the trait are both required.** There is no bare form naming a provider alone, and
omitting the bound reports the missing colon:

```text
error: expected `:`
   |
   | #[use_provider(RectangleArea)]
   |                             ^
```

**There is no call-site rewriting.** The attribute never turns `self.area()` into
`InnerCalculator::area(self)`, so a body that calls the method on `self` compiles but does something
different: it dispatches through the context's own wiring rather than through the parameter. When the
intent is to use the inner implementation, spell out the associated-function call.

## Related constructs

- [`#[cgp_impl]`](../macros/cgp_impl.md) and [`#[cgp_fn]`](../macros/cgp_fn.md) — the two hosts.
- [`#[cgp_component]`](../macros/cgp_component.md) — generates the provider trait whose argument is filled in.
- [`#[uses]`](uses.md) — the counterpart for a bound on the context rather than on a provider.
- [`UseContext`](../providers/use_context.md) — the usual default for an inner-provider parameter.
- [`delegate_components!`](../macros/delegate_components.md) — where a composed wrapper is wired.
- [`check_components!`](../macros/check_components.md) — its `#[check_providers(...)]` form checks each
  layer of a nested stack separately, which is how a broken layer is localized.

The ideas behind it:

- [Higher-order providers](/docs/concepts/higher-order-providers) — the pattern this attribute
  exists to make writable.

## Source

- Parsing and bound completion: [`types/attributes/use_provider/attribute.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/use_provider/attribute.rs)
- Collection: [`types/attributes/cgp_impl_attributes.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/cgp_impl_attributes.rs)
  and [`types/attributes/function.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/function.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
