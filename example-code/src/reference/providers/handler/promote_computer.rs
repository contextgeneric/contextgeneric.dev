//! Code from `docs/reference/providers/handler/promote_computer.md` — `PromoteComputer`.

/// ## Usage and Examples
///
/// `PromoteComputer` fills in every family member *other than* `ComputerComponent`. The base provider
/// answers `ComputerComponent`, and the bundle answers the rest, so one `Computer` implementation
/// serves the whole family.
pub mod filling_the_family_from_a_computer {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::{
        AsyncComputerComponent, ComputerComponent, HandlerComponent, PromoteComputer,
        TryComputerComponent,
    };
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

            // The base answers `ComputerComponent`; the bundle fills in the rest of the family.
            ComputerComponent: Double,
            [
                TryComputerComponent,
                AsyncComputerComponent,
                HandlerComponent,
            ]: PromoteComputer<Double>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: ((), u64),
                TryComputerComponent: ((), u64),
                AsyncComputerComponent: ((), u64),
                HandlerComponent: ((), u64),
            }
        }
    }

    #[test]
    fn one_computer_answers_the_family() {
        use cgp::extra::handler::{CanCompute, CanComputeAsync, CanHandle, CanTryCompute};
        use futures::executor::block_on;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(app.compute(code, 5u64), 10);
        assert_eq!(app.try_compute(code, 5u64), Ok(10));
        assert_eq!(block_on(app.compute_async(code, 5u64)), 10);
        assert_eq!(block_on(app.handle(code, 5u64)), Ok(10));
    }
}
