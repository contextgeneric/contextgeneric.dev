//! Code from `docs/concepts/modular-error-handling.md` — *Modular error handling*.

/// ## Code that fails without knowing what failing means
pub mod code_that_fails_without_knowing {
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::{DebugError, RaiseFrom};
    use cgp::prelude::*;

    #[cgp_component(PortParser)]
    #[use_type(HasErrorType.Error)]
    pub trait CanParsePort {
        fn parse_port(&self, raw: &str) -> Result<u16, Error>;
    }

    #[cgp_impl(new ParsePortFromStr)]
    #[uses(CanRaiseError<ParseIntError>, CanRaiseError<String>)]
    #[use_type(HasErrorType.Error)]
    impl PortParser {
        fn parse_port(&self, raw: &str) -> Result<u16, Error> {
            let parsed: u32 = raw.parse().map_err(Self::raise_error)?;

            if parsed > u16::MAX as u32 {
                return Err(Self::raise_error(format!("port {parsed} out of range")));
            }

            Ok(parsed as u16)
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            PortParserComponent: ParsePortFromStr,

            // Two source errors, two strategies. A `String` is already the error type, so it is
            // converted through `From`; a parse error is formatted with `Debug` and forwarded back
            // through the `String` entry above.
            @ErrorRaiserComponent.String: RaiseFrom,
            @ErrorRaiserComponent.ParseIntError: DebugError,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { PortParserComponent } }
    }

    #[test]
    fn one_capability_two_error_routes() {
        assert_eq!(App.parse_port("8080").unwrap(), 8080);

        // Raised as a `String`, converted straight through.
        assert_eq!(
            App.parse_port("70000").unwrap_err(),
            "port 70000 out of range"
        );

        // Raised as a `ParseIntError`, formatted and then routed through the `String` entry.
        assert!(App.parse_port("http").unwrap_err().contains("ParseIntError"));
    }
}

/// ## The same providers under a different error type
///
/// Nothing about `ParsePortFromStr` changes. The context names a different type and different
/// strategies, and every provider in it follows.
pub mod the_same_providers_under_a_different_error_type {
    use core::fmt::Debug;
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::DebugError;
    use cgp::prelude::*;

    use super::code_that_fails_without_knowing::{ParsePortFromStr, PortParserComponent};

    #[derive(Debug)]
    pub struct AppError {
        pub message: String,
    }

    impl From<String> for AppError {
        fn from(message: String) -> Self {
            AppError { message }
        }
    }

    pub struct StrictApp;

    delegate_components! {
        StrictApp {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<AppError>,
            PortParserComponent: ParsePortFromStr,

            @ErrorRaiserComponent.String: cgp::extra::error::RaiseFrom,
            @ErrorRaiserComponent.ParseIntError: DebugError,
        }
    }

    mod check_strict_app {
        use super::*;
        check_components! { StrictApp { PortParserComponent } }
    }

    #[test]
    fn the_provider_did_not_change() {
        use super::code_that_fails_without_knowing::CanParsePort;

        let error: AppError = StrictApp.parse_port("70000").unwrap_err();
        assert_eq!(error.message, "port 70000 out of range");
    }
}

/// ## An application's own error capability
///
/// Nothing about the pattern is confined to CGP's built-in components. A service that wants every
/// failure to carry a status code declares its own, and wires it the same way.
pub mod an_applications_own_error_capability {
    use core::fmt::Display;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    pub struct ErrUnauthorized;
    pub struct ErrNotFound;

    #[derive(Debug, PartialEq)]
    pub struct HttpError {
        pub status: u16,
        pub message: String,
    }

    #[cgp_component(HttpErrorRaiser)]
    #[use_type(HasErrorType.Error)]
    pub trait CanRaiseHttpError<Code, Detail> {
        fn raise_http_error(code: Code, detail: Detail) -> Error;
    }

    #[cgp_impl(new UnauthorizedError)]
    #[use_type(HasErrorType.{Error = HttpError})]
    impl<Detail> HttpErrorRaiser<ErrUnauthorized, Detail>
    where
        Detail: Display,
    {
        fn raise_http_error(_code: ErrUnauthorized, detail: Detail) -> Error {
            HttpError {
                status: 401,
                message: detail.to_string(),
            }
        }
    }

    #[cgp_impl(new NotFoundError)]
    #[use_type(HasErrorType.{Error = HttpError})]
    impl<Detail> HttpErrorRaiser<ErrNotFound, Detail>
    where
        Detail: Display,
    {
        fn raise_http_error(_code: ErrNotFound, detail: Detail) -> Error {
            HttpError {
                status: 404,
                message: detail.to_string(),
            }
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open HttpErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<HttpError>,

            <Detail> @HttpErrorRaiserComponent.ErrUnauthorized.Detail: UnauthorizedError,
            <Detail> @HttpErrorRaiserComponent.ErrNotFound.Detail: NotFoundError,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                HttpErrorRaiserComponent: [
                    (ErrUnauthorized, &'static str),
                    (ErrNotFound, &'static str),
                ],
            }
        }
    }

    #[test]
    fn a_handler_never_names_the_status_code() {
        let error = <App as CanRaiseHttpError<ErrUnauthorized, &str>>::raise_http_error(
            ErrUnauthorized,
            "you must first login",
        );
        assert_eq!(
            error,
            HttpError {
                status: 401,
                message: "you must first login".to_owned(),
            }
        );
    }
}
