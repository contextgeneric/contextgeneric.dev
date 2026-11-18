//! Code from `docs/reference/providers/error/raise_infallible.md` — `RaiseInfallible`.
//!
//! Pins the Examples program: a provider raises an `Infallible` error uniformly, and the context fills
//! that raiser slot with `RaiseInfallible`. The value is never constructed, so the raise path is wired
//! but never taken.

/// ## Examples
pub mod examples {
    use core::convert::Infallible;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::{RaiseFrom, RaiseInfallible};
    use cgp::prelude::*;

    #[cgp_component(Runner)]
    #[use_type(HasErrorType.Error)]
    pub trait CanRun {
        fn run(&self) -> Result<(), Error>;
    }

    #[cgp_impl(new RunInfallibly)]
    #[uses(CanRaiseError<Infallible>)]
    #[use_type(HasErrorType.Error)]
    impl Runner {
        fn run(&self) -> Result<(), Error> {
            // A step that cannot fail, raised the same way a fallible step would be.
            let outcome: Result<(), Infallible> = Ok(());
            outcome.map_err(Self::raise_error)?;
            Ok(())
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            RunnerComponent: RunInfallibly,

            @ErrorRaiserComponent.Infallible: RaiseInfallible,
            @ErrorRaiserComponent.String: RaiseFrom,
        }
    }

    check_components! {
        App {
            RunnerComponent,
        }
    }

    pub fn demo() {
        assert_eq!(App.run(), Ok(()));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
