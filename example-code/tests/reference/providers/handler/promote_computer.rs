//! Code from `docs/reference/providers/handler/promote_computer.md` — `PromoteComputer`.
//!
//! Pins the Examples program: a `#[cgp_computer]` provider, wired to `PromoteComputer<Self>` by the
//! macro, answers the whole family when the context wires the bundle for it. The hand-written base
//! whose `Handler` entry fails is the C2 fixture
//! `tests/compile_fail/reference/components/handler_common_mistakes_bundle_on_a_plain_computer.rs`.

/// ## Examples
pub mod examples {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanCompute, CanComputeAsync, CanHandle, CanTryCompute};
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn double(value: u64) -> u64 {
        value * 2
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ComputerComponent: Double,
            [
                TryComputerComponent,
                AsyncComputerComponent,
                HandlerComponent,
            ]: PromoteComputer<Double>,
        }
    }

    check_components! {
        App {
            [
                ComputerComponent,
                TryComputerComponent,
                AsyncComputerComponent,
                HandlerComponent,
            ]: ((), u64),
        }
    }

    pub async fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.compute(code, 5), 10);
        assert_eq!(App.try_compute(code, 5), Ok(10));
        assert_eq!(App.compute_async(code, 5).await, 10);
        assert_eq!(App.handle(code, 5).await, Ok(10));
    }

    #[test]
    fn test_demo() {
        futures::executor::block_on(demo());
    }
}

/// ## Common Mistakes
///
/// The one-step entries work for a hand-written base: `TryComputerComponent` goes to `Promote` and
/// `AsyncComputerComponent` to `PromoteAsync`, each of which needs only the `Computer`.
pub mod common_mistakes {
    use cgp::core::error::ErrorTypeProviderComponent;
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
            [TryComputerComponent, AsyncComputerComponent]: PromoteComputer<Double>,
        }
    }

    check_components! {
        App {
            [TryComputerComponent, AsyncComputerComponent]: ((), u64),
        }
    }
}
