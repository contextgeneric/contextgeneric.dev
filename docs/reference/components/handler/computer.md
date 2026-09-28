---
title: 'Computer — a synchronous computation'
description: 'The handler-family component for a synchronous computation that cannot fail: a provider turns an Input into an Output under a Code tag.'
sidebar_label: 'Computer'
sidebar_position: 1
---

# `Computer`

The synchronous, infallible member of the handler family, and the simplest provider to write.

## Overview

`Computer` is for computations that always succeed. Adding two numbers, formatting a value, or
projecting a field never fails, so returning a `Result` and requiring an error type would be noise.
A `Computer` provider names an `Output` type and produces it from the
[**context**](/docs/reference/glossary#context) (the type the implementation runs against), a
phantom `Code` tag naming which computation is wanted, and an `Input`, with no failure path.

It is the member of the [handler family](/docs/concepts/handlers) a provider author reaches for
first, because promotion runs one way. The [handler combinators](../../providers/handler/index.md)
lift a `Computer` into the fallible and async members by wrapping its output in `Ok` or its call in
an async method, while a fallible or async provider never loses its failure path or its `await`.
Writing the narrowest member that fits keeps a provider usable in the most positions.

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
- [`#[prefix]`](../../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.extra.handler`, so a context that joins that namespace binds its provider at `@cgp.extra.handler.ComputerComponent` rather than at the bare key.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the legacy `UseDelegate` and `UseInputDelegate` providers, which dispatch on the `Code` tag and on the `Input` type through an inner table; the `open` statement replaces both, dispatching on either parameter or on the pair.

## Usage

`Computer` and `ComputerComponent` are in the prelude. The consumer trait `CanCompute` is not, and
is imported from `cgp::extra::handler`. The consumer method takes the context, a `PhantomData<Code>`
naming which computation is wanted, and the `Input` by value, and returns the associated `Output`:

```rust
fn compute(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
```

A context gains the operation by wiring `ComputerComponent` to a provider. Three sources supply one:

- **[`#[cgp_computer]`](../../macros/cgp_computer.md)** turns a plain function such as
  `fn double(input: u64) -> u64` into a `Computer` provider, and wires that provider to a promotion
  bundle so it answers the other members too.
- **The [handler combinators](../../providers/handler/index.md)** build a computer from other
  providers: [`ReturnInput`](../../providers/handler/return_input.md) returns its input,
  [`PipeHandlers`](../../providers/handler/pipe_handlers.md) and
  [`ComposeHandlers`](../../providers/handler/compose_handlers.md) chain computers, and
  [`Promote`](../../providers/handler/promote.md) lifts a [`Producer`](./producer.md) into a
  `Computer`.
- **[`UseField<Tag>`](../../providers/use_field.md)** forwards the computation to the value stored
  in the context's `Tag` field, which must itself implement `CanCompute`.

Wiring a component to a promotion bundle such as
[`PromoteComputer<Double>`](../../providers/handler/promote_computer.md) lifts a `Computer` into
the other members. The bundle wraps the output in `Ok` for [`TryComputer`](./try_computer.md) and
runs the computation inside an async method for [`AsyncComputer`](./async_computer.md). Its
by-reference entries pass the borrow `&Input` through as the input, so they work only for a provider
written for `&'a Input` at every lifetime: a computer over an owned `u64` answers `compute`,
`try_compute`, and `compute_async`, but not `compute_ref`. A fallible function written with
`#[cgp_computer]` becomes a `Computer` whose `Output` is the `Result`, and
[`TryPromote`](../../providers/handler/try_promote.md) reads that `Result` as the failure path of a
[`TryComputer`](./try_computer.md).

A context that joins `DefaultNamespace` binds the provider at the component's prefixed path, since a
bare `ComputerComponent` entry would conflict with the namespace's own entry for that key (`E0119`):

```rust
delegate_components! {
    App {
        namespace DefaultNamespace;

        @cgp.extra.handler.ComputerComponent: Double,
    }
}
```

The component has three siblings, each differing from `Computer` on one axis and documented on its
own page: [`ComputerRef`](./computer_ref.md) borrows the input,
[`AsyncComputer`](./async_computer.md) is async, and [`AsyncComputerRef`](./async_computer_ref.md)
is both. None of the four has [`HasErrorType`](../has_error_type.md) as a
[supertrait](/docs/reference/glossary#supertrait), because none of them can fail.

## Examples

A computer provider that doubles its input, wired into a context directly and, through promotion, as
a fallible computer:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanCompute, CanTryCompute};

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ComputerComponent: Double,
        TryComputerComponent: PromoteComputer<Double>,
    }
}

check_components! {
    App {
        [ComputerComponent, TryComputerComponent]: ((), u64),
    }
}

pub fn demo() {
    assert_eq!(App.compute(PhantomData::<()>, 21), 42);
    assert_eq!(App.try_compute(PhantomData::<()>, 21), Ok(42));
}
```

`Double` implements the `Computer` provider trait for any context and any `Code`, with `u64` as both
input and output. `App` is an [environmental
context](/docs/reference/glossary#environmental-context) with no fields, and the component is
**[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**: the computation
acts on the `Input` value, while `App` decides the provider. `App` wires `ComputerComponent` to
`Double`, so `compute` returns `42`, and wires `TryComputerComponent` through the `PromoteComputer`
bundle, so `try_compute` returns `Ok(42)`. The fallible member needs an error type, which `App` sets
to `String` with `ErrorTypeProviderComponent`, imported from `cgp::core::error`. In practice
[`#[cgp_computer]`](../../macros/cgp_computer.md) writes `Double` from
`fn double(input: u64) -> u64` and wires every promotion itself.

## When to use it

**Reach for `Computer` for a synchronous transform that cannot fail.** It is the default member of
the family to implement, because promotion lifts it into the fallible and async members, so one
`Computer` provider can answer a fallible or async bound elsewhere in the wiring. Prefer
[`ComputerRef`](./computer_ref.md) when the computation only reads its input, and
[`AsyncComputer`](./async_computer.md) when it must await but still cannot fail.

Reach for [`TryComputer`](./try_computer.md) instead when the computation can fail, for
[`Handler`](./handler.md) when it is both async and fallible, and for [`Producer`](./producer.md)
when it takes no input. A computation that reads its context's fields, names an abstract type, or
calls another trait of the context is written as a provider with
[`#[cgp_impl]`](../../macros/cgp_impl.md), since a `#[cgp_computer]` function cannot reach its
context.

## Common Mistakes

**Calling `compute` on a concrete context by its bare name is ambiguous.** A context that delegates
`ComputerComponent` also implements the `Computer` provider trait, which the prelude brings into
scope, so `App::compute(&App, PhantomData::<()>, 21)` has two candidates:

```text
error[E0034]: multiple applicable items in scope
  --> src/main.rs:24:18
   |
24 |     let _ = App::compute(&App, PhantomData::<()>, 21);
   |                  ^^^^^^^ multiple `compute` found
   |
   = note: candidate #1 is defined in an impl of the trait `CanCompute` for the type `__Context__`
   = note: candidate #2 is defined in an impl of the trait `cgp::prelude::Computer` for the type `__Provider__`
```

Use method syntax, as in `App.compute(PhantomData::<()>, 21)`, since only the consumer trait takes
`self`, or name the consumer trait, as in
`<App as CanCompute<(), u64>>::compute(&App, PhantomData, 21)`. The same holds for every member of
the family whose provider trait is in scope.

## Related constructs

- [`ComputerRef`](./computer_ref.md) — the by-reference variant that borrows its input.
- [`AsyncComputer`](./async_computer.md) — the async variant that awaits.
- [`AsyncComputerRef`](./async_computer_ref.md) — the async, by-reference variant.
- [`TryComputer`](./try_computer.md) — the fallible synchronous counterpart.
- [`Handler`](./handler.md) — the async and fallible member.
- [`Producer`](./producer.md) — the input-free member of the family.
- [`#[cgp_computer]`](../../macros/cgp_computer.md) — builds a `Computer` provider from a plain function.
- [Handler combinators](../../providers/handler/index.md) — compose computers and promote them into
  the other members.
- [`UseField`](../../providers/use_field.md) — the built-in computer provider that forwards to a
  context field.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.
- [Dispatching](/docs/concepts/dispatching) — routing a computation on its `Code` or `Input`.

## Source

- The synchronous computers `Computer`/`ComputerRef`, and the `UseField` computer:
  [`computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/computer.rs)
- The async computers `AsyncComputer`/`AsyncComputerRef`:
  [`async_computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/async_computer.rs)
- The promotion bundles:
  [`promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)
- Re-exported through `cgp::extra::handler`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
