---
sidebar_label: 'Recovering Send bounds'
sidebar_position: 17
---

# Recovering `Send` bounds

A generic caller needs an explicit guarantee that an async method returns a `Send` future before
it can move that future between threads. CGP's `#[async_trait]` does not add that guarantee.
This page explains the trait-bound gap and a companion-trait workaround, including why it requires
forwarding implementations for concrete context and API pairs.

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

The generated return type is `impl Future<Output = Self::Response>`. The rewrite adds neither
boxing nor allocation, and it does not promise that the future implements `Send`. A concrete
implementation may return a `Send` future, but the generic bound `App: CanHandleApi<Api>` alone
does not establish that fact.

The missing guarantee comes from the trait signature. Opaque return types can explicitly carry
`+ Send`; this particular signature leaves that requirement open so implementations can also use
futures that are not sendable.

## Why spawning needs a stronger bound

Spawning APIs may require both the future and its output to be sendable. For example,
[`tokio::spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html) requires both to be
`Send + 'static`. An async task that awaits `handle_api` needs to satisfy those bounds as a whole.

A generic caller cannot infer the awaited future's `Send` implementation from
`App: CanHandleApi<Api>`. Giving `App` itself a `Send` bound does not constrain every future its
methods return.

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

`CanHandleApiSend` is an ordinary trait that adds a stronger method signature. It inherits the
original response type and needs neither a component key nor provider wiring. A generic caller can
require it with these bounds:

```rust
where
    App: CanHandleApiSend<Api>,
    App::Response: Send,
```

The caller must use `handle_api_send` to obtain the advertised future. The companion trait does not
change the signature of `handle_api`. Its response bound is repeated here because a bound on an
associated type in a trait's `where` clause must also be established at the generic use site.

This trait addresses the future's sendability, not every condition for spawning. The surrounding
task must still satisfy the executor's lifetime and output requirements. A future borrowing a local
`app` is not automatically `'static` merely because it implements `Send`.

## Why forwarding implementations need concrete types

A blanket forwarding implementation cannot prove `Send` from the original trait bound alone.
Its body would await `self.handle_api(api)`, whose future the original signature does not guarantee
to be sendable. Wrapping that call in another async block preserves the same missing requirement.

A concrete context and API let Rust check the actual selected implementation. In this fragment,
`MockApp` is wired to a `QueryBalance` provider whose response is `u64` and whose future is sendable:

```rust
impl CanHandleApiSend<QueryBalance> for MockApp {
    async fn handle_api_send(&self, api: PhantomData<QueryBalance>) -> u64 {
        self.handle_api(api).await
    }
}
```

The compiler resolves this call through the concrete wiring and checks that the returned future
satisfies the companion trait's `Send` promise. A provider that holds a non-sendable value across
an await can still make this implementation fail. The forwarding method verifies the property;
it does not make an otherwise non-sendable future safe to transfer.

## What it costs

This forwarding pattern needs an implementation for each context and API pair that exposes the
stronger interface. A macro can generate the repetitive bodies, but each generated implementation
still needs to satisfy the bound independently.

The companion trait adds an interface to maintain. Methods that need the stronger guarantee need
corresponding signatures and forwarding bodies. Other methods can remain available through the
original supertrait without being duplicated.

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
