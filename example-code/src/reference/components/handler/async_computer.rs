//! Code from `docs/reference/components/handler/async_computer.md` — `AsyncComputer`.
//!
//! Pins the generic consumer from the page's Examples, which awaits an async computation wired on its
//! context. It is generic over the context, so it compiles on its own.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanComputeAsync;

    pub async fn run<Context, Code>(context: &Context, input: u64) -> Context::Output
    where
        Context: CanComputeAsync<Code, u64>,
    {
        context.compute_async(PhantomData::<Code>, input).await
    }
}
