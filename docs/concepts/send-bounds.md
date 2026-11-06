---
sidebar_label: 'Recovering Send bounds'
sidebar_position: 17
---

# Recovering `Send` bounds

Recovering a `Send` bound lets generic code require a sendable future from an async interface that
leaves that guarantee open. CGP's `#[async_trait]` returns an unboxed future without promising
`Send`, so a caller cannot infer sendability from the consumer trait alone. A companion trait can
expose the stronger guarantee, with forwarding implementations that verify it for concrete context
and API pairs.

## What the async trait promises

CGP's `#[async_trait]` rewrites an async method into a method returning `impl Future`. This
component fragment assumes imports from `cgp::prelude::*`:

```rust
#[cgp_component(ApiHandler)]
#[async_trait]
pub trait CanHandleApi<Api> {
    type Response;

    async fn handle_api(&self, _api: PhantomData<Api>) -> Self::Response;
}
```

After expansion, `handle_api` returns `impl Future<Output = Self::Response>`. The rewrite needs
neither boxing nor allocation, but its signature gives callers no `Send` guarantee. Even if a
particular implementation returns a sendable future, generic code with only
`App: CanHandleApi<Api>` cannot rely on that property.

Leaving `Send` out of the signature permits implementations whose futures cannot move between
threads. Others may return sendable futures, but generic callers have to support either case.
An explicit `+ Send` in the opaque return type would make the stronger promise available to them.

## Why spawning needs a stronger bound

The distinction becomes relevant when a task must move between threads. For example,
[`tokio::spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html) requires both the spawned
future and its output to be `Send + 'static`. If that task awaits `handle_api`, the awaited future
must allow the surrounding task to satisfy those requirements.

Adding `App: Send` does not close this gap. That bound allows the application value to move between
threads, but says nothing about the futures its methods return. The caller needs a requirement on
`handle_api`'s return type itself.

Return Type Notation would let a caller state the missing requirement directly:

```rust
// Not available on stable Rust:
fn spawn_handler<App, Api>(app: App)
where
    App: CanHandleApi<Api, handle_api(..): Send> + Send + 'static,
{
}
```

The `handle_api(..): Send` clause constrains the method's return type. The function body is omitted
here to isolate that bound. Return Type Notation is an
[unstable Rust feature](https://doc.rust-lang.org/unstable-book/language-features/return-type-notation.html),
so stable code needs another way to expose the guarantee.

## A companion trait with a sendable future

A companion trait can state `+ Send` directly on its method's return type. This declaration also
requires a sendable response and a `Send + Sync` context, and assumes `core::future::Future` is imported:

```rust
pub trait CanHandleApiSend<Api>: CanHandleApi<Api> + Send + Sync
where
    Self::Response: Send,
{
    fn handle_api_send(&self, api: PhantomData<Api>)
        -> impl Future<Output = Self::Response> + Send;
}
```

The resulting `CanHandleApiSend` is an ordinary trait, so it needs no component key or provider
wiring. By extending the original trait, it reuses its response type while adding a method whose
return type explicitly promises `Send`. Generic callers can request this stronger interface:

```rust
where
    App: CanHandleApiSend<Api>,
    App::Response: Send,
```

With these bounds, the caller can obtain a future known to be `Send` by calling `handle_api_send`.
Calling the original `handle_api` still gives only the original guarantee. The response bound also
appears at this use site because Rust requires the caller to establish the associated-type bound
from the companion trait's `where` clause.

The companion method supplies the missing sendability promise, but the task still needs to meet
the executor's other requirements. For example, a future borrowing a local `app` can be `Send`
without being `'static`. The surrounding task's lifetime and output therefore need attention
independently of this bound.

## Why forwarding implementations need concrete types

It might seem possible to implement this companion trait for every `App: CanHandleApi<Api>` by
awaiting `self.handle_api(api)`. But that implementation would still have to prove that the awaited
future is `Send`, and the generic bound supplies no such guarantee. Wrapping the call in another
async block carries the same uncertainty into the wrapper.

A concrete context and API let Rust check the actual selected implementation. In this fragment,
`MockApp` is wired to a `QueryBalance` provider whose response is `u64` and whose future is sendable:

```rust
impl CanHandleApiSend<QueryBalance> for MockApp {
    async fn handle_api_send(&self, api: PhantomData<QueryBalance>) -> u64 {
        self.handle_api(api).await
    }
}
```

Rust can now follow the concrete wiring and verify the selected provider's future against the
companion method's promise. If the provider holds a non-sendable value across an await, this
implementation can still fail. The forwarding body exposes a verified `Send` property rather than
changing the future's ability to move between threads.

## What it costs

Each context and API pair that exposes the stronger interface needs a forwarding implementation.
A macro can generate those repetitive bodies, but Rust must still verify the `Send` requirement
for each pair.

That also leaves another interface to maintain. Each method needing the stronger guarantee needs
its own signature and forwarding body. Methods without that requirement can remain on the original
[supertrait](/docs/reference/glossary#supertrait), where callers can still use them.

A companion trait is useful when the base interface must support both sendable and non-sendable
futures. If every implementation must return a `Send` future, an explicit `impl Future + Send`
return type on the base trait can express that requirement directly. If the task can stay on a
local executor, the additional guarantee may be unnecessary.

## Where to go next

These pages explain the async components and the wiring checked by a concrete forwarding impl:

- [Handlers](./handlers.md): CGP's async computation interfaces.
- [Consumer and provider traits](./consumer-and-provider-traits.md): How a context call reaches
  the selected provider.
- [`#[async_trait]`](/docs/reference/macros/async_trait): The unboxed future rewrite and its bounds.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
