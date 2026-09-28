---
title: 'Life — a lifetime lifted into a type'
sidebar_label: 'Life'
sidebar_position: 4
description: 'A lifetime lifted into a type, so a component''s lifetime parameter can travel through the dependency marker, whose parameters are types.'
---

# `Life`

A lifetime lifted into a type, so a lifetime parameter can travel through machinery that accepts only
types.

## Overview

`Life<'a>` turns the lifetime `'a` into a type, because CGP's wiring is parameterized by *types* and a CGP
trait may carry a lifetime of its own. The marker that records a provider's dependencies,
[`IsProviderFor`](../traits/wiring/is_provider_for.md), takes a tuple of the trait's generic parameters as
one type argument, so that the compiler can match a provider against the exact instantiation asked for. A
lifetime cannot sit in that tuple, because a tuple is a type and its members must be types. So a lifetime
parameter on the trait must first become a type. `Life<'a>` is that conversion. It packages the lifetime
`'a` as a concrete type that can stand beside the trait's other type parameters.

Without this lift, a trait that borrows could not record its lifetime in the dependency marker.
`Life` lets the lifetime pass through `IsProviderFor` as `(Life<'a>, T)`, so it stays part of the
provider's identity while the marker's argument stays a plain type. The macros insert it, so you
read it in generated code and in errors, and you write it in one place: the
[`check_components!`](../macros/check_components.md) entry for a component with a lifetime.

## Definition

`Life` is a tuple struct wrapping a single phantom marker over a raw pointer to a borrowed unit:

```rust
pub struct Life<'a>(pub PhantomData<*mut &'a ()>);
```

The struct holds nothing at run time. Its only job is to carry the lifetime `'a` in the type system
through [`PhantomData`](phantom_data.md). It is in the prelude, so `use cgp::prelude::*;` is enough.

The phantom type `*mut &'a ()` decides how `Life<'a>` behaves under subtyping. A `*mut T` is
*invariant* in `T`, so wrapping `&'a ()` behind a `*mut` makes `Life<'a>` invariant in `'a`: a
`Life<'long>` is neither a subtype nor a supertype of a `Life<'short>`. The lifetime is therefore an
exact identity wherever subtyping could apply, as in a provider struct holding a
`PhantomData<Life<'a>>`. Trait resolution itself matches lifetimes exactly whatever the variance, so
the choice affects values and types that contain `Life`, not which impl the compiler selects.

The raw pointer has a second effect: `Life<'a>` is neither `Send` nor `Sync`. Neither is any struct
holding a `PhantomData<Life<'a>>`, such as the provider struct `#[cgp_impl(new …)]` declares for a
provider generic over a lifetime. This rarely matters, because `Life` and provider structs appear
only at the type level and are never sent between threads, but a bound such as `Provider: Send` on
such a provider fails with
``error[E0277]: `*mut &'static ()` cannot be sent between threads safely``.

## Behavior

`Life` does not define methods and does not implement traits of its own. Its entire behavior is to
occupy a type position. In the generated provider trait for a component with a lifetime, the macro
collects the lifetime into the [`IsProviderFor`](../traits/wiring/is_provider_for.md) argument tuple
as `Life<'a>`, so the provider's dependency obligation reads the same way as for any type parameter.
The provider trait, its blanket forwarding impl, and the impls that satisfy it all agree on the same
`(Life<'a>, T)` shape. That shared shape lets CGP wire and check a borrowing component like a
non-borrowing one.

A check names the shape itself. A `check_components!` entry lists a component's parameters in the
form the marker records them, so a component with a lifetime is checked as `(Life<'a>, T)`, with the
lifetime declared on the table, as in
`<'a> App<'a> { ReferenceGetterComponent: (Life<'a>, Config) }`. A bare `('a, Config)` is rejected:
the compiler reads `'a` as a trait-object type without a trait and reports
`error: at least one trait is required for an object type`.

## Examples

A borrowing getter component, wired on a context that holds a borrow, and checked with `Life` in its
parameter tuple:

```rust
use cgp::prelude::*;

#[cgp_component(ReferenceGetter)]
pub trait HasReference<'a, T: 'a + ?Sized> {
    fn get_reference(&self) -> &'a T;
}

pub struct Config {
    pub name: String,
}

#[cgp_impl(new GetConfig)]
#[uses(HasField<Symbol!("config"), Value = &'a Config>)]
impl<'a> ReferenceGetter<'a, Config> {
    fn get_reference(&self) -> &'a Config {
        self.get_field(PhantomData::<Symbol!("config")>)
    }
}

#[derive(HasField)]
pub struct App<'a> {
    pub config: &'a Config,
}

delegate_components! {
    <'a> App<'a> {
        ReferenceGetterComponent: GetConfig,
    }
}

check_components! {
    <'a> App<'a> {
        ReferenceGetterComponent: (Life<'a>, Config),
    }
}

pub fn demo() {
    let config = Config {
        name: "demo".to_owned(),
    };
    let app = App { config: &config };

    let config_ref: &Config = app.get_reference();
    assert_eq!(config_ref.name, "demo");
}
```

`App` is an environmental context: it holds the borrowed configuration a provider reads, and the
component is parameter-targeted, returning a borrow of `T` for the lifetime `'a`. The generated
provider trait records the lifetime in its dependency marker as the type `Life<'a>`, as
`cargo cgp expand` shows:

```rust
pub trait ReferenceGetter<
    'a,
    __Context__,
    T: 'a + ?Sized,
>: IsProviderFor<ReferenceGetterComponent, __Context__, (Life<'a>, T)> {
    fn get_reference(__context__: &__Context__) -> &'a T;
}
```

Every impl that satisfies this component, whether through `UseContext`, a table's forwarding impl,
or a provider you write yourself, carries the same `(Life<'a>, T)` tuple, so the resolution
machinery preserves the lifetime end to end.

## When to use it

**You read `Life` in generated code, and you write it only in a check.** The macros insert it
everywhere else.

- **Read `Life<'a>` in an `IsProviderFor` tuple as the component's lifetime.** A dependency marker such
  as `IsProviderFor<..., (Life<'a>, T)>` names a lifetime and a type parameter, in that order.
- **Write `(Life<'a>, T)` in a `check_components!` entry** for a component with a lifetime, and
  declare `'a` in the table's generic list.
- **Hold a lifetime as `PhantomData<Life<'a>>`** when you declare a provider struct by hand and want
  the shape `#[cgp_new_provider]` gives it. A plain [`PhantomData<&'a ()>`](phantom_data.md) also
  compiles, but it is covariant and keeps the struct `Send`.

## Common Mistakes

**A lifetime cannot appear directly in the `IsProviderFor` tuple.** The tuple holds types, so a bare
`'a` is invalid there, and the macro lifts it into `Life<'a>`. `Life` in an error means that the component
carries a lifetime.

**A component whose target is unsized passes its check but fails at the call.** The `?Sized` bound
on `T` above admits `str`, and wiring such a component at `str` compiles and checks:

```rust
#[cgp_impl(new GetName)]
#[uses(HasField<Symbol!("name"), Value = &'a str>)]
impl<'a> ReferenceGetter<'a, str> {
    fn get_reference(&self) -> &'a str {
        self.get_field(PhantomData::<Symbol!("name")>)
    }
}

#[derive(HasField)]
pub struct Borrowed<'a> {
    pub name: &'a str,
}

delegate_components! {
    <'a> Borrowed<'a> {
        ReferenceGetterComponent: GetName,
    }
}

check_components! {
    <'a> Borrowed<'a> {
        ReferenceGetterComponent: (Life<'a>, str),
    }
}
```

A call such as `let name: &str = borrowed.get_reference();` then fails with
``error[E0599]: the method `get_reference` exists for struct `Borrowed<'_>`, but its trait bounds were not satisfied``,
whose note names the unmet bound:

```text
   = note: the following trait bounds were not satisfied:
           `str: Sized`
           which is required by `Borrowed<'_>: HasReference<'_, str>`
```

The parameter tuple `(Life<'a>, str)` is unsized, and the forwarding impl that
[`delegate_components!`](../macros/delegate_components.md) emits for the table requires its
parameter tuple to be `Sized`, while the check goes to the provider's own impl and passes. This is a
defect in the library. Until it is fixed, give such a component a sized target, as `Config` is
above, or implement the consumer trait directly on the context.

**A [higher-order provider](/docs/reference/glossary#higher-order-provider) with a lifetime loses its dependency propagation.** When the component carries a
lifetime, the inner-provider bound of such a stack does not get a marker counterpart, because the rewrite
reads the bound's first generic argument as the context and finds a lifetime there. The stack still
compiles and runs. But it loses the propagation that lets `#[check_providers]` localize a broken layer.
[`IsProviderFor`](../traits/wiring/is_provider_for.md) records this limitation.

## Related constructs

- [`IsProviderFor`](../traits/wiring/is_provider_for.md): whose parameter tuple a lifetime is lifted
  into as `Life<'a>`.
- [`#[cgp_component]`](../macros/cgp_component.md): inserts `Life` when a consumer trait carries a
  lifetime.
- [`check_components!`](../macros/check_components.md): where a check names `Life` in a parameter
  tuple.
- [`PhantomData`](phantom_data.md): the marker `Life` is built from, wrapped for invariance.
- [`Index`](index_type.md) and [`Chars`](chars.md): the other lifts that make a non-type
  addressable in trait resolution, a number and a string where `Life` lifts a lifetime.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): the provider-trait
  machinery whose dependency marker carries this lift.

## Source

- The type:
  [`life.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/life.rs)
- The macro logic that wraps a trait's lifetime parameters in `Life` when building the
  `IsProviderFor` argument tuple:
  [`is_provider_params.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/functions/is_provider_params.rs),
  with the same lifting in a provider struct's `PhantomData` in
  [`empty_struct.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/empty_struct.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
