//! Code from `docs/reference/providers/handler/promote.md` — `Promote`.

/// ## Usage and Examples
///
/// `Promote` lifts a provider up the family. Here a plain `Computer` (`Double`) fills a `TryComputer`
/// slot: `Promote` wraps its infallible result in `Ok`.
pub mod lifting_a_computer_into_a_try_computer {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::{Promote, TryComputerComponent};
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
            TryComputerComponent: Promote<Double>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                TryComputerComponent: ((), u64),
            }
        }
    }

    #[test]
    fn the_infallible_result_is_wrapped_in_ok() {
        use cgp::extra::handler::CanTryCompute;

        assert_eq!(App.try_compute(PhantomData::<()>, 5u64), Ok(10));
    }
}
