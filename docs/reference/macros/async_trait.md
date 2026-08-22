---
sidebar_label: '#[async_trait]'
sidebar_position: 12
---

# `#[async_trait]`

Rewrite a trait's `async fn` declarations into the lint-clean `-> impl Future` form CGP's async methods use.

## Overview

Writing a bare `async fn` inside a trait compiles on stable Rust, and the compiler warns about it. The
`async_fn_in_trait` lint fires because the future such a method returns is *opaque*: a caller working through
the trait cannot name it, so they cannot require anything of it, most importantly that it is `Send`. The
hand-written way to silence the lint is to declare the method as a future-returning function instead, which
is correct and obscures the intent:

```rust
fn fetch(&self, id: &str) -> impl Future<Output = Result<Vec<u8>, String>>;
```

`#[async_trait]` lets you write the natural form and performs that rewrite mechanically:

```rust
#[async_trait]
pub trait CanFetch {
    async fn fetch(&self, id: &str) -> Result<Vec<u8>, String>;
}
```

The trait reads as async code, and the declaration the compiler sees is the lint-clean one.

**The rewrite is a plain desugaring, not a framework.** Unlike the widely-used `async-trait` crate, nothing
here boxes the future or allocates: it is return-position `impl Trait` in traits, so the future is exactly the
one the body produces and the call costs what a hand-written future-returning method costs. That is why the
macro is used throughout CGP wherever a capability is asynchronous: it is simply how an async method is
spelled.

One thing it does *not* do is add a `Send` bound, and that omission has consequences the moment a future is
spawned. It is covered under [Gotchas](#gotchas).

## Usage

Apply the attribute to a trait definition. It takes no arguments. Tokens in the argument position are
ignored, so it is always written bare:

```rust
#[async_trait]
pub trait CanFetch {
    async fn fetch(&self, id: &str) -> Result<Vec<u8>, String>;
}
```

Only methods declared `async` are touched. Non-async methods, associated types, and associated constants in
the same trait pass through unchanged, so a trait can mix them freely.

### Ordering with a host macro

When stacked with another macro, the order follows what that macro needs, and the two cases differ.

With [`#[cgp_component]`](./cgp_component.md), put `#[async_trait]` **outermost**, so it rewrites the trait
before the component macro reads it:

```rust
#[async_trait]
#[cgp_component(StorageObjectFetcher)]
pub trait CanFetchStorageObject {
    async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, String>;
}
```

With [`#[cgp_fn]`](./cgp_fn.md), which *generates* the trait from a function, put it **below** `#[cgp_fn]` on
the `async fn`. `#[cgp_fn]` copies the attribute onto both items it generates:

```rust
#[cgp_fn]
#[async_trait]
pub async fn fetch_storage_object(
    &self,
    #[implicit] storage_client: &Client,
    object_id: &str,
) -> Result<Vec<u8>, String> {
    /* ... */
}
```

## Examples

The common case is declaring an asynchronous component. The consumer trait carries the attribute so its method
is a clean declaration, and each provider implements it with an ordinary `async fn` body:

```rust
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;

#[async_trait]
#[cgp_component(StorageObjectFetcher)]
#[use_type(HasErrorType.Error)]
pub trait CanFetchStorageObject {
    async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, Error>;
}

#[cgp_impl(new FetchFromBucket)]
#[use_type(HasErrorType.Error)]
impl StorageObjectFetcher {
    async fn fetch_storage_object(
        &self,
        #[implicit] bucket_id: &str,
        object_id: &str,
    ) -> Result<Vec<u8>, Error> {
        // await a real client here
        Ok(format!("{bucket_id}/{object_id}").into_bytes())
    }
}

#[derive(HasField)]
pub struct App {
    pub bucket_id: String,
}

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        StorageObjectFetcherComponent: FetchFromBucket,
    }
}
```

Notice the provider needs no `#[async_trait]` of its own. An `async fn` is already legal in an impl block,
since only a trait *declaration* trips the lint, so the provider keeps the natural body while the trait carries
the rewritten signature, and the two agree because an `async fn` desugars to exactly such a future-returning
method.

## When to reach for it, and when not

**Use `#[async_trait]` on every CGP trait with an async method.** There is no judgement to make here: it is
how an async method is declared, and the alternative is either a lint warning or writing the `impl Future`
signature by hand.

The decisions worth making are around it rather than about it.

- **Do not put it on an impl block.** It is accepted there and does nothing, since the rewrite only applies to
  trait definitions. Harmless, but it suggests a misunderstanding of where the lint comes from.
- **Reach for the `Send`-recovery pattern when a future is spawned**, not for a different macro. No attribute
  can add the bound, for the reason in the [Gotchas](#gotchas).
- **Consider whether the capability needs to be async at all.** The [handler family](./cgp_computer.md) has
  synchronous members, and the [promotion combinators](../providers/handler_combinators.md) lift a synchronous
  provider into an async one where a caller needs it. So a computation that does no I/O is better declared
  synchronous and promoted than declared async out of habit.
- **Reach for the `async-trait` crate instead only if you need `dyn` compatibility.** Boxing makes an
  async trait object-safe, and this macro deliberately does not box. CGP resolves providers statically, so it
  does not need `dyn`; a codebase that does for other reasons is outside what this macro is for.

## Under the hood

:::note

### Advanced

This section shows the rewrite. It is the simplest expansion in CGP, and worth seeing once because it explains
why a provider needs no attribute of its own. `cargo cgp expand` prints the same thing for your own code.

:::

For each `async` method the macro removes the `async` keyword and wraps the return type in
`impl ::core::future::Future<Output = …>`. A method with no return arrow is treated as returning `()`. From
this input:

```rust
#[async_trait]
pub trait CanFetch {
    async fn fetch(&self, id: &str) -> Result<Vec<u8>, String>;
    async fn run(&self);
    fn sync_method(&self) -> u8;
}
```

the macro emits:

```rust
pub trait CanFetch {
    fn fetch(
        &self,
        id: &str,
    ) -> impl ::core::future::Future<Output = Result<Vec<u8>, String>>;
    fn run(&self) -> impl ::core::future::Future<Output = ()>;
    fn sync_method(&self) -> u8;
}
```

Three things to read off it. The `Output` is the original return type verbatim. The bodiless `run` picked up
`Output = ()`. And `sync_method` was left completely alone, which lets a trait mix async and
synchronous methods.

**The macro rewrites only trait definitions.** Applied to anything else, most importantly an `impl` block,
it returns the tokens unchanged. This passthrough makes the composition work: an `async fn` is legal in
an impl already, so the provider's body stays as written while the trait's declaration carries the future
type, and the `async fn` satisfies it because an `async fn` desugars to precisely that.

The composition with [`#[cgp_fn]`](./cgp_fn.md) shows both halves at once. `#[cgp_fn]` first produces a trait
and a blanket impl, attaching `#[async_trait]` to each:

```rust
#[async_trait]
pub trait FetchStorageObject {
    async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, String>;
}

#[async_trait]
impl<__Context__> FetchStorageObject for __Context__
where
    Self: HasField<Symbol!("storage_client"), Value = Client>,
{
    async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, String> {
        let storage_client: &Client =
            self.get_field(PhantomData::<Symbol!("storage_client")>);
        /* ... */
    }
}
```

`#[async_trait]` then runs on both. On the trait it rewrites the declaration; on the impl it is a no-op, so
the `async fn` body survives intact.

## Gotchas

**The generated future carries no `Send` bound**, and this is the limitation that matters in practice. Because
the rewrite produces a bare `impl Future<Output = T>`, the future is `Send` only when the concrete future
happens to be, and the trait does not require it. So code that spawns the future onto a multi-threaded,
work-stealing executor cannot express what it needs through this trait.

The bound you would want to write is Return Type Notation (`App: CanFetch<fetch(..): Send>`), which is not
stabilized, so it cannot be written today. The workaround is to declare a second, ordinary trait whose method
spells `+ Send` on its return type directly, and to implement it for each concrete context. That pattern is
mechanical but unavoidable; the opacity that makes the rewrite zero-cost is the same opacity that hides the
auto-traits.

**A default-bodied async method is mishandled.** The macro rewrites the *signature* and never the body, so a
method with a default gets its `async` stripped and its return type changed while the body stays a plain
expression:

```text
error[E0277]: `{integer}` is not a future
```

That error names the body's type rather than the rewrite that caused it. In practice this is rarely hit,
because async trait methods are almost always declarations with the behaviour supplied by a provider. But a
default-bodied `async fn` inside an `#[async_trait]` trait is not supported. Wrap the body in an
`async { … }` block by hand, or move it to a provider.

**Attribute arguments are silently ignored.** `#[async_trait(anything)]` is accepted and discarded rather than
rejected, so a typo'd option gives no feedback. Always write it bare.

**On an impl block it does nothing**, which is correct but can mislead. Seeing it on a provider suggests the
provider needed it; it did not, and removing it changes nothing.

## Related constructs

- [`#[cgp_component]`](./cgp_component.md) — the usual host, with `#[async_trait]` placed outermost.
- [`#[cgp_fn]`](./cgp_fn.md) — the other host, with `#[async_trait]` placed beneath it.
- [`#[cgp_impl]`](./cgp_impl.md) — where a provider writes an ordinary `async fn` body and needs no attribute.
- [`Handler`](../components/handler.md) — CGP's built-in async, fallible component, declared this way.
- [`#[cgp_computer]`](./cgp_computer.md) — where an `async` function selects the async base trait.
- [Handler combinators](../providers/handler_combinators.md) — the promotions that lift a synchronous provider
  into an async one.

The ideas behind it:

- [Recovering `Send` bounds](/docs/concepts/send-bounds) — why the future this rewrite produces
  carries no `Send` bound, and how to get one back.
- [Handlers](/docs/concepts/handlers) — the component family whose async members are declared this
  way.

## Source

- Entry point: [`cgp-async-macro/src/lib.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-async-macro/src/lib.rs)
- The rewrite: [`impl_async.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-async-macro/src/impl_async.rs)
- Prelude re-export: [`cgp-core/src/prelude.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/main/cgp-core/src/prelude.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
