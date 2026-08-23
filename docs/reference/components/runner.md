---
sidebar_label: 'CanRun'
sidebar_position: 8
---

# `CanRun`

Run a task named at the type level to a `Result<(), Error>`, with the behavior chosen through wiring.

## Overview

`CanRun<Code>` gives a context a uniform way to *execute a unit of work* selected at the type level. The
unit of work is identified by a `Code` type parameter, a phantom tag rather than a value, so a single
**context**, the type a capability runs against that supplies the values an implementation needs as its
own fields, can host many distinct tasks, one per `Code`, and dispatch each to its own provider. Running
a task here means invoking the provider wired for that `Code` and awaiting an asynchronous
`Result<(), Error>`: the task either completes or produces the context's abstract error. The component
carries no input or output beyond success-or-error, so it models a fire-and-complete action rather than
a transformation, which separates it from the [handler family](./handler/index.md) that maps an
`Input` to an `Output`.

`CanRun` is the execution layer that ties a CGP application together: a runner provider typically reaches
the context's runtime through [`HasRuntime`](./has_runtime.md) to spawn or await work, and dispatches to
other components to do the actual job. Application code calls `CanRun` to set everything in
motion. Its companion [`CanSendRun`](./send_runner.md) is the variant whose returned future is `Send`,
for when the work must cross a thread boundary.

## Definition

`CanRun` is defined as:

```rust
#[cgp_component(Runner)]
#[async_trait]
#[derive_delegate(UseDelegate<Code>)]
#[use_type(HasErrorType.Error)]
pub trait CanRun<Code> {
    async fn run(&self, _code: PhantomData<Code>) -> Result<(), Error>;
}
```

Its attributes:

- [`#[cgp_component]`](../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `Runner` that implementations target and the wiring key `RunnerComponent`, while `CanRun` stays the consumer trait callers use.
- [`#[async_trait]`](../macros/async_trait.md) — rewrites the `async fn` into a method returning `-> impl Future`, the lint-clean, allocation-free form.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates a `UseDelegate` provider that dispatches on the `Code` type, so a context can route each `Code` to its own provider; the `open` statement is the modern sugar for the same dispatch.
- [`#[use_type]`](../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`CanRun` is imported from `cgp::extra::run`. Its method is `async` and takes only a `Code` tag:

```rust
async fn run(&self, _code: PhantomData<Code>) -> Result<(), Error>;
```

A context gains the capability by wiring `RunnerComponent` to a provider. Because the component dispatches
on `Code`, the idiomatic way a context hosts several tasks is the `open` statement, routing each task tag
to its own provider:

```rust
delegate_components! {
    App {
        open RunnerComponent;

        @RunnerComponent.ActionA: RunWithFooBar,
        @RunnerComponent.ActionB: SpawnAndRun<ActionA>,
    }
}
```

With this wiring, `app.run(PhantomData::<ActionA>)` runs the `RunWithFooBar` provider while
`app.run(PhantomData::<ActionB>)` runs `SpawnAndRun<ActionA>`. A runner provider is a normal CGP
provider written for the `Runner` provider trait; it receives `&Context` and the `PhantomData<Code>` tag
and may call other components on the context, fetching values or performing effects, before it completes.
Existing code often wires the same dispatch with the older `UseDelegate<Code>` table, which the `open`
statement supersedes.

## Examples

A spawning runner provider requires the context to be [`CanSendRun`](./send_runner.md) so it can hand a
`Send` future to a spawner, and runs an inner task on a background thread:

```rust
#[cgp_impl(new SpawnAndRun<InCode>: RunnerComponent)]
#[use_type(HasErrorType.Error)]
impl<Code, InCode> Runner<Code>
where
    Self: 'static + Send + Clone + CanSendRun<InCode>,
{
    async fn run(&self, _code: PhantomData<Code>) -> Result<(), Error> {
        let context = self.clone();

        spawn(async move {
            let _ = context.send_run(PhantomData).await;
        });

        Ok(())
    }
}
```

`SpawnAndRun<InCode>` runs the task `Code` by cloning the context and spawning the *inner* task `InCode`.
The spawner requires a `Send + 'static` future, which `context.send_run(PhantomData::<InCode>)`
returns, so the bound `Self: CanSendRun<InCode>` makes the spawn type-check. None of the abstract
task or error types carry `Send` bounds; that requirement is discharged only at the concrete
[`CanSendRun`](./send_runner.md) proxy. `App` is an **environmental context**, and the task is a
type-level `Code` selector the wiring dispatches on.

## When to reach for it, and when not

**Reach for `CanRun` to model a unit of work a context executes to completion**, such as a background
job, a startup action, or a scheduled task, especially when one context hosts several such tasks that
should each dispatch to their own provider. It is the entry point application code calls, and it composes
with [`HasRuntime`](./has_runtime.md) for the runtime the work runs on.

Reach for the [handler family](./handler/index.md) instead when the work transforms an `Input` into an
`Output` rather than only succeeding or failing, and reach for [`CanSendRun`](./send_runner.md) when the
future must be `Send` to be spawned across threads. For a synchronous action that cannot fail, a plain
method or a [`Computer`](./handler/computer.md) is simpler than a runner.

## Related constructs

- [`CanSendRun` / `SendRunner`](./send_runner.md) — the variant whose future is `Send`, for spawning
  across threads.
- [`HasRuntime`](./has_runtime.md) — the runtime a runner provider reaches to spawn or await work.
- [`HasErrorType`](./has_error_type.md) — the supertrait supplying the `Self::Error` a task fails with.
- [Handler family](./handler/index.md) — the computation components a runner dispatches to, which
  transform an input rather than only completing.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates the per-`Code` dispatch this uses.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the effectful computation family a runner drives.
- [Recovering `Send` bounds](/docs/concepts/send-bounds) — the workaround the `Send` runner variant
  embodies.

## Source

- `CanRun` / `Runner` and `CanSendRun` / `SendRunner`:
  [`cgp-run/src/lib.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-run/src/lib.rs),
  reached from the facade as `cgp::extra::run`.
- The `#[cgp_component]` and `#[derive_delegate]` expansions they rely on:
  [`cgp-macro-core/src/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
