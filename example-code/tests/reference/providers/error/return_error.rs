//! Code from `docs/reference/providers/error/return_error.md` — `ReturnError`.
//!
//! Pins the Examples program: an `AppError` source is raised through `ReturnError` by returning it,
//! beside a `String` converted by `RaiseFrom` and a `ParseIntError` formatted by `DebugError`.

/// ## Usage
///
/// The whole-component wiring the section shows: every raise on the context goes to `ReturnError`,
/// which type-checks only for the source equal to the context's error.
pub mod usage {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::ReturnError;
    use cgp::prelude::*;

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: ReturnError,
        }
    }

    pub fn raise_own_error<Context>(message: String) -> Context::Error
    where
        Context: CanRaiseError<String>,
    {
        Context::raise_error(message)
    }

    #[test]
    fn the_source_is_returned_unchanged() {
        assert_eq!(raise_own_error::<App>("reserved".to_owned()), "reserved");
    }
}

/// ## Examples
pub mod examples {
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

    #[cgp_component(PortChecker)]
    #[use_type(HasErrorType.Error)]
    pub trait CanCheckPort {
        fn check_port(&self, raw: &str) -> Result<u16, Error>;
    }

    #[cgp_impl(new CheckPort)]
    #[uses(CanRaiseError<AppError>, CanRaiseError<ParseIntError>)]
    #[use_type(HasErrorType.{Error = AppError})]
    impl PortChecker {
        fn check_port(&self, raw: &str) -> Result<u16, AppError> {
            let port: u16 = raw.parse().map_err(Self::raise_error)?;

            if port == 0 {
                return Err(Self::raise_error(AppError {
                    message: "port 0 is reserved".to_owned(),
                }));
            }

            Ok(port)
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<AppError>,
            PortCheckerComponent: CheckPort,

            @ErrorRaiserComponent.AppError: ReturnError,
            @ErrorRaiserComponent.String: RaiseFrom,
            @ErrorRaiserComponent.ParseIntError: DebugError,
        }
    }

    check_components! {
        App {
            PortCheckerComponent,
        }
    }

    pub fn demo() {
        assert_eq!(App.check_port("8080"), Ok(8080));
        assert_eq!(
            App.check_port("0"),
            Err(AppError { message: "port 0 is reserved".to_owned() })
        );
        assert!(App.check_port("http").unwrap_err().message.contains("ParseIntError"));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
