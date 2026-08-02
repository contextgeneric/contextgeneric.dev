---
sidebar_label: '#[cgp_type]'
---

# `#[cgp_type]`

Define an abstract-type component, whose concrete type each context chooses for itself.

## What it's for

Generic code often has to name a type it should not choose. A fallible operation returns *some* error, a
geometry routine computes in *some* scalar, a storage layer holds *some* connection handle — and code
written once for many applications cannot decide which. Rust's answer is an
[associated type](https://doc.rust-lang.org/reference/items/associated-items.html): declare
`trait HasScalarType { type Scalar; }`, and code written against it says `Self::Scalar` while leaving the
actual type open.

An abstract type in CGP is exactly that trait and nothing more exotic. What `#[cgp_type]` adds is the
ability to fill the slot by **wiring** rather than by writing an impl. Without it, giving a context a
concrete scalar means writing a provider by hand — a whole impl whose only content is
`type Scalar = f64;`. Since every abstract-type provider has that same trivial shape, `#[cgp_type]`
generates it once and for all, so a **context** — the type the capability runs against, which supplies
the values and types it needs — names the concrete type directly in its wiring table:

```rust
delegate_components! {
    App {
        ScalarTypeProviderComponent: UseType<f64>,
    }
}
```

That is the payoff: `App` now implements `HasScalarType` with `Scalar = f64`, and no provider was written
anywhere. The arrangement mirrors what [`#[cgp_getter]`](./cgp_getter.md) does for values — one
general-purpose provider, parameterized by the thing the context wants to supply.

Two consequences are worth having early. Because the type is chosen per context, two applications can
pick differently from the same generic code. And because the trait is an ordinary Rust trait, a context
can skip the wiring entirely and write `impl HasScalarType for App { type Scalar = f64; }` — barely
longer, and the clearest way to see that nothing unusual is happening.

## Using it

Apply the attribute to a trait containing **exactly one associated type and no methods**:

```rust
#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}
```

Like [`#[cgp_component]`](./cgp_component.md), the component needs a provider trait name, and
`#[cgp_type]` derives one. **The default is keyed off the associated type's name, not the trait's** —
`type Scalar` yields the provider trait `ScalarTypeProvider` and the marker
`ScalarTypeProviderComponent`. This is worth fixing in mind, because every other macro derives its
default from the trait name.

Pass an identifier to override it, exactly as with `#[cgp_component]`:

```rust
#[cgp_type(ProvideScalar)]
pub trait HasScalarType {
    type Scalar;
}
```

The keyed form works here too, so `name`, `provider`, and `context` can each be set explicitly. Only the
default for `provider` differs.

### Bounds on the associated type

A bound on the associated type is carried everywhere the type appears, and enforced on whatever concrete
type a context picks:

```rust
#[cgp_type]
pub trait HasScalarType {
    type Scalar: Copy;
}
```

A context whose wiring names a type that does not satisfy the bound is rejected — `UseType<String>` here,
since `String` is not `Copy` — so the bound is written once on the declaration and enforced against every
context's choice.

**It is enforced lazily, though, like all CGP wiring.** The `delegate_components!` entry naming
`UseType<String>` compiles on its own; the failure appears only once a
[`check_components!`](./check_components.md) forces the question, or the type is finally used. The bound is a
real constraint, not a wiring-site guard.

## Examples

An abstract scalar, a context that supplies it, and generic code that never names a concrete type:

```rust
use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar: Copy;
}

pub struct App;

delegate_components! {
    App {
        ScalarTypeProviderComponent: UseType<f64>,
    }
}

pub fn zero<Context>() -> Context::Scalar
where
    Context: HasScalarType,
    Context::Scalar: Default,
{
    Default::default()
}
```

`App` implements `HasScalarType` with `Scalar = f64` through the generated
[`UseType`](../providers/use_type.md) impl, and the `check_components!` block is what confirms `f64` satisfies
the `Copy` bound.

The point compounds when several pieces of code share one type. Because the type lives on a trait the
context implements, everything that mentions a scalar refers to the *same* `Self::Scalar`, so one wiring
line fixes it for all of them:

```rust
#[cgp_component(AreaCalculator)]
#[use_type(HasScalarType.Scalar)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> Scalar;
}
```

Here `Rectangle` and `Circle` carry no scalar type of their own — the context computing their areas does,
and changing its `UseType<f32>` to `UseType<f64>` changes the scalar for every shape at once. The bare
`Scalar` in that signature is [`#[use_type]`](../attributes/use_type.md) at work, which is how an
abstract type is *imported* wherever it is used.

Direct implementation stays available, and is often clearer for a single context:

```rust
impl HasScalarType for App {
    type Scalar = f64;
}
```

## When to reach for it, and when not

**Reach for `#[cgp_type]` whenever generic code has to name a type the context should choose.** The error
type is the canonical instance — CGP's own [`HasErrorType`](../components/has_error_type.md) is defined
exactly this way, which is why every fallible capability can say `Error` and mean whatever the
application picked.

The decision worth making deliberately is **whether the type needs to be abstract at all**, because
there is a cheaper option covering more ground than it looks.

- **Prefer an inferred impl parameter while the type only flows through values.** If the type appears
  solely because a provider reads a field of it,
  [`#[impl_generics]`](./cgp_fn.md#a-type-the-caller-should-not-name) on a `#[cgp_fn]` puts a parameter on
  the implementation alone. Nothing is wired, nothing is declared, and a context qualifies just by
  carrying a field of a compatible type.
- **Climb to an abstract type when the type must be nameable.** Two things force it: the capability's own
  signature has to mention the type, or two capabilities have to agree they mean the *same* type. An
  inferred parameter can do neither, since it exists only where a value of it passes through.
- **Never thread it as a generic parameter on the capability.** A parameter is an input the caller
  supplies, so it lands in every intermediate signature whether that layer touches the type or not. An
  abstract type is determined by the context and propagates nowhere. That difference is what the
  construct is really buying, and it is why a context can decide a dozen types without any signature
  growing.
- **Use plain [`#[cgp_component]`](./cgp_component.md) when the trait carries methods.** `#[cgp_type]` is
  only for a trait whose entire content is one associated type. A trait with a method *and* a type it
  produces is an ordinary component — CGP's own `CanCompute` is that shape.

One alternative looks like this construct and is not. A getter written with
[`#[cgp_auto_getter]`](./cgp_auto_getter.md) may declare an associated type and return it, inferring it
from the field. When the type's only job is to be a getter's return type, that is shorter and needs no
wiring; `#[cgp_type]` is for a type that stands on its own.

## Under the hood

:::note

### Advanced

This section shows what the macro generates. You do not need it to use `#[cgp_type]`, but the `UseType`
impl is the piece that makes wiring a type possible, and seeing it explains why no provider has to be
written. `cargo cgp expand` prints the same thing for your own code.

:::

`#[cgp_type]` emits everything [`#[cgp_component]`](./cgp_component.md) would, then adds two provider
impls of its own. The component half is the familiar shape — consumer trait, provider trait, two blanket
impls, marker, and the standard [`UseContext`](../providers/use_context.md) and
[`RedirectLookup`](../providers/redirect_lookup.md) impls — differing only in that every blanket impl
forwards an *associated type* rather than a method:

```rust
pub trait ScalarTypeProvider<__Context__>:
    IsProviderFor<ScalarTypeProviderComponent, __Context__, ()>
{
    type Scalar: Copy;
}
```

The first addition is the **`UseType` impl**, and it is the heart of the macro. It implements the provider
trait for `UseType<Scalar>` by setting the abstract type to the generic parameter:

```rust
impl<Scalar, __Context__> ScalarTypeProvider<__Context__> for UseType<Scalar>
where
    Scalar: Copy,
{
    type Scalar = Scalar;
}
```

Read it as: `UseType<T>` is a provider that supplies `T`. Wiring a context's component to `UseType<f64>`
therefore gives it `Scalar = f64` with nothing written. Note the declaration's `Copy` bound copied into
the impl's `where` clause — that is what turns an unsuitable concrete type into an error where the wiring
names it.

The second addition is a [`WithProvider`](../providers/with_provider.md) impl, which adapts CGP's
foundational abstract-type machinery into this component:

```rust
impl<__Provider__, Scalar, __Context__> ScalarTypeProvider<__Context__>
    for WithProvider<__Provider__>
where
    Scalar: Copy,
    __Provider__: TypeProvider<__Context__, ScalarTypeProviderComponent, Type = Scalar>,
{
    type Scalar = Scalar;
}
```

[`HasType` / `TypeProvider`](../components/has_type.md) is CGP's single built-in abstract-type component,
and `UseType` is itself a `TypeProvider`. This impl is what lets one `UseType<T>` satisfy both the
built-in component and any `#[cgp_type]` component you declare, instead of needing a separate provider
per component.

Each generated provider impl is paired with a matching
[`IsProviderFor`](../traits/is_provider_for.md) impl carrying the same bounds, as everywhere else.

<details>
<summary>Formal grammar</summary>

The attribute argument is the same grammar as [`#[cgp_component]`](./cgp_component.md)'s, in the Rust
Reference's [notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CgpTypeArgs -> CgpComponentArgs    // see #[cgp_component]
```

A bare provider name, or the keyed `name` / `provider` / `context` form. The only difference from
`#[cgp_component]` is the default applied when `provider` is omitted: rather than being required, it is
derived from the **associated type's** name plus `TypeProvider`, so `type Scalar;` yields
`ScalarTypeProvider`. Every other key and default behaves as documented there.

</details>

## Gotchas

**The trait must contain exactly one associated type and nothing else.** A method beside the type, a
second type, or no type at all all report the same thing:

```text
error: type trait should contain exactly one associated type item
```

A trait that genuinely needs a method alongside a type it produces is an ordinary
[`#[cgp_component]`](./cgp_component.md).

**The associated type may not be generic or carry a `where` clause**, since the `UseType` impl supplies
one concrete type per wiring and has nowhere to put either:

```text
error: generic associated type and where clause are not supported
```

**The default provider name comes from the type, not the trait.** `HasScalarType` declaring
`type Scalar` yields `ScalarTypeProviderComponent`, not `HasScalarTypeComponent` — so a wiring entry
guessed from the trait name names a component that does not exist, and the error is an unresolved type
rather than anything about wiring.

**A bound on the type is not enforced at the wiring line.** Wiring `UseType<String>` for a type declared
`Copy` compiles; nothing complains until a check evaluates the lookup, and then the primary span is the
component name in the `check_components!` block rather than the offending wiring entry:

```text
error[E0277]: the trait bound `String: Copy` is not satisfied
 --> src/main.rs:5:27
  |
5 | check_components! { App { ScalarTypeProviderComponent } }
  |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Copy` is not implemented for `String`
  |
note: required for `UseType<String>` to implement `IsProviderFor<ScalarTypeProviderComponent, App>`
  |
2 | #[cgp_type] pub trait HasScalarType { type Scalar: Copy; }
  | ^^^^^^^^^^^                                        ---- unsatisfied trait bound introduced here
```

The offending type and the declaration that demanded the bound are both in the notes rather than the
headline, so read downward. **Without a check the mistake is silent**, which is the ordinary consequence of
[lazy wiring](./delegate_components.md#gotchas) rather than anything specific to abstract types.

## Related constructs

- [`#[cgp_component]`](./cgp_component.md) — the general macro this specializes.
- [`UseType`](../providers/use_type.md) — the provider a context wires to supply the concrete type.
- [`#[use_type]`](../attributes/use_type.md) — imports an abstract type so its name can be written bare;
  a different construct from the `UseType` provider despite the shared name.
- [`HasType`](../components/has_type.md) — CGP's built-in abstract-type component, which this builds on.
- [`HasErrorType`](../components/has_error_type.md) — the canonical abstract type, defined this way.
- [`UseDelegatedType`](../providers/use_delegated_type.md) — resolves the type through a table rather than
  fixing it.
- [`WithProvider`](../providers/with_provider.md) — the adapter the second generated impl is for.
- [`#[cgp_getter]`](./cgp_getter.md) — the value-level counterpart, using `UseField` where this uses
  `UseType`.

## Source

- Entry point: [`cgp_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_type.rs)
- Implementation: [`types/cgp_type/item.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_type/item.rs)
- Runtime `HasType`, `TypeProvider`, and `UseType`: [`cgp-type/src/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-type/src/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
