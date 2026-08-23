//! Code from `docs/reference/providers/error/return_error.md` — `ReturnError`.

/// ## Usage and Examples
///
/// `ReturnError` raises a source that already *is* the context's error type, so it returns its
/// argument. Here the context's error is `AppError`: an `AppError` source is raised through
/// `ReturnError`, and a `ParseIntError` is formatted and forwarded to the `String` route that
/// `RaiseFrom` converts (`AppError: From<String>`).
pub mod returning_the_error_itself {
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::{DebugError, RaiseFrom, ReturnError};
    use cgp::prelude::*;

    #[derive(Debug, PartialEq)]
    pub struct AppError {
        pub message: String,
    }

    impl From<String> for AppError {
        fn from(message: String) -> Self {
            AppError { message }
        }
    }

    #[cgp_component(Checker)]
    #[use_type(HasErrorType.Error)]
    pub trait CanCheck {
        fn check(&self, raw: &str) -> Result<u16, Error>;
    }

    #[cgp_impl(new CheckPort)]
    #[uses(CanRaiseError<AppError>, CanRaiseError<ParseIntError>)]
    #[use_type(HasErrorType.{Error = AppError})]
    impl Checker {
        fn check(&self, raw: &str) -> Result<u16, AppError> {
            let parsed: u32 = raw.parse().map_err(Self::raise_error)?;

            if parsed == 0 {
                // Already an `AppError`, raised by returning it.
                return Err(Self::raise_error(AppError {
                    message: "port 0 is reserved".to_owned(),
                }));
            }

            Ok(parsed as u16)
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<AppError>,
            CheckerComponent: CheckPort,

            @ErrorRaiserComponent.AppError: ReturnError,
            @ErrorRaiserComponent.String: RaiseFrom,
            @ErrorRaiserComponent.ParseIntError: DebugError,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { CheckerComponent } }
    }

    #[test]
    fn an_app_error_source_is_returned_unchanged() {
        assert_eq!(App.check("8080").unwrap(), 8080);
        assert_eq!(
            App.check("0").unwrap_err(),
            AppError { message: "port 0 is reserved".to_owned() },
        );
    }
}
