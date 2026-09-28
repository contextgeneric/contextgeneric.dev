---
title: 'Producer — a value from no input'
description: 'The input-free handler-family component: a synchronous, infallible provider that produces an Output from the context and a Code tag alone.'
sidebar_label: 'Producer'
sidebar_position: 4
---

# `Producer`

The input-free member of the handler family: a synchronous, infallible source of a value.

## Overview

`Producer` is for computations that take no input and yield a value. A default configuration, a
constant, or a value read entirely from the [**context**](/docs/reference/glossary#context) (the
type the implementation runs against) has no `Input` to transform: it needs only the context and a
phantom `Code` tag naming which value to produce. The rest of the [handler
family](/docs/concepts/handlers) threads an `Input` through every method, and `Producer` is the case
where that input is absent. It is synchronous and infallible, and returns its `Output` directly.

A producer still feeds the rest of the family. Promotion turns it into a
[`Computer`](./computer.md) that ignores whatever input it is given, and from there into the other
members, which is how a constant or a context-derived value enters a pipeline of input-taking
handlers.

## Definition

`CanProduce` is defined as:

```rust
#[cgp_component(Producer)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
pub trait CanProduce<Code> {
    type Output;

    fn produce(&self, _code: PhantomData<Code>) -> Self::Output;
}
```

Its attributes:

- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `Producer` that implementations target and the wiring key `ProducerComponent`, while `CanProduce` stays the consumer trait callers use.
- [`#[prefix]`](../../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.extra.handler`, so a context that joins that namespace binds its provider at `@cgp.extra.handler.ProducerComponent` rather than at the bare key.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the legacy `UseDelegate` provider, which dispatches on the `Code` type through an inner table; the `open` statement replaces it. There is no `UseInputDelegate`, since there is no input.

## Usage

`Producer` and `ProducerComponent` are in the prelude. The consumer trait `CanProduce` is not, and
is imported from `cgp::extra::handler`. The consumer method takes only the context and a `Code` tag,
with no `Input`:

```rust
fn produce(&self, _code: PhantomData<Code>) -> Self::Output;
```

A context gains the operation by wiring `ProducerComponent` to a provider. The component dispatches
on the `Code` tag alone, and it has no by-reference or async sibling, since there is no input to
borrow.

In everyday use the provider comes from [`#[cgp_producer]`](../../macros/cgp_producer.md), which
turns a zero-argument function into a `Producer` provider and wires that provider to the
[`PromoteProducer<Self>`](../../providers/handler/promote_producer.md) bundle. So a context can wire
any member of the family to it directly, and each member returns the produced value whatever input
it is given:

```rust
#[cgp_producer]
fn default_port() -> u16 {
    8080
}

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        [ComputerComponent, HandlerComponent]: DefaultPort,
    }
}
```

The fallible members need the context's error type to form their `Result`, which is why `App` wires
`ErrorTypeProviderComponent`, but they always return `Ok`. A `#[cgp_producer]` function cannot reach
its context, since it has no parameters, so a producer that reads a field or calls another trait of
the context is a provider written with [`#[cgp_impl]`](../../macros/cgp_impl.md).

## Examples

A producer wired into a context and invoked directly:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::CanProduce;

#[cgp_new_provider]
impl<Context, Code> Producer<Context, Code> for MagicNumber {
    type Output = u64;

    fn produce(_context: &Context, _code: PhantomData<Code>) -> u64 {
        42
    }
}

pub struct App;

delegate_components! {
    App {
        ProducerComponent: MagicNumber,
    }
}

check_components! {
    App {
        ProducerComponent: (),
    }
}

pub fn demo() {
    assert_eq!(App.produce(PhantomData::<()>), 42);
}
```

`MagicNumber` produces `42` for any context and any `Code`, and `App` delegates `ProducerComponent`
to it. `App` is an [environmental context](/docs/reference/glossary#environmental-context), and the
component is **[self-targeted](/docs/reference/glossary#self-targeted-component)**: there is no
input to act on, so the operation describes the context itself, and `Code` is a
[selector](/docs/reference/glossary#selector) naming which value is wanted.

## When to use it

**Reach for `Producer` for a value that a computation needs but does not compute from an input**,
such as a constant, a default, or a value read from the context. It is the family's source, and
promotion lets one producer feed the input-taking members, so a producer can seed a pipeline such as
`PipeHandlers<Product![DefaultPort, …]>`.

Reach for [`Computer`](./computer.md) instead when the value is computed *from* an input, and for
[`TryComputer`](./try_computer.md) or [`Handler`](./handler.md) when producing it can fail or must
await. A `#[cgp_producer]` function returning a `Result` is not a fallible producer: the fallible
members wrap the whole `Result` in `Ok`, so an `Err` short-circuits nothing downstream.

## Common Mistakes

**`PromoteProducer<P>` answers the fallible and async members only when `P` is wired to it.** Its
`ComputerComponent` entry works for any producer, but its other entries reach `P` through `P`'s own
`Computer` wiring. So a context that wires `TryComputerComponent` to `PromoteProducer<MagicNumber>`,
for the hand-written `MagicNumber` above, fails at the check:

```text
error[E0277]: the trait bound `MagicNumber: DelegateComponent<cgp::prelude::ComputerComponent>` is not satisfied
...
   = note: required for `MagicNumber` to implement `Computer<App, (), ()>`
   = note: required for `Promote<MagicNumber>` to implement `IsProviderFor<cgp::prelude::TryComputerComponent, App, ((), ())>`
```

Write the producer with [`#[cgp_producer]`](../../macros/cgp_producer.md), which wires it to
`PromoteProducer<Self>`, or wire the hand-written producer to that bundle yourself, as the macro
does.

## Related constructs

- [`Computer`](./computer.md) — the synchronous input-taking member; a producer promotes into it.
- [`TryComputer`](./try_computer.md) — the fallible member of the family.
- [`Handler`](./handler.md) — the general async and fallible member.
- [`#[cgp_producer]`](../../macros/cgp_producer.md) — builds a `Producer` provider from a zero-argument
  function.
- [`PromoteProducer`](../../providers/handler/promote_producer.md) — the bundle that lifts a
  producer into the rest of the family.
- [`Promote`](../../providers/handler/promote.md) — the one-step lift from a producer to a computer.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.

## Source

- `Producer`:
  [`produce.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/produce.rs)
- `Promote`:
  [`promote.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote.rs)
- The `PromoteProducer` bundle:
  [`promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)
- Re-exported through `cgp::extra::handler`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
