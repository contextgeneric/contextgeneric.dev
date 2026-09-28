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

    /// The Usage section's `open` table, giving two tags two types.
    pub struct NameTag;

    pub struct TwoTypes;

    delegate_components! {
        TwoTypes {
            open TypeProviderComponent;

            @TypeProviderComponent.ScalarTag: UseType<f64>,
            @TypeProviderComponent.NameTag: UseType<String>,
        }
    }

    check_components! {
        TwoTypes {
            TypeProviderComponent: [ScalarTag, NameTag],
        }
    }

    #[test]
    fn open_gives_each_tag_its_own_type() {
        let _: PhantomData<TypeOf<TwoTypes, NameTag>> = PhantomData::<String>;
        assert_eq!(zero::<TwoTypes>(), 0.0);
    }

    #[test]
    fn app_resolves_any_tag_to_f64() {
        // `App` wires `TypeProviderComponent` to `UseType<f64>`, so `HasType<ScalarTag>` resolves to
        // `f64` for every tag, `ScalarTag` included.
        assert_eq!(zero::<App>(), 0.0);
    }
}
