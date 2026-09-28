//! Code from `docs/reference/providers/use_type.md` — *`UseType` (provider)*.
//!
//! Pins that wiring an abstract-type component to `UseType<f64>` sets the context's abstract type to
//! `f64`, read back through generic code, and that the check enforces the `Copy` bound. The unchecked
//! `UseType<String>` from Common Mistakes is a trybuild fixture; the `WithType` alias is covered by
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

    pub fn zero<Context>() -> Context::Scalar
    where
        Context: HasScalarType,
        Context::Scalar: Default,
    {
        Default::default()
    }

    pub fn demo() {
        let scalar: f64 = zero::<App>();
        assert_eq!(scalar, 0.0);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## Under the hood
///
/// The abstract type whose generated `UseType` impl the page lists, for `cargo cgp expand`.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Copy;
    }
}
