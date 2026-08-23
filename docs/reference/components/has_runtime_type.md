---
sidebar_label: 'HasRuntimeType'
sidebar_position: 7
---

# `HasRuntimeType`

Declare the abstract runtime *type* a context runs against, chosen per context through wiring.

## Overview

`HasRuntimeType` lets context-generic code name the runtime type a context uses without committing to a
concrete one. A runtime is whatever object supplies the capabilities an application needs at execution
time, and different deployments want different runtimes: Tokio in production, a mock in tests, an
executor in a benchmark. `HasRuntimeType` gives the **context**, the type a capability runs against that
supplies the values an implementation needs as its own fields, an abstract associated `Runtime` type,
resolved to a concrete one at wiring time.

The runtime abstraction is split into a type component and a getter component because the two questions
are independent. `HasRuntimeType` answers *what the runtime type is*, while its companion
[`HasRuntime`](./has_runtime.md) answers *how to obtain the runtime value* from a borrow of the context.
Some code is generic only over the runtime type: it names types the runtime exposes but never touches a
runtime value, and it needs `HasRuntimeType` alone. Keeping the type separate means such a bound asks
for exactly the capability it uses.

## Definition

`HasRuntimeType` is defined as:

```rust
#[cgp_type]
pub trait HasRuntimeType {
    type Runtime;
}

pub type RuntimeOf<Context> = <Context as HasRuntimeType>::Runtime;
```

Its attributes:

- [`#[cgp_type]`](../macros/cgp_type.md) — makes this an abstract-type component rather than a plain trait: it generates the provider trait, the component marker, and a [`UseType`](../providers/use_type.md) impl, so a context binds the concrete type by wiring.

## Usage

`HasRuntimeType` is imported from `cgp::extra::runtime`. It is an abstract-type component with a single
associated type, `Runtime`, carrying no bound, so any concrete type may be plugged in. A context supplies
the type either by implementing `HasRuntimeType` directly or, more commonly, by wiring
`RuntimeTypeProviderComponent` to [`UseType<R>`](../providers/use_type.md):

```rust
delegate_components! {
    App {
        RuntimeTypeProviderComponent: UseType<TokioRuntime>,
    }
}
```

This makes the context resolve `HasRuntimeType` with `Runtime = TokioRuntime`, and from then on the
alias `RuntimeOf<Context>` is `TokioRuntime`. Generic code names the runtime type with the
[`#[use_type]`](../attributes/use_type.md) attribute, which imports it as the bare name `Runtime`, or
through the `RuntimeOf<Context>` alias.

## Examples

A context fixes its runtime type, and generic code names it without committing to a concrete one:

```rust
use cgp::prelude::*;
use cgp::extra::runtime::{HasRuntimeType, RuntimeOf};

pub struct TokioRuntime { /* handle, clock, etc. */ }

pub struct App;

delegate_components! {
    App {
        RuntimeTypeProviderComponent: UseType<TokioRuntime>,
    }
}

fn describe<Context>() -> &'static str
where
    Context: HasRuntimeType,
    RuntimeOf<Context>: Default,
{
    "the context has a default-constructible runtime type"
}
```

`App` resolves `HasRuntimeType` with `Runtime = TokioRuntime` through `UseType`, so `RuntimeOf<App>` is
`TokioRuntime`. `describe` names only the runtime type and never a runtime value, which is exactly the
case `HasRuntimeType` serves alone. `App` is an **environmental context**, and the capability targets it.

## When to reach for it, and when not

**Reach for `HasRuntimeType` when code names the runtime type but never touches a runtime value.** That
is the narrower of the two runtime capabilities, and asking for it alone keeps a bound honest about what
it uses. It is also the type half a context must wire before it can satisfy
[`HasRuntime`](./has_runtime.md), because the getter borrows a value of this type.

Reach for [`HasRuntime`](./has_runtime.md) instead when a provider actually performs an effect and needs
the runtime *value*, since `HasRuntime` supertraits this one and gives access to both. For an abstract
type that is not the runtime, define your own component with [`#[cgp_type]`](../macros/cgp_type.md)
rather than reusing this one.

## Related constructs

- [`HasRuntime`](./has_runtime.md) — the getter companion that borrows a value of this type.
- [`#[cgp_type]`](../macros/cgp_type.md) — the macro `HasRuntimeType` is defined with.
- [`HasType` / `TypeProvider`](./has_type.md) — the built-in abstract-type substrate this rests on.
- [`UseType`](../providers/use_type.md) — the provider that fixes the runtime type in wiring.
- [`CanRun` / `Runner`](./runner.md) — the task runner whose providers reach the runtime.

The ideas behind it:

- [Abstract types](/docs/concepts/abstract-types) — the associated types a context chooses for itself.

## Source

- `HasRuntimeType`, the `RuntimeTypeProvider` provider trait, and the `RuntimeOf` alias:
  [`has_runtime_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-runtime/src/traits/has_runtime_type.rs)
- Re-exported through `cgp::extra::runtime`:
  [`cgp-runtime/src/lib.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-runtime/src/lib.rs)
- The `#[cgp_type]` expansion it relies on:
  [`cgp-macro-core/src/types/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
