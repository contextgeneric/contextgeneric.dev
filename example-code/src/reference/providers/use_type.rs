//! Code from `docs/reference/providers/use_type.md` — *`UseType` (provider)*.
//!
//! Pins that wiring an abstract-type component to `UseType<f64>` (or the `WithType` alias) sets the
//! context's abstract type to `f64` and enforces the `Copy` bound at the wiring site.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Copy;
    }

    pub struct App;

    delegate_components! {
        App {
            ScalarTypeProviderComponent: UseType<f64>,
        }
    }

    check_components! {
        App {
            ScalarTypeProviderComponent,
        }
    }
}

/// ## Examples — the `WithType` alias
pub mod examples_with_type {
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
