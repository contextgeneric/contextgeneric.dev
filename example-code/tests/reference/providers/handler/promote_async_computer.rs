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

/// ## Under the hood
///
/// `AsyncComputerRefComponent` is `PromoteRef<P>`, one step from the base, so a hand-written
/// `AsyncComputer` whose input is a borrow answers it.
pub mod under_the_hood {
    use cgp::extra::handler::CanComputeAsyncRef;
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<'a, Context, Code> AsyncComputer<Context, Code, &'a u64> for DoubleRefLater {
        type Output = u64;

        async fn compute_async(
            _context: &Context,
            _code: PhantomData<Code>,
            input: &'a u64,
        ) -> u64 {
            input * 2
        }
    }

    pub struct App;

    delegate_components! {
        App {
            AsyncComputerRefComponent: PromoteAsyncComputer<DoubleRefLater>,
        }
    }

    check_components! {
        App {
            AsyncComputerRefComponent: ((), u64),
        }
    }

    #[test]
    fn test_compute_async_ref() {
        let output = futures::executor::block_on(App.compute_async_ref(PhantomData::<()>, &21));
        assert_eq!(output, 42);
    }
}
