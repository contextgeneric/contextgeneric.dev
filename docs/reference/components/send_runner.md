---
sidebar_label: 'CanSendRun'
sidebar_position: 9
---

# `CanSendRun`

The runner variant whose returned future is `Send`, for work that must cross a thread boundary.

## Overview

`CanSendRun<Code>` is [`CanRun`](./runner.md) with one added guarantee: the future its method returns is
[`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html), so it can be handed to a spawner such as
`tokio::spawn`. It exists to solve a specific Rust limitation. A generic `async fn` over abstract
**context** types (the context being the type a capability runs against that supplies the values an
implementation needs as its own fields) cannot promise its future is `Send` without annotating `Send`
bounds on every abstract type in scope, and those bounds pollute every interface.
`CanSendRun<Code>` sidesteps that by returning an explicit
`impl Future<Output = Result<(), Error>> + Send`, so the `Send` requirement lives on this one trait
rather than spreading across the abstract types.

A context implements `CanSendRun<Code>` as a thin proxy over its [`CanRun<Code>`](./runner.md)
implementation. Because the proxy is written against the *concrete* context, the compiler can confirm
the concrete future is `Send` without abstract-type annotations. This is a workaround that stays
necessary until Rust stabilizes Return Type Notation; once that lands, the `Send`-bound future could be
expressed without the separate trait. The pattern is developed as a concept in
[recovering `Send` bounds](/docs/concepts/send-bounds).

## Definition

`CanSendRun` is defined as:

```rust
#[cgp_component(SendRunner)]
#[async_trait]
#[derive_delegate(UseDelegate<Code>)]
#[use_type(HasErrorType.Error)]
pub trait CanSendRun<Code> {
    fn send_run(
        &self,
        _code: PhantomData<Code>,
    ) -> impl Future<Output = Result<(), Error>> + Send;
}
```

Its attributes:

- [`#[cgp_component]`](../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `SendRunner` that implementations target and the wiring key `SendRunnerComponent`, while `CanSendRun` stays the consumer trait callers use.
- [`#[async_trait]`](../macros/async_trait.md) — the attribute CGP's async trait methods carry; it rewrites an `async fn` into a lint-clean `-> impl Future` method. Here `send_run` is already written in that form so it can add the `+ Send` bound that [`CanRun`](./runner.md)'s `run` omits.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates a `UseDelegate` provider that dispatches on the `Code` type, so a context can route each `Code` to its own provider; the `open` statement is the modern sugar for the same dispatch.
- [`#[use_type]`](../attributes/use_type.md) — adds `HasErrorType` as a supertrait and rewrites the bare `Error` to `<Self as HasErrorType>::Error`.

## Usage

`CanSendRun` is imported from `cgp::extra::run`. Its method differs from `run` only in the shape of its
return, an explicit `Send` future rather than a plain `async fn`:

```rust
fn send_run(&self, _code: PhantomData<Code>) -> impl Future<Output = Result<(), Error>> + Send;
```

A context gains the capability by wiring `SendRunnerComponent`, or, in the common case, by supplying a
proxy impl on the concrete context that forwards to its own `run`. The proxy discharges the
`Send` bound:

```rust
#[cgp_provider]
impl SendRunner<App, ActionA> for App {
    async fn send_run(context: &App, code: PhantomData<ActionA>) -> Result<(), Infallible> {
        context.run(code).await
    }
}
```

Because this impl names the concrete `App` and `ActionA`, the future produced by `context.run(code)` has
a fully known type, so the compiler can verify it is `Send` and satisfy the `+ Send` bound on
`send_run`, which the generic [`CanRun`](./runner.md) definition deliberately does not assert. A spawning
runner provider can then require `Context: CanSendRun<InCode>`, clone the context into a `Send` future,
and hand it to a spawner, all without `Send` bounds leaking into the abstract interfaces.

## Examples

A spawning provider requires the context to be `CanSendRun` and hands a `Send` future to a spawner:

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

The spawner requires a `Send + 'static` future, which
`context.send_run(PhantomData::<InCode>)` returns, so the bound `Context: CanSendRun<InCode>`
makes the spawn type-check. The context wires its [`CanRun`](./runner.md) tasks and supplies a
`SendRunner` proxy for the inner task, and the `Send` bound is discharged only at that concrete proxy,
never on the abstract task or error types. `App` is an **environmental context**, and the task is a
type-level `Code` selector.

## When to use it

**Reach for `CanSendRun` when a runner's future must be `Send`**, which in practice means whenever the
work is handed to a work-stealing spawner like `tokio::spawn`. A provider that spawns bounds its context
by `CanSendRun<InCode>` and supplies a concrete proxy for each task it spawns, which keeps the `Send`
requirement out of every abstract interface.

Do not reach for it when the future is never spawned across a thread boundary, where plain
[`CanRun`](./runner.md) is enough and needs no proxy. `CanSendRun` is a targeted workaround for the
`Send`-future limitation rather than a default: use it only where the `Send` bound is actually required.

## Related constructs

- [`CanRun` / `Runner`](./runner.md) — the base runner this proxies, without the `Send` requirement.
- [`HasRuntime`](./has_runtime.md) — the runtime a spawning runner reaches to hand off the `Send` future.
- [`HasErrorType`](./has_error_type.md) — the supertrait supplying the `Self::Error` a task fails with.
- [`#[derive_delegate]`](../attributes/derive_delegate.md) — generates the per-`Code` dispatch this uses.

The ideas behind it:

- [Recovering `Send` bounds](/docs/concepts/send-bounds) — restoring the `Send` guarantee an async trait
  method drops, which this component embodies.
- [Handlers](/docs/concepts/handlers) — the effectful computation family a runner drives.

## Source

- `CanSendRun` / `SendRunner`, defined beside `CanRun`:
  [`cgp-run/src/lib.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-run/src/lib.rs),
  reached from the facade as `cgp::extra::run`.
- The `#[cgp_component]` and `#[derive_delegate]` expansions it relies on:
  [`cgp-macro-core/src/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
