//! Code from `docs/reference/providers/error/raise_from.md` — `RaiseFrom`.
//!
//! Pins the Usage wiring and the Examples program, in which one `ErrorRaiserComponent: RaiseFrom`
//! entry raises two different source types through the error's `From` impls. The source with no
//! `From` impl, from Common Mistakes, is a trybuild fixture.

/// ## Examples
pub mod examples {
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    #[derive(Debug, PartialEq)]
    pub enum AppError {
        Parse(ParseIntError),
        Message(String),
    }

    impl From<ParseIntError> for AppError {
        fn from(e: ParseIntError) -> Self {
            AppError::Parse(e)
        }
    }

    impl From<String> for AppError {
        fn from(message: String) -> Self {
            AppError::Message(message)
        }
    }

    #[cgp_component(PortParser)]
    #[use_type(HasErrorType.Error)]
    pub trait CanParsePort {
        fn parse_port(&self, raw: &str) -> Result<u16, Error>;
    }

    #[cgp_impl(new ParsePort)]
    #[uses(CanRaiseError<ParseIntError>, CanRaiseError<String>)]
    #[use_type(HasErrorType.Error)]
    impl PortParser {
        fn parse_port(&self, raw: &str) -> Result<u16, Error> {
            let port: u16 = raw.parse().map_err(Self::raise_error)?;

            if port == 0 {
                return Err(Self::raise_error("port 0 is reserved".to_owned()));
            }

            Ok(port)
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<AppError>,
            ErrorRaiserComponent: RaiseFrom,
            PortParserComponent: ParsePort,
        }
    }

    check_components! {
        App {
            PortParserComponent,
        }
    }

    pub fn demo() {
        assert_eq!(App.parse_port("8080"), Ok(8080));
        assert_eq!(
            App.parse_port("0"),
            Err(AppError::Message("port 0 is reserved".to_owned()))
        );
        assert!(matches!(App.parse_port("http"), Err(AppError::Parse(_))));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
