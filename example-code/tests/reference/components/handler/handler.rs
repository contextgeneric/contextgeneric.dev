//! Code from `docs/reference/components/handler/handler.md` — `Handler`.
//!
//! Pins the page's Examples: a generic consumer bounded by `CanHandle`, and a context whose
//! `HandlerComponent` is a synchronous `Computer` lifted in two promotion steps. The one-step bundle
//! wiring from Common Mistakes is a trybuild fixture.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanHandle, Promote, PromoteAsync};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for Double {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            HandlerComponent: PromoteAsync<Promote<Double>>,
        }
    }

    check_components! {
        App {
            HandlerComponent: ((), u64),
        }
    }

    pub async fn run_with<Context, Code>(
        context: &Context,
        input: u64,
    ) -> Result<Context::Output, Context::Error>
    where
        Context: CanHandle<Code, u64>,
    {
        context.handle(PhantomData::<Code>, input).await
    }

    pub async fn demo() -> Result<u64, String> {
        run_with::<App, ()>(&App, 21).await
    }

    #[test]
    fn test_demo() {
        assert_eq!(futures::executor::block_on(demo()), Ok(42));
    }
}
