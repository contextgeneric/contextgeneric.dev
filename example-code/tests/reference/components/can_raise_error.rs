//! Code from `docs/reference/components/can_raise_error.md` — `CanRaiseError`.
//!
//! The page's Definition carries `#[track_caller]`, which the `cgp` revision this crate pins predates;
//! the caller-location claims were checked against the local `cgp` checkout instead. The rejected
//! concrete-context call is a trybuild fixture.

/// ## Usage
///
/// The `open` table routing each source error to a provider. `ParseError` stands for any source
/// error the application meets.
pub mod usage {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::{DebugError, RaiseFrom};
    use cgp::prelude::*;

    #[derive(Debug)]
    pub struct ParseError;

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            @ErrorRaiserComponent.String: RaiseFrom,
            @ErrorRaiserComponent.ParseError: DebugError,
        }
    }

    check_components! {
        App {
            ErrorRaiserComponent: [String, ParseError],
        }
    }

    /// Generic code calls the associated function on the context type.
    pub fn raise_parse<Context: CanRaiseError<ParseError>>() -> Context::Error {
        Context::raise_error(ParseError)
    }

    #[test]
    fn each_source_error_reaches_its_provider() {
        assert_eq!(<App as CanRaiseError<String>>::raise_error("boom".to_owned()), "boom");
        assert_eq!(raise_parse::<App>(), "ParseError");
    }
}

/// ## Examples
///
/// `LoadOrFail`, wired on a context whose error type is `String`.
pub mod examples {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self, path: &str) -> Result<String, Error>;
    }

    #[cgp_impl(new LoadOrFail)]
    #[uses(CanRaiseError<String>)]
    #[use_type(HasErrorType.Error)]
    impl Loader {
        fn load(&self, path: &str) -> Result<String, Error> {
            if path.is_empty() {
                return Err(Self::raise_error("empty path".to_owned()));
            }
            Ok(format!("contents of {path}"))
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: RaiseFrom,
            LoaderComponent: LoadOrFail,
        }
    }

    check_components! {
        App {
            LoaderComponent,
        }
    }

    #[test]
    fn load_raises_into_the_contexts_error() {
        assert_eq!(App.load("a.txt"), Ok("contents of a.txt".to_owned()));
        assert_eq!(App.load(""), Err("empty path".to_owned()));
    }
}
