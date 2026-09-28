//! Code from `docs/reference/providers/error/panic_on_error.md` — `PanicOnError`.
//!
//! Pins the Examples program: a raise on `TestApp` panics with the source's `Debug` output. The
//! `#[should_panic]` test is what pins that behavior.

/// ## Examples
pub mod examples {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::PanicOnError;
    use cgp::prelude::*;

    #[cgp_component(Runner)]
    #[use_type(HasErrorType.Error)]
    pub trait CanRun {
        fn run(&self) -> Result<(), Error>;
    }

    #[cgp_impl(new RunOrAbort)]
    #[uses(CanRaiseError<String>)]
    #[use_type(HasErrorType.Error)]
    impl Runner {
        fn run(&self) -> Result<(), Error> {
            Err(Self::raise_error("unrecoverable".to_owned()))
        }
    }

    pub struct TestApp;

    delegate_components! {
        TestApp {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: PanicOnError,
            RunnerComponent: RunOrAbort,
        }
    }

    check_components! {
        TestApp {
            RunnerComponent,
        }
    }

    pub fn demo() {
        let _ = TestApp.run(); // panics with "\"unrecoverable\""
    }

    #[test]
    #[should_panic(expected = "\"unrecoverable\"")]
    fn test_demo() {
        demo();
    }
}
