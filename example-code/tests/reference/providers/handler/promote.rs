//! Code from `docs/reference/providers/handler/promote.md` — `Promote`.
//!
//! Pins the Examples program: a hand-written `Computer` fills a `TryComputer` slot through
//! `Promote`, and a `Handler` slot through the two-step `PromoteAsync<Promote<...>>`, and a
//! hand-written `Producer` fills a `Computer` slot.

/// ## Examples
pub mod examples {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanCompute, CanHandle, CanTryCompute, Promote, PromoteAsync};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for Double {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    #[cgp_new_provider]
    impl<Context, Code> Producer<Context, Code> for DefaultPort {
        type Output = u16;

        fn produce(_context: &Context, _code: PhantomData<Code>) -> u16 {
            8080
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ComputerComponent: Promote<DefaultPort>,
            TryComputerComponent: Promote<Double>,
            HandlerComponent: PromoteAsync<Promote<Double>>,
        }
    }

    check_components! {
        App {
            ComputerComponent: ((), ()),
            [TryComputerComponent, HandlerComponent]: ((), u64),
        }
    }

    pub async fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.compute(code, ()), 8080);
        assert_eq!(App.try_compute(code, 21), Ok(42));
        assert_eq!(App.handle(code, 21).await, Ok(42));
    }

    #[test]
    fn test_demo() {
        futures::executor::block_on(demo());
    }
}
