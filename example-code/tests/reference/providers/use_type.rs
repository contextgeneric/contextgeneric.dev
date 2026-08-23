//! Code from `docs/reference/providers/use_type.md` — *`UseType` (provider)*.
//!
//! Pins that wiring an abstract-type component to `UseType<f64>` sets the context's abstract type to
//! `f64` and enforces the `Copy` bound at the wiring site. The `WithType` alias is covered by
//! `with_type.rs`.

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
