//! Code from `docs/reference/components/has_error_type.md` — `HasErrorType`.
//!
//! Pins that a context wires its abstract error to a concrete type through `UseType`, and that an
//! error-aware component names the error as the bare `Error` via `#[use_type]`.

/// ## Usage
///
/// The direct impl, with `anyhow::Error` as the context's error type.
pub mod usage {
    use cgp::prelude::*;

    pub struct App;

    impl HasErrorType for App {
        type Error = anyhow::Error;
    }

    pub fn fail() -> Result<(), <App as HasErrorType>::Error> {
        Err(anyhow::anyhow!("failed"))
    }

    // `ErrorOnly<E>`: a context with an error type and nothing else.
    use cgp::core::error::{ErrorOf, ErrorOnly};

    pub fn parse_port<Context: HasErrorType<Error = String>>(
        _context: &Context,
        raw: &str,
    ) -> Result<u16, ErrorOf<Context>> {
        raw.parse().map_err(|_| format!("bad port: {raw}"))
    }

    #[test]
    fn test_error_only_supplies_an_error_type() {
        let context = ErrorOnly::<String>::default();
        assert_eq!(parse_port(&context, "80"), Ok(80));
        assert_eq!(parse_port(&context, "x"), Err("bad port: x".to_owned()));
        assert_eq!(core::mem::size_of::<ErrorOnly<String>>(), 0);
    }
}

/// ## Examples
pub mod examples {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    #[cgp_component(Validator)]
    #[use_type(HasErrorType.Error)]
    pub trait CanValidate {
        fn validate(&self) -> Result<(), Error>;
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
        }
    }

    check_components! {
        App {
            ErrorTypeProviderComponent,
        }
    }
}
