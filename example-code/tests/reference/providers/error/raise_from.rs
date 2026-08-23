//! Code from `docs/reference/providers/error/raise_from.md` — `RaiseFrom`.

/// ## Usage and Examples
///
/// The page shows `RaiseFrom` wired on its own, then dispatched per source type beside a formatter.
/// One program covers both: a `String` is converted straight through `RaiseFrom` (the abstract error
/// *is* `String`, and `String: From<String>`), while a `ParseIntError` is formatted by `DebugError`
/// and forwarded back through the same `String` route.
///
/// The page's illustrative `ParseError` key is a real `ParseIntError` here, so the parse can run.
pub mod raising_through_from {
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::{DebugError, RaiseFrom};
    use cgp::prelude::*;

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
            let parsed: u32 = raw.parse().map_err(Self::raise_error)?;

            if parsed > u16::MAX as u32 {
                // Raised as a `String`, converted straight through by `RaiseFrom`.
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
            PortParserComponent: ParsePort,

            @ErrorRaiserComponent.String: RaiseFrom,
            @ErrorRaiserComponent.ParseIntError: DebugError,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { PortParserComponent } }
    }

    #[test]
    fn a_string_is_converted_and_a_parse_error_is_formatted() {
        assert_eq!(App.parse_port("8080").unwrap(), 8080);
        assert_eq!(App.parse_port("70000").unwrap_err(), "port 70000 out of range");
        assert!(App.parse_port("nope").unwrap_err().contains("ParseIntError"));
    }
}
