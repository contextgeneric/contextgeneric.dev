//! Code from `docs/reference/components/handler/handler.md` — `Handler`.
//!
//! Pins the generic consumer from the page's Examples, which bounds its context by `CanHandle` and so
//! accepts any wired computation. It is generic over the context, so it compiles on its own.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanHandle;

    pub async fn run_with<Context, Code>(
        context: &Context,
        input: String,
    ) -> Result<Context::Output, Context::Error>
    where
        Context: CanHandle<Code, String>,
    {
        context.handle(PhantomData::<Code>, input).await
    }
}
