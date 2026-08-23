---
sidebar_label: 'TryComputer'
sidebar_position: 2
---

# `TryComputer`

The fallible synchronous member of the handler family: a computer that can return an error.

## Overview

`TryComputer` is for synchronous computations that can fail. A computation that parses a string, looks
up a key, or checks an invariant may not be able to produce its output, and it needs a way to report the
failure. Where [`Computer`](./computer.md) returns its `Output` directly, `TryComputer` returns
`Result<Output, Error>`, where the error is the **context's** shared abstract error type. The context is
the type a capability runs against, which supplies the values an implementation needs as its own fields.
This places `TryComputer` one step up from `Computer` on the fallibility axis of the
[handler family](/docs/concepts/handlers), still synchronous but now able to fail, and one step below
[`Handler`](./handler.md), which adds asynchrony on top.

Returning the *context's* abstract error rather than a concrete one keeps a `TryComputer`
provider generic. A provider does not commit to `anyhow::Error` or `std::io::Error`; it returns
`Self::Error`, and the concrete type is decided at wiring time. This is why the component supertraits
[`HasErrorType`](../has_error_type.md): the supertrait supplies the `Self::Error` it names in its
`Result`, and it ties every fallible component in a context to the same error so their results compose.

## Definition

`CanTryCompute` is defined as:

```rust
#[cgp_component(TryComputer)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
#[use_type(HasErrorType.Error)]
pub trait CanTryCompute<Code, Input> {
    type Output;

    fn try_compute(
        &self,
        _code: PhantomData<Code>,
        input: Input,
    ) -> Result<Self::Output, Error>;
}
```

Its attributes:

- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `TryComputer` that implementations target and the wiring key `TryComputerComponent`, while `CanTryCompute` stays the consumer trait callers use.
- [`#[prefix]`](../../macros/cgp_namespace.md) — registers the generated names into the `@cgp.extra.handler` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the dispatching providers a context routes through: `UseDelegate<Code>` dispatches on the `Code` tag, and `UseInputDelegate<Input>` on the `Input` type; the `open` statement is the modern sugar for the `Code` dispatch.
- [`#[use_type]`](../../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`TryComputer` and its consumer trait `CanTryCompute` are imported from `cgp::extra::handler`. The
consumer method mirrors `compute` but returns a `Result`:

```rust
fn try_compute(&self, _code: PhantomData<Code>, input: Input) -> Result<Self::Output, Error>;
```

A context gains the capability by wiring `TryComputerComponent` to a provider. In everyday use the
provider comes from [`#[cgp_computer]`](../../macros/cgp_computer.md), which turns a function returning
`Result<u64, String>` into a `TryComputer` provider and wires the promotion table so the same function
also answers `CanCompute`, `CanHandle`, and the by-reference forms.

The component dispatches on both the `Code` tag and the `Input` type. Its by-reference sibling
[`TryComputerRef`](./try_computer_ref.md), documented on its own page, differs by one axis: its
`try_compute_ref` method borrows the input as `&Input`. Both are synchronous; the async-and-fallible
counterpart is [`Handler`](./handler.md).

## Examples

A `TryComputer` provider that parses a string into a number, raising the context's error on failure:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::{CanTryCompute, TryComputer, TryComputerComponent};

#[cgp_impl(new ParseU64)]
#[uses(CanRaiseError<core::num::ParseIntError>)]
#[use_type(HasErrorType.Error)]
impl<Code> TryComputer<Code, String> {
    type Output = u64;

    fn try_compute(
        &self,
        _code: PhantomData<Code>,
        input: String,
    ) -> Result<Self::Output, Error> {
        input.parse().map_err(|e| Self::raise_error(e))
    }
}

delegate_components! {
    App {
        TryComputerComponent: ParseU64,
    }
}
```

`ParseU64` returns its `u64` output or the context's abstract error, converting the concrete
`ParseIntError` into that error with [`CanRaiseError`](../can_raise_error.md). Because the consumer trait
supertraits `HasErrorType`, `App` must also wire an error type and an error raiser before it can call
`try_compute`. The example is **parameter-targeted**: the computation acts on the `Input`, while the
context decides the provider and the error type.

## When to use it

**Reach for `TryComputer` for a synchronous computation that can fail.** It is the fallible middle of
the family, and it sits in the promotion lattice, so it both receives providers promoted from a
[`Computer`](./computer.md) and is itself promoted up to a [`Handler`](./handler.md). Prefer
`TryComputerRef` when the computation only reads its input.

Reach for [`Computer`](./computer.md) instead when the computation cannot fail, since a pure computer
promotes into a `TryComputer` for free and stays usable in more positions, and for
[`Handler`](./handler.md) when the computation must also await. A provider author implements whichever
single variant fits and lets the wiring bridge the rest.

## Related constructs

- [`TryComputerRef`](./try_computer_ref.md) — the by-reference variant that borrows its input.
- [`Computer`](./computer.md) — the infallible counterpart; promotes into a `TryComputer`.
- [`Handler`](./handler.md) — the async-and-fallible generalization.
- [`Producer`](./producer.md) — the input-free member of the family.
- [`HasErrorType`](../has_error_type.md) — the supertrait supplying the `Self::Error` this returns.
- [`CanRaiseError`](../can_raise_error.md) — converts a concrete source error into that abstract error.
- [`#[cgp_computer]`](../../macros/cgp_computer.md) — builds a `TryComputer` provider from a fallible
  function.
- [Handler combinators](../../providers/handler/index.md) — promote a `Computer` into this and this into a
  `Handler`.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Modular error handling](/docs/concepts/modular-error-handling) — the abstract error a fallible
  computer returns.

## Source

- `TryComputer` and `TryComputerRef`:
  [`try_compute.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/try_compute.rs)
- The `ReturnInput` provider and the promotion combinators:
  [`cgp-handler/src/providers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-handler/src/)
- Re-exported through `cgp::extra::handler`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
