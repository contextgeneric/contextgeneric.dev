//! Code from `docs/reference/providers/handler/promote_async_computer.md` — `PromoteAsyncComputer`.

/// ## Usage and Examples
///
/// `PromoteAsyncComputer` fills the family from an infallible async base. It answers
/// `HandlerComponent` (and the async-ref members) but not `AsyncComputerComponent`, so the base is
/// wired to `AsyncComputerComponent` directly and the bundle fills in the rest.
pub mod filling_the_family_from_an_async_computer {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::{AsyncComputerComponent, HandlerComponent, PromoteAsyncComputer};
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn double(value: u64) -> u64 {
        value * 2
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: RaiseFrom,

            AsyncComputerComponent: Double,
            HandlerComponent: PromoteAsyncComputer<Double>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                AsyncComputerComponent: ((), u64),
                HandlerComponent: ((), u64),
            }
        }
    }

    #[test]
    fn the_async_base_answers_the_family() {
        use cgp::extra::handler::{CanComputeAsync, CanHandle};
        use futures::executor::block_on;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(block_on(app.compute_async(code, 5u64)), 10);
        assert_eq!(block_on(app.handle(code, 5u64)), Ok(10));
    }
}
