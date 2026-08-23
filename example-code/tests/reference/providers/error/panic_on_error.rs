//! Code from `docs/reference/providers/error/panic_on_error.md` — `PanicOnError`.

/// ## Usage and Examples
///
/// `PanicOnError` raises by panicking with the source error's `Debug` output rather than returning
/// an error value. It suits a context that treats an error as a fault to abort on, such as a test
/// harness. The `#[should_panic]` test is what pins that behavior.
pub mod aborting_on_error {
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

    mod check_test_app {
        use super::*;
        check_components! { TestApp { RunnerComponent } }
    }

    #[test]
    #[should_panic]
    fn raising_aborts_the_program() {
        let _ = TestApp.run();
    }
}
