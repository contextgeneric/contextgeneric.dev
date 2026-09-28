---
title: 'CanRaiseError — raise into the abstract error'
description: 'The component that converts a concrete source error into the context''s abstract error type, dispatching per source error to the provider the context wires.'
sidebar_label: 'CanRaiseError'
sidebar_position: 2
---

# `CanRaiseError`

Turn a concrete source error into the context's abstract `Self::Error`.

## Overview

`CanRaiseError<SourceError>` lets generic CGP code produce its context's abstract error from any
concrete error it meets. A provider that calls a fallible operation gets back a specific error type,
a parse error, an I/O error, a string message, but it must return the **context's** abstract
`Self::Error`, whose concrete identity it does not know. The context here is the type the
implementation runs against, and it decides how each source error maps into its chosen error type.
`CanRaiseError<SourceError>` bridges the gap: generic code writes `Context::raise_error(source)` and
the context converts the concrete `SourceError` into `Self::Error`.

Because the trait is parameterized by `SourceError`, one context can know how to raise many different
source errors into its single abstract error. This is the "raise" half of CGP's error handling; the
"wrap" half, attaching detail to an error the context already holds, is [`CanWrapError`](./can_wrap_error.md).
Both build on [`HasErrorType`](./has_error_type.md), which supplies the `Self::Error` they produce.

## Definition

`CanRaiseError` is defined as:

```rust
#[cgp_component(ErrorRaiser)]
#[prefix(@cgp.core.error in DefaultNamespace)]
#[derive_delegate(UseDelegate<SourceError>)]
#[use_type(HasErrorType.Error)]
pub trait CanRaiseError<SourceError> {
    #[track_caller]
    fn raise_error(error: SourceError) -> Error;
}
```

Its attributes:

- [`#[cgp_component]`](../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `ErrorRaiser` that implementations target and the wiring key `ErrorRaiserComponent`, while `CanRaiseError` stays the consumer trait callers use.
- [`#[prefix]`](../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.core.error`, so a context that joins that namespace binds its provider at `@cgp.core.error.ErrorRaiserComponent.String` rather than at the bare key.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates a `UseDelegate` provider that dispatches on the `SourceError` type, so a context can route each `SourceError` to its own provider; the `open` statement is the modern sugar for the same dispatch.
- [`#[use_type]`](../attributes/use_type.md) — adds `HasErrorType` as a [supertrait](/docs/reference/glossary#supertrait) and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`CanRaiseError` is in the prelude. It carries the concrete error type as a parameter, and its method is
an associated function rather than a `&self` method, because raising an error is a property of the
context type rather than of any particular value:

```rust
fn raise_error(error: SourceError) -> Error;
```

A context gains the operation by wiring `ErrorRaiserComponent`, whose key lives under
`cgp::core::error`, to a provider. Because the trait dispatches per source-error type, the natural
wiring is a table keyed on `SourceError`, most idiomatically an
[`open` statement](../macros/delegate_components.md) that routes each concrete error to a provider that
knows how to handle it:

```rust
delegate_components! {
    App {
        open ErrorRaiserComponent;

        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseError: DebugError,
    }
}
```

`ParseError` stands for any source error type the application meets. The key comes from
`cgp::core::error` and the providers from `cgp::extra::error`.

The interchangeable strategies that satisfy `CanRaiseError` are the
[error providers](../providers/error/index.md): `RaiseFrom` converts through `From`, `DebugError` and
`DisplayError` format the source into a string, `ReturnError` returns it unchanged, and so on. The
standalone backend crates (`cgp-error-anyhow`, `cgp-error-eyre`, `cgp-error-std`) supply providers for
common cases, so an application usually wires a backend rather than writing raise logic itself.

Because `raise_error` is an associated function, generic code calls it on the context *type*,
`Context::raise_error(source)` or `Self::raise_error(source)`, without borrowing a context value. This
matches how errors are constructed deep inside generic code where only the type parameter is in scope.

Calling it on a concrete context by its bare name, as `App::raise_error(…)`, is ambiguous when the
provider trait `ErrorRaiser` is also in scope, because a context implements the provider trait too.
The compiler reports ``error[E0034]: multiple applicable items in scope``. Name the consumer trait
in that case, as `<App as CanRaiseError<String>>::raise_error(…)`, or leave `ErrorRaiser`
unimported.

**`raise_error` carries `#[track_caller]`, and the attribute survives every layer of forwarding.**
Rust applies `#[track_caller]` on a trait method declaration to every implementation of that method,
and `#[cgp_component]` keeps it on the provider trait's declaration. It therefore covers the
consumer and provider blanket implementations, the `UseDelegate`, `RedirectLookup`, and `UseContext`
implementations, and every provider. An error library that records `Location::caller()`, such as
`eyre` with its `track-caller` feature, records the line that called `raise_error`, whether the
component is wired directly, with `open`, through a namespace path, or through a `UseDelegate`
table. The location survives only while every call between that line and the library is
`#[track_caller]`: a generic helper function without the attribute records its own line instead.

## Examples

A provider raises a concrete error into the context's abstract one:

```rust
use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}

#[cgp_impl(new LoadOrFail)]
#[uses(CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self, path: &str) -> Result<String, Error> {
        if path.is_empty() {
            return Err(Self::raise_error("empty path".to_owned()));
        }
        Ok(format!("contents of {path}"))
    }
}
```

The provider `LoadOrFail` names neither the context nor its concrete error type. It requires
`CanRaiseError<String>` through [`#[uses]`](../attributes/uses.md) to turn a `String` message into the
abstract error, and any wired context that satisfies that bound, typically by plugging in an error
backend, makes `load` produce errors in that context's chosen type. The context is an
**[environmental context](/docs/reference/glossary#environmental-context)**, and the component targets
it rather than a value.

## When to use it

**Reach for `CanRaiseError<E>` whenever a provider holds a concrete error `E` and must return the
context's abstract error.** It is the standard way a fallible provider bridges from a library's error
type to the context's own, and declaring it with [`#[uses(CanRaiseError<E>)]`](../attributes/uses.md)
keeps the dependency off the consumer trait's public signature. Where a provider also needs to attach a
message or other detail as the error propagates, pair it with [`CanWrapError`](./can_wrap_error.md).

Do not reach for it when the source error already *is* the context's error type, where returning it
directly is simpler, or when a component fixes one concrete error type for the whole program with no
per-context choice.

## Related constructs

- [`CanWrapError`](./can_wrap_error.md) — the companion that attaches detail to an existing error.
- [`HasErrorType`](./has_error_type.md) — the supertrait supplying the `Self::Error` this produces.
- [Error providers](../providers/error/index.md) — `RaiseFrom`, `DebugError`, and the other strategies
  that satisfy this component.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates the `UseDelegate` dispatch this
  uses.
- [`#[uses]`](../attributes/uses.md) — the idiomatic way a provider declares this dependency.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — the error type, its construction,
  and its detail as three independent wiring decisions.

## Source

- The trait:
  [`can_raise_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-error/src/traits/can_raise_error.rs)
- The abstract error it builds on:
  [`has_error_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-error/src/traits/has_error_type.rs)
- The pluggable providers that implement it:
  [`standalone/error/`](https://github.com/contextgeneric/cgp/tree/main/crates/standalone/error/)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
