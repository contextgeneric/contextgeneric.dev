//! Code from `docs/reference/providers/error/debug_error.md` — `DebugError`.
//!
//! Pins the Examples program: a `ParseIntError` is formatted with `Debug` and forwarded to the
//! `String` entry, which `RaiseFrom` converts. The `String` key wired to `DebugError` itself,
//! from Common Mistakes, is a trybuild fixture.

/// ## Examples
pub mod examples {
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
    #[uses(CanRaiseError<ParseIntError>)]
    #[use_type(HasErrorType.Error)]
    impl PortParser {
        fn parse_port(&self, raw: &str) -> Result<u16, Error> {
            raw.parse().map_err(Self::raise_error)
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

    check_components! {
        App {
            PortParserComponent,
        }
    }

    pub fn demo() {
        assert_eq!(App.parse_port("8080"), Ok(8080));
        assert_eq!(
            App.parse_port("http"),
            Err("ParseIntError { kind: InvalidDigit }".to_owned())
        );
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
