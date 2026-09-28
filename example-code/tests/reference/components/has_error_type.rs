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
