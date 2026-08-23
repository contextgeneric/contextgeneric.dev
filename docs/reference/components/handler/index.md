---
sidebar_label: 'Overview'
sidebar_position: 0
---

# The handler family

The handler family models a computation as a swappable component, so a pipeline step, an I/O call, or a
type-level interpreter is wired and composed the same way as any other component. Every member
transforms an `Input` into an `Output` under a phantom `Code` tag, against a **context**, the type a
capability runs against that supplies the values an implementation needs as its own fields. The members
differ along three axes: synchronous or async, infallible or fallible, and taking an input or not. This
page maps the family; each member has its own page for the full detail.

The one distinction that organizes the rest is which corner of those axes a member sits in.
[`Computer`](./computer.md) is the simplest, a pure synchronous transform that cannot fail.
[`TryComputer`](./try_computer.md) adds a failure path, returning a `Result` against the context's
abstract error type. [`Handler`](./handler.md) adds asynchrony on top of that, so it is both async and
fallible, and it is the general corner every other member is a special case of.
[`Producer`](./producer.md) is the odd one out on the input axis: it takes no input and simply yields a
value from the context and a `Code` tag.

The family is built to be written from its weakest member and promoted upward. A pure `Computer`
promotes into a `TryComputer` by wrapping its output in `Ok`, into an `AsyncComputer` by wrapping it in a
future that never awaits, and all the way up to a `Handler`; a `Producer` promotes into any of them by
ignoring the input the target supplies. Because promotion only ever widens, a provider author implements
the narrowest variant that fits and lets the wiring lift it wherever a more capable one is required.
This is why generic pipeline code bounds against [`Handler`](./handler.md): it is the one bound every
member can meet.

The family has nine members in all, because three of the base members carry by-reference and, for the
computers, async siblings, each a component of its own that differs from its base by a single axis. The
computers cover both axes: [`Computer`](./computer.md) with [`ComputerRef`](./computer_ref.md),
[`AsyncComputer`](./async_computer.md), and [`AsyncComputerRef`](./async_computer_ref.md);
[`TryComputer`](./try_computer.md) with [`TryComputerRef`](./try_computer_ref.md); and
[`Handler`](./handler.md) with [`HandlerRef`](./handler_ref.md). [`Producer`](./producer.md) has no such
sibling, because it has no input to borrow. Each has its own page; the variant pages lean on their base
member for the shared model.

You rarely implement a member of this family by hand. Providers come from
[`#[cgp_computer]`](../../macros/cgp_computer.md) and [`#[cgp_producer]`](../../macros/cgp_producer.md),
which turn a plain function into a provider and wire the promotion table so one function answers the
whole family, and they are composed and routed with the
[handler combinators](../../providers/handler/index.md), the
[dispatch combinators](../../providers/dispatch/index.md), and the
[monad providers](../../providers/monad/index.md). A fallible member supertraits
[`HasErrorType`](../has_error_type.md), and an effectful one reaches the runtime through
[`HasRuntime`](../has_runtime.md).

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
