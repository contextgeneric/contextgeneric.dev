//! Code from `docs/reference/components/has_error_type.md` — `HasErrorType`.
//!
//! Pins that a context wires its abstract error to a concrete type through `UseType`, and that an
//! error-aware component names the error as the bare `Error` via `#[use_type]`. The page's direct-impl
//! form uses `anyhow::Error`, which this crate does not depend on, so the wired `String` form stands
//! in for it.

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

    mod check_app {
        use super::*;

        check_components! {
            App {
                ErrorTypeProviderComponent,
            }
        }
    }
}
