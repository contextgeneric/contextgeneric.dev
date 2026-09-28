//! Code from `docs/reference/providers/handler/promote_async_computer.md` — `PromoteAsyncComputer`.
//!
//! Pins the Examples program: an async `#[cgp_computer]` function is the `AsyncComputer` base, and
//! the bundle answers `HandlerComponent` from it, one step away, as it would for a hand-written base.

/// ## Examples
pub mod examples {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanComputeAsync, CanHandle};
    use cgp::prelude::*;

    #[cgp_computer]
    pub async fn double_later(value: u64) -> u64 {
        value * 2
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            AsyncComputerComponent: DoubleLater,
            HandlerComponent: PromoteAsyncComputer<DoubleLater>,
        }
    }

    check_components! {
        App {
            [AsyncComputerComponent, HandlerComponent]: ((), u64),
        }
    }

    pub async fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.compute_async(code, 5).await, 10);
        assert_eq!(App.handle(code, 5).await, Ok(10));
    }

    #[test]
    fn test_demo() {
        futures::executor::block_on(demo());
    }
}
