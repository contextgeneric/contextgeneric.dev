//! Code from `docs/reference/components/handler/handler_ref.md` — `HandlerRef`.
//!
//! Pins the generic consumer from the page's Examples, which awaits a fallible computation over a
//! borrowed input. It is generic over the context, so it compiles on its own.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanHandleRef;

    pub async fn serve<Context, Code>(
        context: &Context,
        request: &String,
    ) -> Result<Context::Output, Context::Error>
    where
        Context: CanHandleRef<Code, String>,
    {
        context.handle_ref(PhantomData::<Code>, request).await
    }
}
