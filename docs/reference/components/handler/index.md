---
description: 'The handler family: Computer, TryComputer, Handler, Producer, and their async and by-reference variants, and how a provider promotes between them.'
sidebar_label: 'Overview'
sidebar_position: 0
---

# The handler family

The handler family models a computation as a swappable component, so a pipeline step, an I/O call,
or a type-level interpreter is wired and composed the same way as any other component. Every member
turns an `Input` into an `Output` under a phantom `Code` tag, against a
[**context**](/docs/reference/glossary#context), the type the implementation runs against. The
members differ along three axes: synchronous or async, infallible or fallible, and taking the input
by value, by reference, or not at all. This page maps the family; each member has its own page for
the full detail.

The one distinction that organizes the rest is which corner of those axes a member sits in.
[`Computer`](./computer.md) is the simplest, a synchronous transform that cannot fail.
[`TryComputer`](./try_computer.md) adds a failure path, returning a `Result` in the context's
abstract error type. [`AsyncComputer`](./async_computer.md) makes the computer async instead, and
[`Handler`](./handler.md) is both async and fallible, the most general corner.
[`Producer`](./producer.md) is the odd one out on the input axis: it takes no input and yields a
value from the context and a `Code` tag alone.

The family is built to be written from its narrowest member and promoted outward. A `Computer`
promotes into a `TryComputer` by wrapping its output in `Ok`, into an `AsyncComputer` by running
inside an async method, and into a `Handler` in two such steps. A `Producer` promotes into a
`Computer` by ignoring the input it is given. A by-reference member answers the owned-input one by
dereferencing the input, and the reverse works only for a provider that takes the borrow as its
input. Promotion never removes a failure path or an `await`, so a provider author implements the
narrowest member that fits and lets the wiring lift it where a more capable one is required. Generic
code bounded by `CanHandle` therefore accepts a provider written for any owned-input member, once
the context wires the matching promotion.

The family has nine members in all, because each of the four input-taking corners carries a
by-reference sibling, a component of its own whose method takes `&Input`:
[`Computer`](./computer.md) with [`ComputerRef`](./computer_ref.md),
[`AsyncComputer`](./async_computer.md) with [`AsyncComputerRef`](./async_computer_ref.md),
[`TryComputer`](./try_computer.md) with [`TryComputerRef`](./try_computer_ref.md), and
[`Handler`](./handler.md) with [`HandlerRef`](./handler_ref.md). [`Producer`](./producer.md) has no
such sibling, because it has no input to borrow, and no async one either. The variant pages lean on
their base member for the shared model.

Every member's wiring key is in the prelude, and so are the provider traits `Computer`,
`AsyncComputer`, `AsyncComputerRef`, `TryComputer`, `Handler`, and `Producer`. The consumer traits
(`CanCompute`, `CanTryCompute`, `CanHandle`, `CanProduce`, and their siblings), the provider traits
`ComputerRef`, `TryComputerRef`, and `HandlerRef`, and the one-step promotion providers are imported
from `cgp::extra::handler`. Because a context that delegates a member also implements its provider
trait, calling a method on a concrete context by its bare name, as `App::compute(&App, …)`, is
ambiguous (`E0034`) whenever that provider trait is in scope; method syntax, `App.compute(…)`, is
not. Every member registers under `@cgp.extra.handler` in `DefaultNamespace`, so a context that
joins that namespace binds a member at a path such as `@cgp.extra.handler.ComputerComponent`, since
a bare-key entry would conflict with the namespace's own (`E0119`).

You rarely implement a member of this family by hand. Providers come from
[`#[cgp_computer]`](../../macros/cgp_computer.md) and
[`#[cgp_producer]`](../../macros/cgp_producer.md), which turn a plain function into a provider and
wire it to a promotion bundle, so one function answers the rest of the family, and they are composed
and routed with the [handler combinators](../../providers/handler/index.md), the [dispatch
combinators](../../providers/dispatch/index.md), and the [monad
providers](../../providers/monad/index.md). A fallible member has
[`HasErrorType`](../has_error_type.md) as a [supertrait](/docs/reference/glossary#supertrait), and
an effectful one reaches the runtime through [`HasRuntime`](../has_runtime.md).

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
