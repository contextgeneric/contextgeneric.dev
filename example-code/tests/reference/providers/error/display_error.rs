//! Code from `docs/reference/providers/error/display_error.md` — `DisplayError`.

/// ## Usage and Examples
///
/// `DisplayError` formats a `Display` source into a `String` through `to_string()` and forwards it
/// to the context's `String` route. A `String` entry must be present for it to forward to.
pub mod formatting_through_display {
    use core::num::ParseIntError;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::{DisplayError, RaiseFrom};
    use cgp::prelude::*;

    #[cgp_component(Parser)]
    #[use_type(HasErrorType.Error)]
    pub trait CanParse {
        fn parse(&self, raw: &str) -> Result<u32, Error>;
    }

    #[cgp_impl(new ParseNumber)]
    #[uses(CanRaiseError<ParseIntError>)]
    #[use_type(HasErrorType.Error)]
    impl Parser {
        fn parse(&self, raw: &str) -> Result<u32, Error> {
            let parsed = raw.parse().map_err(Self::raise_error)?;
            Ok(parsed)
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            ParserComponent: ParseNumber,

            @ErrorRaiserComponent.String: RaiseFrom,
            @ErrorRaiserComponent.ParseIntError: DisplayError,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { ParserComponent } }
    }

    #[test]
    fn a_parse_error_is_carried_as_its_display_message() {
        assert_eq!(App.parse("42").unwrap(), 42);
        // The `Display` of `ParseIntError` reads "invalid digit found in string".
        assert_eq!(App.parse("nope").unwrap_err(), "invalid digit found in string");
    }
}
