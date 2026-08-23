//! Code from `docs/reference/components/handler/async_computer_ref.md` — `AsyncComputerRef`.
//!
//! Pins the generic consumer from the page's Examples, which awaits an infallible computation over a
//! borrowed input. It is generic over the context, so it compiles on its own.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanComputeAsyncRef;

    pub async fn scan<Context, Code>(context: &Context, input: &String) -> Context::Output
    where
        Context: CanComputeAsyncRef<Code, String>,
    {
        context.compute_async_ref(PhantomData::<Code>, input).await
    }
}
