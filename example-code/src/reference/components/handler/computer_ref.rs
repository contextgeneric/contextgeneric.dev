//! Code from `docs/reference/components/handler/computer_ref.md` — `ComputerRef`.
//!
//! Pins the generic consumer from the page's Examples, which computes from a borrowed input. It is
//! generic over the context, so it compiles on its own.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanComputeRef;

    pub fn measure<Context, Code>(context: &Context, input: &String) -> Context::Output
    where
        Context: CanComputeRef<Code, String>,
    {
        context.compute_ref(PhantomData::<Code>, input)
    }
}
