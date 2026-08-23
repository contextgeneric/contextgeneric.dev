//! Code from `docs/reference/providers/error/raise_infallible.md` — `RaiseInfallible`.

/// ## Usage and Examples
///
/// `RaiseInfallible` fills the raiser slot for `core::convert::Infallible` on a context whose
/// operation cannot fail. The `Infallible` value is never constructed, so the raise path is wired
/// but never taken at run time; `check_components!` is what proves the wiring resolves. A second
/// source, a `String`, uses `RaiseFrom` beside it.
pub mod absorbing_an_impossible_error {
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
            // A step that cannot fail. Its `Infallible` error is raised uniformly, which lets this
            // provider be wired the same way as one whose step can fail.
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

    mod check_app {
        use super::*;
        check_components! { App { RunnerComponent } }
    }

    #[test]
    fn the_infallible_step_runs() {
        assert_eq!(App.run(), Ok(()));
    }
}
