//! Code from `docs/reference/components/handler/try_computer_ref.md` — `TryComputerRef`.
//!
//! Pins the generic consumer from the page's Examples, which runs a fallible computation over a
//! borrowed input. It is generic over the context, so it compiles on its own.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanTryComputeRef;

    pub fn check<Context, Code>(
        context: &Context,
        input: &String,
    ) -> Result<Context::Output, Context::Error>
    where
        Context: CanTryComputeRef<Code, String>,
    {
        context.try_compute_ref(PhantomData::<Code>, input)
    }
}
