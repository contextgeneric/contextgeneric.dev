---
sidebar_label: 'CanWrapError'
sidebar_position: 3
---

# `CanWrapError`

Attach a piece of detail to an error the context already holds, enriching it as it propagates.

## Overview

`CanWrapError<Detail>` enriches an error as it travels up a call stack. Where
[`CanRaiseError`](./can_raise_error.md) converts a *foreign* error into the context's abstract error,
`CanWrapError` takes an error the **context** already holds and folds a piece of `Detail` into it, a
message, a span, a path, producing an enriched `Self::Error`. The context is the type a capability runs
against, which supplies the values an implementation needs as its own fields, and it decides how detail
is combined with an existing error. Together the two components cover the common error-handling motions
in CGP: raise a foreign error in, then wrap context onto it as it bubbles up.

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
    fn wrap_error(error: Error, detail: Detail) -> Error;
}
```

Its attributes:

- [`#[cgp_component]`](../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `ErrorWrapper` that implementations target and the wiring key `ErrorWrapperComponent`, while `CanWrapError` stays the consumer trait callers use.
- [`#[prefix]`](../macros/cgp_namespace.md) — registers the generated names into the `@cgp.core.error` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates a `UseDelegate` provider that dispatches on the `Detail` type, so a context can route each `Detail` to its own provider; the `open` statement is the modern sugar for the same dispatch.
- [`#[use_type]`](../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`CanWrapError` is in the prelude. Its method is an associated function that takes the context's current
error plus a detail and returns a new error with the detail folded in:

```rust
fn wrap_error(error: Error, detail: Detail) -> Error;
```

A context gains the capability by wiring `ErrorWrapperComponent`, whose key lives under
`cgp::core::error`, to a provider. The trait dispatches per detail type, so the natural wiring is a
table keyed on `Detail`, most idiomatically an [`open` statement](../macros/delegate_components.md):

```rust
delegate_components! {
    App {
        open ErrorWrapperComponent;

        @ErrorWrapperComponent.String: DisplayError,
    }
}
```

The [error providers](../providers/error/index.md) supply the strategies that satisfy it, and the
standalone backend crates (`cgp-error-anyhow`, `cgp-error-eyre`, `cgp-error-std`) wire wrapping for
common detail types, so an application usually plugs in a backend rather than writing wrap logic itself.
Because `wrap_error` is an associated function, generic code calls it on the context type,
`Context::wrap_error(err, detail)`, without borrowing a context value.

## Examples

A provider wraps a message onto an error as it propagates:

```rust
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
```

The provider `LoadOrFail` first raises a `String` into the context's abstract error with
[`CanRaiseError`](./can_raise_error.md), then wraps a further message onto it with `CanWrapError`. Both
dependencies are declared with [`#[uses]`](../attributes/uses.md), so neither appears on the public
`CanLoad` signature, and any context that satisfies them makes `load` produce enriched errors in its own
error type. The context is an **environmental context**, and the capability targets it.

## When to reach for it, and when not

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

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
