---
sidebar_label: 'Computer'
sidebar_position: 1
---

# `Computer`

The pure synchronous transform of the handler family: infallible, and the simplest provider to write.

## Overview

`Computer` is for computations that always succeed and need no error type. A computation that adds two
numbers, formats a value, or projects a field never fails and never needs the **context's** abstract
error, so forcing it to return a `Result` would be noise. The context is the type a capability runs
against, which supplies the values an implementation needs as its own fields. `Computer` captures the
succeeds-always case: a provider names an `Output` type and produces it from the context, a phantom
`Code` tag, and an `Input`, with no failure path.

It is the simplest member of the [handler family](/docs/concepts/handlers) and the one a provider author
reaches for first, because a pure computation can always be *promoted* into the fallible, async, and
general variants when the wiring needs them, while a fallible provider cannot be demoted back. Writing
the narrowest variant that fits keeps a provider usable in every position.

## Definition

`CanCompute` is defined as:

```rust
#[cgp_component(Computer)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
pub trait CanCompute<Code, Input> {
    type Output;

    fn compute(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
}
```

Its attributes:

- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `Computer` that implementations target and the wiring key `ComputerComponent`, while `CanCompute` stays the consumer trait callers use.
- [`#[prefix]`](../../macros/cgp_namespace.md) — registers the generated names into the `@cgp.extra.handler` path of `DefaultNamespace`, so a context that joins the namespace inherits the wiring by default.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the dispatching providers a context routes through: `UseDelegate<Code>` dispatches on the `Code` tag, and `UseInputDelegate<Input>` on the `Input` type; the `open` statement is the modern sugar for the `Code` dispatch.

## Usage

`Computer` and its consumer trait `CanCompute` are imported from `cgp::extra::handler`. The consumer
method takes the context, a `PhantomData<Code>` naming which computation is wanted, and the `Input` by
value, returning the associated `Output`:

```rust
fn compute(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
```

A context gains the capability by wiring `ComputerComponent` to a provider. In everyday code the
provider comes from [`#[cgp_computer]`](../../macros/cgp_computer.md), which turns a plain function such as
`fn double(input: u64) -> u64` into a `Computer` provider and wires the promotion table so the same
function also answers `CanTryCompute`, `CanComputeAsync`, `CanHandle`, and the by-reference forms.

The component dispatches on both the `Code` tag and the `Input` type. It has three siblings, each
differing from `Computer` by one axis and documented on its own page:
[`ComputerRef`](./computer_ref.md) borrows the input, [`AsyncComputer`](./async_computer.md) is the async
counterpart, and [`AsyncComputerRef`](./async_computer_ref.md) combines the two. None of the four
supertrait [`HasErrorType`](../has_error_type.md), because none of them can fail.

## Examples

A computer provider that doubles its input, wired into a context and invoked through the consumer trait:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::{CanCompute, Computer, ComputerComponent};

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

#[derive(HasField)]
pub struct App;

delegate_components! {
    App {
        ComputerComponent: Double,
    }
}

// `App` now implements `CanCompute<(), u64, Output = u64>`:
fn run(app: &App) -> u64 {
    app.compute(PhantomData::<()>, 21) // returns 42
}
```

`Double` implements the `Computer` provider trait for any context and any `Code`, fixing `Input` and
`Output` to `u64`. `App` delegates `ComputerComponent` to it, and the call passes `PhantomData::<()>` as
the `Code` tag. The example is **parameter-targeted**: the computation acts on the `Input` value, while
the context decides the provider. In practice a provider like this is written with
[`#[cgp_computer]`](../../macros/cgp_computer.md) rather than by hand.

## When to use it

**Reach for `Computer` for a synchronous transform that cannot fail.** It is the default member of the
family to implement, because promotion lifts it into every other variant, so one `Computer` provider can
answer a fallible or async bound elsewhere in the wiring. Prefer [`ComputerRef`](./computer_ref.md) when
the computation only reads its input, and [`AsyncComputer`](./async_computer.md) when it must await but
still cannot fail.

Reach for [`TryComputer`](./try_computer.md) instead when the computation can fail, for
[`Handler`](./handler.md) when it is both async and fallible, and for [`Producer`](./producer.md) when it
takes no input. Do not force a fallible computation into a `Computer` by making its `Output` a `Result`
unless you specifically want the result passed through unexamined.

## Related constructs

- [`ComputerRef`](./computer_ref.md) — the by-reference variant that borrows its input.
- [`AsyncComputer`](./async_computer.md) — the async variant that awaits.
- [`AsyncComputerRef`](./async_computer_ref.md) — the async, by-reference variant.
- [`TryComputer`](./try_computer.md) — the fallible synchronous counterpart.
- [`Handler`](./handler.md) — the general async-and-fallible generalization.
- [`Producer`](./producer.md) — the input-free member of the family.
- [`#[cgp_computer]`](../../macros/cgp_computer.md) — builds a `Computer` provider from a plain function.
- [Handler combinators](../../providers/handler/index.md) — compose computers and promote them into the
  other variants.
- [`UseField`](../../providers/use_field.md) — the built-in computer provider that forwards to a context
  field.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Dispatching](/docs/concepts/dispatching) — routing a computation on its `Code` or `Input`.

## Source

- The synchronous computers `Computer`/`ComputerRef`:
  [`computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/computer.rs)
- The async computers `AsyncComputer`/`AsyncComputerRef`:
  [`async_computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/async_computer.rs)
- Re-exported through `cgp::extra::handler`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
