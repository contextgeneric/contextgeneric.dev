---
title: 'CanWrapError — add detail to an error'
description: 'The component that folds a piece of detail, such as a message or a path, into an error the context already holds, dispatching per detail type.'
sidebar_label: 'CanWrapError'
sidebar_position: 3
---

# `CanWrapError`

Attach a piece of detail to an error the context already holds, enriching it as it propagates.

## Overview

`CanWrapError<Detail>` enriches an error as it travels up a call stack. Where
[`CanRaiseError`](./can_raise_error.md) converts a *foreign* error into the context's abstract
error, `CanWrapError` takes an error the **context** already holds and folds a piece of `Detail`
into it, a message, a span, a path, producing an enriched `Self::Error`. The context is the type the
implementation runs against, and it decides how detail is combined with an existing error. Together
the two components cover the common error-handling motions in CGP: raise a foreign error in, then
wrap context onto it as it bubbles up.

Because the trait is parameterized by `Detail`, one context can attach many kinds of detail, each
through its own provider, all onto the same abstract error. Like its companion, `CanWrapError` builds on
[`HasErrorType`](./has_error_type.md), which supplies the `Self::Error` it takes and returns.

## Definition

`CanWrapError` is defined as:

```rust
#[cgp_component(ErrorWrapper)]
#[prefix(@cgp.core.error in DefaultNamespace)]
#[derive_delegate(UseDelegate<Detail>)]
#[use_type(HasErrorType.Error)]
pub trait CanWrapError<Detail> {
    #[track_caller]
    fn wrap_error(error: Error, detail: Detail) -> Error;
}
```

Its attributes:

- [`#[cgp_component]`](../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `ErrorWrapper` that implementations target and the wiring key `ErrorWrapperComponent`, while `CanWrapError` stays the consumer trait callers use.
- [`#[prefix]`](../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.core.error`, so a context that joins that namespace binds its provider at `@cgp.core.error.ErrorWrapperComponent.String` rather than at the bare key.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates a `UseDelegate` provider that dispatches on the `Detail` type, so a context can route each `Detail` to its own provider; the `open` statement is the modern sugar for the same dispatch.
- [`#[use_type]`](../attributes/use_type.md) — adds `HasErrorType` as a [supertrait](/docs/reference/glossary#supertrait) and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`CanWrapError` is in the prelude. Its method is an associated function that takes the context's current
error plus a detail and returns a new error with the detail folded in:

```rust
fn wrap_error(error: Error, detail: Detail) -> Error;
```

A context gains the operation by wiring `ErrorWrapperComponent`, whose key lives under
`cgp::core::error`, to a provider. The trait dispatches per detail type, so the natural wiring is a
table keyed on `Detail`, most idiomatically an [`open` statement](../macros/delegate_components.md):

```rust
delegate_components! {
    App {
        open ErrorWrapperComponent;

        @ErrorWrapperComponent.String: AppendDetail,
        @ErrorWrapperComponent.u64: DisplayError,
    }
}
```

`AppendDetail` is a provider of the application's own that folds a `String` detail into the error;
[Examples](#examples) defines it. [`DisplayError`](../providers/error/display_error.md) formats any
`Display` detail into a `String` and forwards it to the context's own `CanWrapError<String>`, so here a
`u64` detail reaches `AppendDetail` too. The forwarding is why `DisplayError` cannot serve the `String`
entry itself: `@ErrorWrapperComponent.String: DisplayError` sends the lookup back to the same entry, and
the check overflows:

```text
error[E0275]: overflow evaluating the requirement `App: IsProviderFor<ErrorWrapperComponent, App, String>`
```

The [error providers](../providers/error/index.md) supply the strategies that satisfy it, and the
standalone backend crates (`cgp-error-anyhow`, `cgp-error-eyre`, `cgp-error-std`) wire wrapping for
common detail types, so an application usually plugs in a backend rather than writing wrap logic itself.
Because `wrap_error` is an associated function, generic code calls it on the context type,
`Context::wrap_error(err, detail)`, without borrowing a context value.

Calling it on a concrete context by its bare name, as `App::wrap_error(…)`, is ambiguous when the
provider trait `ErrorWrapper` is also in scope, because a context implements the provider trait too.
The compiler reports ``error[E0034]: multiple applicable items in scope``. Name the consumer trait
in that case, as `<App as CanWrapError<String>>::wrap_error(…)`, or leave `ErrorWrapper` unimported.

**`wrap_error` carries `#[track_caller]`, and the attribute survives every layer of forwarding.**
Rust applies `#[track_caller]` on a trait method declaration to every implementation of that method,
and `#[cgp_component]` keeps it on the provider trait's declaration. It therefore covers the
consumer and provider blanket implementations, the `UseDelegate`, `RedirectLookup`, and `UseContext`
implementations, and every provider. An error library that records `Location::caller()`, such as
`eyre` with its `track-caller` feature, records the line that called `wrap_error`, whether the
component is wired directly, with `open`, through a namespace path, or through a `UseDelegate`
table. The location survives only while every call between that line and the library is
`#[track_caller]`: a generic helper function without the attribute records its own line instead.

## Examples

A provider wraps a message onto an error as it propagates:

```rust
use cgp::core::error::{
    ErrorRaiserComponent, ErrorTypeProviderComponent, ErrorWrapper, ErrorWrapperComponent,
};
use cgp::extra::error::RaiseFrom;
use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}

#[cgp_impl(new LoadOrFail)]
#[uses(CanRaiseError<String>, CanWrapError<String>)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self, path: &str) -> Result<String, Error> {
        if path.is_empty() {
            let err = Self::raise_error("empty path".to_owned());
            return Err(Self::wrap_error(err, format!("while loading {path}")));
        }
        Ok(format!("contents of {path}"))
    }
}

#[cgp_impl(new AppendDetail)]
#[use_type(HasErrorType.{Error = String})]
impl ErrorWrapper<String> {
    fn wrap_error(error: Error, detail: String) -> Error {
        format!("{detail}: {error}")
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ErrorRaiserComponent: RaiseFrom,
        ErrorWrapperComponent: AppendDetail,
        LoaderComponent: LoadOrFail,
    }
}

check_components! {
    App {
        LoaderComponent,
    }
}
```

`App.load("")` returns `Err("while loading : empty path")`. `AppendDetail` pins the abstract error to
`String` with the equality form of [`#[use_type]`](../attributes/use_type.md), since it builds a
`String` directly.

The provider `LoadOrFail` first raises a `String` into the context's abstract error with
[`CanRaiseError`](./can_raise_error.md), then wraps a further message onto it with `CanWrapError`. Both
dependencies are declared with [`#[uses]`](../attributes/uses.md), so neither appears on the public
`CanLoad` signature, and any context that satisfies them makes `load` produce enriched errors in its own
error type. `App` here satisfies them with `RaiseFrom` and `AppendDetail`. The context is an **[environmental context](/docs/reference/glossary#environmental-context)**, and the component targets it.

## When to use it

**Reach for `CanWrapError<D>` when a provider should add context to an error before returning it.** It
is how a CGP program builds the equivalent of an error chain or a `.context(...)` message without
committing to a concrete error library, and it pairs naturally with
[`CanRaiseError`](./can_raise_error.md) in a provider that both converts and enriches. Declare it with
[`#[uses(CanWrapError<D>)]`](../attributes/uses.md) so the dependency stays off the consumer trait.

Do not reach for it when there is no detail worth attaching, where raising the error is enough, or when
the context's error type already captures the surrounding context by other means.

## Related constructs

- [`CanRaiseError`](./can_raise_error.md) — the companion that converts a foreign error into the abstract
  one.
- [`HasErrorType`](./has_error_type.md) — the supertrait supplying the `Self::Error` this enriches.
- [Error providers](../providers/error/index.md) — the strategies that satisfy this component.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates the `UseDelegate` dispatch this
  uses.
- [`#[uses]`](../attributes/uses.md) — the idiomatic way a provider declares this dependency.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — the error type, its construction,
  and its detail as three independent wiring decisions.

## Source

- The trait:
  [`can_wrap_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-error/src/traits/can_wrap_error.rs)
- The abstract error it builds on:
  [`has_error_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-error/src/traits/has_error_type.rs)
- The pluggable providers that implement it:
  [`standalone/error/`](https://github.com/contextgeneric/cgp/tree/main/crates/standalone/error/)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
