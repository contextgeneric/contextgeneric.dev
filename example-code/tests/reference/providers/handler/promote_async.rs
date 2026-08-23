//! Code from `docs/reference/providers/handler/promote_async.md` — `PromoteAsync`.

/// ## Usage and Examples
///
/// `PromoteAsync` lifts a synchronous provider into an asynchronous one. Here a plain `Computer`
/// (`Double`) fills an `AsyncComputer` slot; the returned future is ready immediately.
pub mod lifting_a_computer_into_an_async_computer {
    use cgp::extra::handler::{AsyncComputerComponent, PromoteAsync};
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn double(value: u64) -> u64 {
        value * 2
    }

    pub struct App;

    delegate_components! {
        App {
            AsyncComputerComponent: PromoteAsync<Double>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                AsyncComputerComponent: ((), u64),
            }
        }
    }

    #[test]
    fn the_synchronous_provider_runs_in_an_async_method() {
        use cgp::extra::handler::CanComputeAsync;
        use futures::executor::block_on;

        assert_eq!(block_on(App.compute_async(PhantomData::<()>, 5u64)), 10);
    }
}
