---
title: 'DiscardDetail — wrap by dropping the detail'
description: 'The error wrapper that ignores the attached detail and returns the error unchanged, for an error type that cannot carry extra context.'
sidebar_label: 'DiscardDetail'
sidebar_position: 6
---

# `DiscardDetail`

Wrap an error by throwing the detail away and returning the error unchanged.

## Overview

`DiscardDetail` is the `ErrorWrapper` provider that ignores whatever detail is attached and returns the
error as it was. It satisfies the `CanWrapError` trait of a
[**context**](/docs/reference/glossary#context), the type the implementation runs against, without
enriching the error. This is useful when a context's error type cannot carry extra context, or when
the wrapping detail is deliberately not kept. It is the wrapping counterpart of a no-op: the error
propagates unchanged. Like every CGP provider, `DiscardDetail` carries no runtime value.

## Usage

Import the provider from `cgp::extra::error` and the wiring key `ErrorWrapperComponent` from
`cgp::core::error`; neither is in the prelude. It takes no type parameter and accepts any detail type:

```rust
use cgp::core::error::ErrorWrapperComponent;
use cgp::extra::error::DiscardDetail;

delegate_components! {
    App {
        ErrorWrapperComponent: DiscardDetail,
    }
}
```

Wired this way, any call to `Context::wrap_error(error, detail)` on `App` returns `error` and drops
`detail`, for every detail type.

## Examples

A provider raises an error and wraps it with a detail string, and the context's wrapper drops the
detail:

```rust
use cgp::prelude::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent, ErrorWrapperComponent};
use cgp::extra::error::{DiscardDetail, RaiseFrom};

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self) -> Result<(), Error>;
}

#[cgp_impl(new LoadConfig)]
#[uses(CanRaiseError<String>, CanWrapError<String>)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self) -> Result<(), Error> {
        let error = Self::raise_error("disk offline".to_owned());
        Err(Self::wrap_error(error, "while loading config".to_owned()))
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ErrorRaiserComponent: RaiseFrom,
        ErrorWrapperComponent: DiscardDetail,
        LoaderComponent: LoadConfig,
    }
}

check_components! {
    App {
        LoaderComponent,
    }
}

pub fn demo() {
    assert_eq!(App.load(), Err("disk offline".to_owned()));
}
```

`LoadConfig` attaches `"while loading config"`, but `App`, an
[environmental context](/docs/reference/glossary#environmental-context), wires `DiscardDetail`, so
the error arrives as the bare `"disk offline"`. Wiring a wrapper that keeps the detail instead changes
the result without touching `LoadConfig`.

## When to use it

**Reach for `DiscardDetail` when a context's error type cannot hold extra detail, or when a call site
attaches detail that this context has no use for.** It keeps the `CanWrapError` trait satisfiable
without storing anything.

Reach for [`DebugError`](debug_error.md) or [`DisplayError`](display_error.md) when the detail should
be kept as a formatted string, or for a backend provider when the concrete error type can carry
structured context.

## Under the hood

`DiscardDetail` implements `ErrorWrapper<Context, Detail>` for any context and any detail type:

```rust
#[cgp_new_provider]
impl<Context, Detail> ErrorWrapper<Context, Detail> for DiscardDetail
where
    Context: HasErrorType,
{
    fn wrap_error(error: Context::Error, _detail: Detail) -> Context::Error {
        error
    }
}
```

`Detail` is unconstrained, so any type is accepted, and the body returns the error untouched. The
generated [`IsProviderFor`](../../traits/wiring/is_provider_for.md) impl carries the same
`HasErrorType` bound.

## Related constructs

- [`CanWrapError`](../../components/can_wrap_error.md) — the component `DiscardDetail` supplies,
  through the `ErrorWrapper` provider trait.
- [`DebugError`](debug_error.md), [`DisplayError`](display_error.md) — wrap by keeping the detail as a
  formatted string instead of discarding it.
- [`delegate_components!`](../../macros/delegate_components.md) — its `open` statement dispatches
  wrappers per detail type.

The ideas behind it:

- [Modular error handling](/docs/concepts/modular-error-handling) — attaching detail as a wiring
  decision independent of the error type and its construction.

## Source

- [`discard_detail.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-error-extra/src/impls/discard_detail.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
