//! Code from `docs/reference/components/has_type.md` — `HasType`.
//!
//! Pins that wiring the built-in `TypeProviderComponent` to `UseType<f64>` makes a context implement
//! `HasType<Tag>` with `Type = f64` for any tag, and that generic code reads it back through `TypeOf`.

/// ## Examples
pub mod examples {
    use cgp::core::types::{TypeOf, TypeProviderComponent};
    use cgp::prelude::*;

    pub struct ScalarTag;

    pub struct App;

    delegate_components! {
        App {
            TypeProviderComponent: UseType<f64>,
        }
    }

    mod check_app {
        use super::*;

        check_components! {
            App {
                TypeProviderComponent: ScalarTag,
            }
        }
    }

    pub fn zero<Context>() -> TypeOf<Context, ScalarTag>
    where
        Context: HasType<ScalarTag>,
        TypeOf<Context, ScalarTag>: Default,
    {
        Default::default()
    }

    #[test]
    fn app_resolves_any_tag_to_f64() {
        // `App` wires `TypeProviderComponent` to `UseType<f64>`, so `HasType<ScalarTag>` resolves to
        // `f64` for every tag, `ScalarTag` included.
        assert_eq!(zero::<App>(), 0.0);
    }
}
