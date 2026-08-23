---
sidebar_label: 'HasErrorType'
sidebar_position: 1
---

# `HasErrorType`

Give a context one shared, abstract `Error` type, so fallible generic code never names a concrete error.

## Overview

`HasErrorType` lets generic CGP code fail without committing to a concrete error type. A provider that
may error has to produce *some* error, but it runs against a **context**, the type a capability runs
against that supplies the values an implementation needs, and it cannot know whether that context wants
`anyhow::Error`, `std::io::Error`, or an enum of its own. `HasErrorType` resolves this by giving the
context one abstract `Self::Error` type that every fallible operation refers to. Generic code returns
`Result<T, Self::Error>`, and the concrete error is decided once, at wiring time, by whichever error
backend the context plugs in.

Putting the error type on one component also keeps errors composable. If each fallible trait declared
its own associated `Error`, a context bounded by several of them would face several unrelated `Self::Error`
types with no way to unify them. Because the fallible components all supertrait `HasErrorType`, every one
of them names the *same* `Self::Error`, so their results combine. This single shared abstract error is
the anchor that [`CanRaiseError`](./can_raise_error.md) and [`CanWrapError`](./can_wrap_error.md) build
on.

## Definition

`HasErrorType` is defined as:

```rust
#[cgp_type]
#[prefix(@cgp.core.error in DefaultNamespace)]
pub trait HasErrorType {
    type Error: Debug;
}

pub type ErrorOf<Context> = <Context as HasErrorType>::Error;
```

Its attributes:

- [`#[cgp_type]`](../macros/cgp_type.md) — makes this an abstract-type component rather than a plain trait: it generates the provider trait, the component marker, and a [`UseType`](../providers/use_type.md) impl, so a context binds the concrete type by wiring.
- [`#[prefix]`](../macros/cgp_namespace.md) — registers the generated names into the `@cgp.core.error` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.

## Usage

`HasErrorType` is in the prelude, so `use cgp::prelude::*;` is enough to name it. It is an
abstract-type component: a
[trait](https://doc.rust-lang.org/book/ch10-02-traits.html) with one
[associated type](https://doc.rust-lang.org/reference/items/associated-items.html), `Error`, carrying a
`Debug` bound. A context supplies its error type in one of two ways.

The direct way is an ordinary trait impl, which shows that `HasErrorType` is a plain associated-type
trait:

```rust
impl HasErrorType for App {
    type Error = anyhow::Error;
}
```

The wired way, and the common one, delegates the component's key to a provider that supplies the type.
Because `HasErrorType` is defined with [`#[cgp_type]`](../macros/cgp_type.md), a context can name the
concrete error directly with [`UseType<E>`](../providers/use_type.md):

```rust
delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
    }
}
```

The wiring key is `ErrorTypeProviderComponent`, not the trait name, and it lives under
`cgp::core::error` rather than in the prelude, so a context that wires it imports the key. Whatever
concrete error is chosen must implement `Debug`, because that bound is carried through every generated
construct. The standalone backend crates `cgp-error-anyhow`, `cgp-error-eyre`, and `cgp-error-std`
supply ready-made providers that set `Error` to their respective types.

Generic code names the abstract error with the [`#[use_type]`](../attributes/use_type.md) attribute,
which imports it as the bare name `Error`, or through the `ErrorOf<Context>` alias, which spells
`<Context as HasErrorType>::Error`.

## Examples

A context declares its abstract error, and generic code returns it without naming a concrete type:

```rust
use cgp::prelude::*;

#[cgp_component(Validator)]
#[use_type(HasErrorType.Error)]
pub trait CanValidate {
    fn validate(&self) -> Result<(), Error>;
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
    }
}
```

Here [`#[use_type(HasErrorType.Error)]`](../attributes/use_type.md) adds `HasErrorType` as a supertrait
of `CanValidate` and rewrites the bare `Error` to `<Self as HasErrorType>::Error`, so `validate` returns
the context's shared error without spelling `Self::Error`. `App` wires its error type to `String`, which
satisfies the `Debug` bound. `App` is an **environmental context**, a type that stands for the
application and carries its choices, and the capability targets that context rather than a value.

## When to use it

**Reach for `HasErrorType` in any fallible component whose error type the context should choose.** It is
the foundation of CGP's error handling and is a supertrait of every other fallible capability, so a
component that returns a `Result` almost always wants the context's abstract error rather than a
concrete one. Import it with [`#[use_type(HasErrorType.Error)]`](../attributes/use_type.md) rather than
writing a `: HasErrorType` supertrait and a `Self::Error` path by hand.

Do not reach for it when a component genuinely wants one fixed error type for the whole program with no
per-context choice, where a concrete error in the signature is simpler. And where a provider must pin
the abstract error to a concrete type, use the equality form
[`#[use_type(HasErrorType.{Error = AppError})]`](../attributes/use_type.md) rather than a hand-written
`where Self: HasErrorType<Error = AppError>` clause.

## Related constructs

- [`#[cgp_type]`](../macros/cgp_type.md) — the macro `HasErrorType` is defined with, which generates its
  `UseType` provider.
- [`HasType` / `TypeProvider`](./has_type.md) — the built-in abstract-type substrate `#[cgp_type]`
  builds this on.
- [`CanRaiseError`](./can_raise_error.md) — raises a source error into this abstract error.
- [`CanWrapError`](./can_wrap_error.md) — wraps detail onto this abstract error.
- [`#[use_type]`](../attributes/use_type.md) — imports the abstract error into a definition as the bare
  name `Error`.
- [Error providers](../providers/error/index.md) — the interchangeable strategies that construct the
  error.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — the abstract error type, its
  construction, and its detail as three independent wiring decisions.
- [Abstract types](/docs/concepts/abstract-types) — the associated types a context chooses for itself.

## Source

- The trait and the `ErrorOf` alias:
  [`has_error_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-error/src/traits/has_error_type.rs)
- The `#[cgp_type]` machinery it relies on:
  [`cgp_type/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_type/)
- The pluggable concrete error backends:
  [`standalone/error/`](https://github.com/contextgeneric/cgp/tree/main/crates/standalone/error/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
