//! Code from `docs/reference/components/has_runtime_type.md` — `HasRuntimeType`.
//!
//! Pins a context that fixes its runtime type with `UseType`, and the generic function that names the
//! runtime type without touching a runtime value.

/// ## Examples
pub mod examples {
    use cgp::extra::runtime::{HasRuntimeType, RuntimeOf, RuntimeTypeProviderComponent};
    use cgp::prelude::*;

    pub struct TokioRuntime;

    pub struct App;

    delegate_components! {
        App {
            RuntimeTypeProviderComponent: UseType<TokioRuntime>,
        }
    }

    mod check_app {
        use super::*;

        check_components! {
            App {
                RuntimeTypeProviderComponent,
            }
        }
    }

    pub fn describe<Context>() -> &'static str
    where
        Context: HasRuntimeType,
        RuntimeOf<Context>: Default,
    {
        "the context has a default-constructible runtime type"
    }
}
