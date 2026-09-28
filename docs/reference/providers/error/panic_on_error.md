---
title: 'PanicOnError — panic instead of raising'
description: 'The error raiser that panics with the source error''s Debug output rather than returning an error value, for tests and fail-fast tools.'
sidebar_label: 'PanicOnError'
sidebar_position: 7
---

# `PanicOnError`

Abort with the source error's debug output instead of producing an abstract error value.

## Overview

`PanicOnError` is the `ErrorRaiser` provider that panics rather than returning an error. When a
[**context**](/docs/reference/glossary#context), the type the implementation runs against, treats an
error as a programming fault that should stop the program, `PanicOnError` raises by calling `panic!`
with the source error's debug representation. Its signature promises to return the abstract error,
but the body never does, because `panic!` diverges. This suits tests and fail-fast tooling. Like every
CGP provider, `PanicOnError` carries no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorRaiserComponent` from
`cgp::core::error`; neither is in the prelude. It takes no type parameter and accepts any `Debug`
source:

```rust
use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::PanicOnError;

delegate_components! {
    TestApp {
        ErrorRaiserComponent: PanicOnError,
    }
}
```

Any call to `Context::raise_error(source)` on `TestApp` panics with `format!("{source:?}")` rather
than returning. The context still wires an error type, because the provider's signature names
`Context::Error` and its `where` clause requires `HasErrorType`.

## Examples

A test context wires every raise to `PanicOnError`, so a provider that fails aborts the test with the
source's debug output:

```rust
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::PanicOnError;

#[cgp_component(Runner)]
#[use_type(HasErrorType.Error)]
pub trait CanRun {
    fn run(&self) -> Result<(), Error>;
}

#[cgp_impl(new RunOrAbort)]
#[uses(CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl Runner {
    fn run(&self) -> Result<(), Error> {
        Err(Self::raise_error("unrecoverable".to_owned()))
    }
}

pub struct TestApp;

delegate_components! {
    TestApp {
        ErrorTypeProviderComponent: UseType<String>,
        ErrorRaiserComponent: PanicOnError,
        RunnerComponent: RunOrAbort,
    }
}

check_components! {
    TestApp {
        RunnerComponent,
    }
}

pub fn demo() {
    let _ = TestApp.run(); // panics with "\"unrecoverable\""
}
```

`RunOrAbort` raises a `String` and would return it as an error on another context. `TestApp`, an
[environmental context](/docs/reference/glossary#environmental-context) for tests, wires the raise
to `PanicOnError`, so `run` panics with the `Debug` form of the string, quotes included, and never
returns.

## When to use it

**Reach for `PanicOnError` where an error means a bug that should abort rather than be handled**, such
as a test harness or a fail-fast tool. It spares the context a handling strategy, though not an error
type.

Do not wire it in production code that should recover from errors. Use [`RaiseFrom`](raise_from.md) or
a backend provider there, so a raised error becomes a value the caller can handle.

## Under the hood

`PanicOnError` implements `ErrorRaiser<Context, E>` for any `Debug` source over any context with an
error type:

```rust
#[cgp_new_provider]
impl<Context, E> ErrorRaiser<Context, E> for PanicOnError
where
    Context: HasErrorType,
    E: Debug,
{
    fn raise_error(e: E) -> Context::Error {
        panic!("{e:?}")
    }
}
```

The return type is `Context::Error`, but `panic!` diverges, so the body never produces one. The
generated [`IsProviderFor`](../../traits/wiring/is_provider_for.md) impl carries the same bounds.

## Related constructs

- [`CanRaiseError`](../../components/can_raise_error.md) — the component `PanicOnError` supplies.
- [`RaiseFrom`](raise_from.md), [`ReturnError`](return_error.md) — the raisers to use when an error
  should become a value rather than abort.
- [`delegate_components!`](../../macros/delegate_components.md) — its `open` statement dispatches
  raisers per source type, so a context can panic on some sources and raise others.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — the raising strategy as a wiring
  decision, here choosing to abort.

## Source

- [`panic_error.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/panic_error.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
