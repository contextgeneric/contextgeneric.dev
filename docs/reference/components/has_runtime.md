---
sidebar_label: 'HasRuntime'
sidebar_position: 6
---

# `HasRuntime`

Hand out a borrow of the context's runtime value, so effectful code reaches it generically.

## Overview

`HasRuntime` lets context-generic code obtain the runtime it runs against without naming a concrete one.
A runtime here is whatever object provides the capabilities an application needs at execution time,
spawning tasks, sleeping, opening sockets, reading the clock, and different deployments want different
runtimes: Tokio in production, a mock in tests, a single-threaded executor in a benchmark. Rather than
thread a concrete runtime type through every signature, the **context**, the type a capability runs
against that supplies the values an implementation needs as its own fields, stores one runtime value,
and `HasRuntime` is the getter that borrows it.

`HasRuntime` answers *how to obtain the runtime value*; its companion
[`HasRuntimeType`](./has_runtime_type.md) answers *what the runtime type is*. The two are split because
the questions are independent: some code names only the runtime type and never touches a value, and
needs `HasRuntimeType` alone, while code that performs effects needs `HasRuntime`, which supertraits
`HasRuntimeType` so the value's type is always in scope. `HasRuntime` is the seam where context-generic
logic meets the concrete async machinery, which is why it underpins the [`CanRun`](./runner.md) family
of task runners.

## Definition

`HasRuntime` is defined as:

```rust
#[cgp_getter]
#[use_type(HasRuntimeType.Runtime)]
pub trait HasRuntime {
    fn runtime(&self) -> &Runtime;
}
```

Its attributes:

- [`#[cgp_getter]`](../macros/cgp_getter.md) — makes this a getter component: it derives the provider name from the trait name and generates a [`UseField`](../providers/use_field.md) impl, so the field it reads is chosen by wiring.
- [`#[use_type]`](../attributes/use_type.md) — adds `HasRuntimeType` as a supertrait and rewrites the bare `Runtime` to `<Self as HasRuntimeType>::Runtime`.

## Usage

`HasRuntime` is imported from `cgp::extra::runtime`. It is a getter component: its method borrows the
runtime value out of a borrow of the context, returning `&Runtime`, where `Runtime` is the abstract type
supplied by [`HasRuntimeType`](./has_runtime_type.md):

```rust
fn runtime(&self) -> &Runtime;
```

A context supplies the value either by implementing `HasRuntime` directly or, more commonly, by wiring
`RuntimeGetterComponent` to a [`UseField`](../providers/use_field.md) provider that names the field
holding the runtime. Because `HasRuntime` supertraits `HasRuntimeType`, a context cannot satisfy it
without also having declared its runtime *type*, which keeps the returned `&Runtime` well-defined. A
context that stores its runtime in a `runtime` field wires:

```rust
delegate_components! {
    App {
        RuntimeTypeProviderComponent: UseType<TokioRuntime>,
        RuntimeGetterComponent: UseField<Symbol!("runtime")>,
    }
}
```

One entry fixes the abstract runtime type, and the other says where the value lives. Context-generic
providers then write `where Self: HasRuntime` and call `self.runtime()` to obtain a `&RuntimeOf<Self>`,
never naming the concrete runtime.

## Examples

A context declares its runtime type and the field holding the value, and a provider reaches the runtime
generically:

```rust
use cgp::prelude::*;
use cgp::extra::runtime::{HasRuntime, HasRuntimeType, RuntimeOf};

pub struct TokioRuntime { /* handle, clock, etc. */ }

#[derive(HasField)]
pub struct App {
    pub runtime: TokioRuntime,
}

delegate_components! {
    App {
        RuntimeTypeProviderComponent: UseType<TokioRuntime>,
        RuntimeGetterComponent: UseField<Symbol!("runtime")>,
    }
}

fn runtime_of<Context>(context: &Context) -> &RuntimeOf<Context>
where
    Context: HasRuntime,
{
    context.runtime()
}
```

`App` resolves `HasRuntimeType` with `Runtime = TokioRuntime` through `UseType`, and resolves
`HasRuntime` by reading its `runtime` field through `UseField`. `runtime_of` names neither
`TokioRuntime` nor any field, so swapping `UseType<TokioRuntime>` for `UseType<MockRuntime>` in a test
context retargets it at the mock with no change to its body. `App` is an **environmental context**, and
the capability targets it.

## When to reach for it, and when not

**Reach for `HasRuntime` in any provider that performs an effect against the runtime**, such as spawning
a task, awaiting a timer, or opening a connection, and reach it as a `#[uses(HasRuntime)]` dependency so
the bound stays off the consumer trait. It is the standard way effectful CGP code stays generic over the
runtime, and it makes the same task-running code reusable across a Tokio context, a mock, and a
test executor with only a wiring change.

Reach for [`HasRuntimeType`](./has_runtime_type.md) alone when code names the runtime type but never
touches a value, which asks for exactly the capability it uses. Do not reach for `HasRuntime` when a
value the provider needs is not really the runtime but an ordinary context field, where an
[`#[implicit]`](../attributes/implicit.md) argument is simpler.

## Related constructs

- [`HasRuntimeType`](./has_runtime_type.md) — the companion that declares the runtime *type* this
  borrows.
- [`CanRun` / `Runner`](./runner.md) — the primary consumer, reaching the runtime through `HasRuntime` to
  execute work.
- [`CanSendRun` / `SendRunner`](./send_runner.md) — the `Send`-future variant of the runner.
- [`#[cgp_getter]`](../macros/cgp_getter.md) — the macro `HasRuntime` is defined with.
- [`UseField`](../providers/use_field.md) — the provider that reads the runtime value from a named field.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the effectful computation family that reaches the runtime through
  this getter.
- [Abstract types](/docs/concepts/abstract-types) — the abstract runtime type a context chooses for
  itself.

## Source

- `HasRuntime` and its `RuntimeGetter` provider trait:
  [`has_runtime.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-runtime/src/traits/has_runtime.rs)
- Re-exported through `cgp::extra::runtime`:
  [`cgp-runtime/src/lib.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-runtime/src/lib.rs)
- The `#[cgp_getter]` expansion it relies on:
  [`cgp-macro-core/src/types/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
