//! Code from `docs/reference/providers/with_type.md` — *`WithType`*.
//!
//! Pins that wiring an abstract-type component to the `WithType<f64>` alias binds the context's abstract
//! type to `f64`, the same as the plain `UseType<f64>`.

/// ## Examples
pub mod examples {
    use cgp::core::types::WithType;
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Copy;
    }

    pub struct App;

    delegate_components! {
        App {
            ScalarTypeProviderComponent: WithType<f64>,
        }
    }

    check_components! {
        App {
            ScalarTypeProviderComponent,
        }
    }
}
