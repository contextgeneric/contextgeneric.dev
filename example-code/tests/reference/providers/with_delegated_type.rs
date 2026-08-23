//! Code from `docs/reference/providers/with_delegated_type.md` — *`WithDelegatedType`*.
//!
//! `WithDelegatedType` (`WithProvider<UseDelegatedType<..>>`) is the form that wires the foundational
//! `UseDelegatedType` provider. This pins that one entry answers two abstract-type components, each
//! resolving to the concrete type held in the `AppTypes` table.

/// ## Examples
pub mod examples {
    use cgp::core::types::WithDelegatedType;
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar;
    }

    #[cgp_type]
    pub trait HasIndexType {
        type Index;
    }

    pub struct App;
    pub struct AppTypes;

    delegate_components! {
        AppTypes {
            ScalarTypeProviderComponent: f64,
            IndexTypeProviderComponent: usize,
        }
    }

    delegate_components! {
        App {
            [
                ScalarTypeProviderComponent,
                IndexTypeProviderComponent,
            ]: WithDelegatedType<AppTypes>,
        }
    }

    check_components! {
        App {
            ScalarTypeProviderComponent,
            IndexTypeProviderComponent,
        }
    }
}
