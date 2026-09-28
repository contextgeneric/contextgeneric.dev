//! Code from `docs/reference/providers/handler/promote_try_computer.md` — `PromoteTryComputer`.
//!
//! Pins the Examples program, a `#[cgp_computer]` function returning `Result` whose provider answers
//! the family through the bundle, and the one-step entries a hand-written base can use. The
//! hand-written base behind the `Handler` entry is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanHandle, CanTryCompute};
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn checked_double(value: u64) -> Result<u64, String> {
        value.checked_mul(2).ok_or_else(|| "overflow".to_owned())
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            [TryComputerComponent, HandlerComponent]: PromoteTryComputer<CheckedDouble>,
        }
    }

    check_components! {
        App {
            [TryComputerComponent, HandlerComponent]: ((), u64),
        }
    }

    pub async fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.try_compute(code, 21), Ok(42));
        assert_eq!(App.try_compute(code, u64::MAX), Err("overflow".to_owned()));
        assert_eq!(App.handle(code, 21).await, Ok(42));
    }

    #[test]
    fn test_demo() {
        futures::executor::block_on(demo());
    }
}

/// ## Common Mistakes
///
/// The `TryComputerComponent` and `AsyncComputerComponent` entries are `TryPromote<P>` and
/// `PromoteAsync<P>`, one step from the base, so they serve a hand-written `Computer` returning
/// `Result`; the async one keeps the `Result` as its output.
pub mod common_mistakes {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanComputeAsync, PromoteAsync, TryPromote};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for CheckedDouble
    where
        Context: HasErrorType<Error = String>,
    {
        type Output = Result<u64, String>;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> Result<u64, String> {
            input.checked_mul(2).ok_or_else(|| "overflow".to_owned())
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            [TryComputerComponent, AsyncComputerComponent]: PromoteTryComputer<CheckedDouble>,
            // The fix the page gives for the handler slot.
            HandlerComponent: PromoteAsync<TryPromote<CheckedDouble>>,
        }
    }

    check_components! {
        App {
            [TryComputerComponent, AsyncComputerComponent, HandlerComponent]: ((), u64),
        }
    }

    pub async fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.compute_async(code, u64::MAX).await, Err("overflow".to_owned()));
    }

    #[test]
    fn test_demo() {
        futures::executor::block_on(demo());
    }
}
