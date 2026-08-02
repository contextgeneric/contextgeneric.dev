---
sidebar_label: 'Recovering Send bounds'
sidebar_position: 17
---

# Recovering `Send` bounds

Restoring the guarantee an async trait method drops, so the future it returns can be spawned on a
multi-threaded runtime.

This page answers *why will my async CGP handler not spawn?* It is short, and it is about a gap in
stable Rust rather than about CGP — CGP just meets it sooner than most code does. It explains where the
guarantee goes, the bound that would fix it and does not exist yet, and the workaround. It closes on
what that workaround costs, which is repetition.

## Where the guarantee goes

An async method in a CGP trait is declared with `#[async_trait]`, which rewrites it into a method
returning `impl Future`:

```rust
#[cgp_component(ApiHandler)]
#[async_trait]
pub trait CanHandleApi<Api> {
    type Response;

    async fn handle_api(&self, _api: PhantomData<Api>) -> Self::Response;
}
```

That rewrite is faithful and costs nothing — no boxing, no allocation. It also drops every auto-trait
bound. The returned future is `Send` when the concrete future the body produces happens to be, and a
caller working through the trait has no way to *require* it.

The opacity is the trade. Return-position `impl Trait` is zero-cost precisely because the caller does
not see the concrete type, and the future's `Send`-ness is part of what is hidden.

## Why that matters, and what you cannot write

It matters the moment the future is spawned. A work-stealing runtime — the default Tokio runtime an Axum
server runs on — may move a task between threads while it is suspended, so every future it drives must
be `Send`. A generic handler awaiting `handle_api` produces a task that is `Send` only if that future
is.

For a *concrete* context the compiler checks this itself. For a generic one it cannot, because the fact
in question is the one the trait refuses to expose.

The bound you want has a name and is not stable:

```rust
// Not available on stable Rust:
fn spawn_handler<App, Api>(app: App)
where
    App: CanHandleApi<Api, handle_api(..): Send> + Send + 'static,
{
}
```

Return Type Notation — `handle_api(..): Send` — says exactly the right thing: whatever arguments the
method is called with, its future is `Send`. It is not stabilized, so this cannot be written in
production code today.

## Recovering it with a second trait

The workaround is to declare an ordinary trait whose method spells the bound out in its own return type,
where the notation is not needed:

```rust
pub trait CanHandleApiSend<Api>: CanHandleApi<Api> + Send + Sync
where
    Self::Response: Send,
{
    fn handle_api_send(&self, api: PhantomData<Api>)
        -> impl Future<Output = Self::Response> + Send;
}
```

`CanHandleApiSend` is **not a component**. It adds nothing to any wiring table and has no providers; it
exists to carry a stronger signature. It inherits the whole capability from `CanHandleApi` as a
supertrait, additionally requires the response and the context to be `Send`, and writes `+ Send` on the
future directly.

A spawning caller can now say what it needs, in one bound with no missing notation:

```rust
where
    App: CanHandleApiSend<Api>,
```

## Why the implementation cannot be generic

The obvious next step is one blanket implementation covering every context that already handles the API.
It does not compile, and the reason is worth following, because it is the same gap wearing a disguise.

Such an impl would wrap `self.handle_api(..)` in an `async` block, and that block is `Send` only if the
future it awaits is. For a generic `App` and `Api` the awaited future is an opaque `impl Future` whose
auto-traits are unknown — so the impl cannot prove its own `+ Send` return type. A generic blanket impl
*is* Return Type Notation, and it is blocked for the same reason.

Dropping to a concrete context and a concrete API closes it:

```rust
impl CanHandleApiSend<QueryBalance> for MockApp {
    async fn handle_api_send(&self, api: PhantomData<QueryBalance>) -> u64 {
        self.handle_api(api).await
    }
}
```

Now `Self` is a fixed type and `Api` is a fixed marker, so the call resolves through the wiring to a
concrete provider producing a concrete future — and the compiler computes that future's auto-traits and
finds it `Send`. No annotation is needed, because `Send` is inferred structurally for a known type.

Each of these impls is mechanical: forward, and await. Each is also a *proof*, accepted only because at
this instantiation the future really is `Send`.

## What it costs

**One impl per context per API.** That is the price of the missing notation, and it is the whole cost:
where RTN would have allowed a single generic impl, this needs one for every pair. A service with eight
endpoints and two contexts writes sixteen forwarding bodies.

**It is boilerplate that cannot be abstracted away**, since abstracting it is what does not compile. A
macro could generate it, and the impls would still be there.

**And it is a second trait to keep in step.** Adding a method to the capability means adding it here
too, and nothing enforces that the two stay aligned beyond the supertrait.

The one consolation is that the whole thing disappears when Return Type Notation stabilizes. This is a
workaround with a known expiry, not a design.

## Where to go next

[Handlers](./handlers.md) is the family whose futures most often need this, since it is where CGP's
async I/O lives. [Consumer and provider traits](./consumer-and-provider-traits.md) explains the wiring
the concrete impl forwards through, which is what makes the resolved future a concrete, checkable type.

For the constructs, [`#[async_trait]`](/docs/reference/macros/async_trait) is the rewrite that drops the
bound, and its own page records the same gap from the macro's side.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
