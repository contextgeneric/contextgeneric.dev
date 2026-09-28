//! Code from `docs/reference/providers/handler/promote_async.md` — `PromoteAsync`.
//!
//! Pins the Examples program: a hand-written `Computer` serves an `AsyncComputer` slot, and a
//! hand-written `TryComputer` serves a `Handler` slot.

/// ## Examples
pub mod examples {
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::DebugError;
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::{CanComputeAsync, CanHandle, PromoteAsync};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for Double {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    #[cgp_impl(new ParseU64)]
    #[uses(CanRaiseError<ParseIntError>)]
    #[use_type(HasErrorType.Error)]
    impl<Code> TryComputer<Code, String> {
        type Output = u64;

        fn try_compute(&self, _code: PhantomData<Code>, input: String) -> Result<u64, Error> {
            input.parse().map_err(Self::raise_error)
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            @ErrorRaiserComponent.String: RaiseFrom,
            @ErrorRaiserComponent.ParseIntError: DebugError,

            AsyncComputerComponent: PromoteAsync<Double>,
            HandlerComponent: PromoteAsync<ParseU64>,
        }
    }

    check_components! {
        App {
            AsyncComputerComponent: ((), u64),
            HandlerComponent: ((), String),
        }
    }

    pub async fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.compute_async(code, 21).await, 42);
        assert_eq!(App.handle(code, "12".to_owned()).await, Ok(12));
    }

    #[test]
    fn test_demo() {
        futures::executor::block_on(demo());
    }
}
