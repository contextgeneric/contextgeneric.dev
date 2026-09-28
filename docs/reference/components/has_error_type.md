---
title: 'HasErrorType — the context''s error type'
description: 'The component that gives a context one abstract Error type, so fallible generic code never names a concrete error and the context chooses it by wiring.'
sidebar_label: 'HasErrorType'
sidebar_position: 1
---

# `HasErrorType`

Give a context one shared, abstract `Error` type, so fallible generic code never names a concrete error.

## Overview

`HasErrorType` lets generic CGP code fail without committing to a concrete error type. A provider
that may error has to produce *some* error, but it runs against a **context**, the type that
implements the trait, and it cannot know whether that context wants `anyhow::Error`,
`std::io::Error`, or an enum of its own. `HasErrorType` resolves this by giving the context one
abstract `Self::Error` type that every fallible operation refers to. Generic code returns
`Result<T, Self::Error>`, and the concrete error is decided once, at wiring time, by whichever error
backend the context plugs in.

Putting the error type on one component also keeps errors composable. If each fallible trait declared
its own associated `Error`, a context bounded by several of them would face several unrelated `Self::Error`
types with no way to unify them. Because the fallible components all [supertrait](/docs/reference/glossary#supertrait) `HasErrorType`, every one
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

- [`#[cgp_type]`](../macros/cgp_type.md) — makes this an abstract-type component rather than a plain trait: it generates the provider trait, the [component marker](/docs/reference/glossary#component-marker), and a [`UseType`](../providers/use_type.md) impl, so a context binds the concrete type by wiring.
- [`#[prefix]`](../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.core.error`, so a context that joins that namespace binds its provider at `@cgp.core.error.ErrorTypeProviderComponent` rather than at the bare key.

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
`<Context as HasErrorType>::Error`. `ErrorOf`, the provider trait `ErrorTypeProvider`, and the key
`ErrorTypeProviderComponent` all come from `cgp::core::error`.

The same module carries `ErrorOnly<E>`, a zero-sized context whose one trait is `HasErrorType`, with
`Error = E` for any `E: Debug`. It implements `Default`, so `ErrorOnly::<String>::default()` stands
in wherever code needs a context that has an error type and nothing else, such as a test of a
fallible function that reads nothing from its context:

```rust
use cgp::core::error::{ErrorOf, ErrorOnly};
use cgp::prelude::*;

pub fn parse_port<Context: HasErrorType<Error = String>>(
    _context: &Context,
    raw: &str,
) -> Result<u16, ErrorOf<Context>> {
    raw.parse().map_err(|_| format!("bad port: {raw}"))
}

pub fn demo() {
    let context = ErrorOnly::<String>::default();
    assert_eq!(parse_port(&context, "80"), Ok(80));
    assert_eq!(parse_port(&context, "x"), Err("bad port: x".to_owned()));
}
```

## Examples

A context declares its abstract error, and generic code returns it without naming a concrete type:

```rust
use cgp::core::error::ErrorTypeProviderComponent;
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

check_components! {
    App {
        ErrorTypeProviderComponent,
    }
}
```

Here [`#[use_type(HasErrorType.Error)]`](../attributes/use_type.md) adds `HasErrorType` as a supertrait
of `CanValidate` and rewrites the bare `Error` to `<Self as HasErrorType>::Error`, so `validate` returns
the context's shared error without spelling `Self::Error`. `App` wires its error type to `String`, which
satisfies the `Debug` bound. `App` is an **[environmental context](/docs/reference/glossary#environmental-context)**, a type that stands for the
application and carries its choices, and the component targets that context rather than a value.

## When to use it

**Reach for `HasErrorType` in any fallible component whose error type the context should choose.** It is
the foundation of CGP's error handling and is a supertrait of every other fallible trait, so a
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
- [`HasType` / `TypeProvider`](./has_type.md) — the tag-indexed abstract-type component; a
  `TypeProvider` can back this component through the `WithProvider` impl `#[cgp_type]` generates.
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
- `ErrorOnly`:
  [`error_only.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-error/src/contexts/error_only.rs)
- The `#[cgp_type]` machinery it relies on:
  [`cgp_type/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_type/)
- The pluggable concrete error backends:
  [`standalone/error/`](https://github.com/contextgeneric/cgp/tree/main/crates/standalone/error/)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
