//! Code from `docs/reference/traits/type-level/transform_map.md` — `TransformMap`.
//!
//! Pins the Examples program: a transform marker with one impl per source state, applied to a
//! builder whose fields are present and absent. A transform missing a source state is a trybuild
//! fixture.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::IsOptional;
    use cgp::core::field::traits::{TransformMap, TransformMapFields};
    use cgp::prelude::*;

    pub struct FillDefaults;

    impl<T> TransformMap<IsPresent, IsPresent, T> for FillDefaults {
        fn transform_mapped(value: T) -> T {
            value
        }
    }

    impl<T: Default> TransformMap<IsNothing, IsPresent, T> for FillDefaults {
        fn transform_mapped(_value: ()) -> T {
            T::default()
        }
    }

    impl<T: Default> TransformMap<IsOptional, IsPresent, T> for FillDefaults {
        fn transform_mapped(value: Option<T>) -> T {
            value.unwrap_or_default()
        }
    }

    #[derive(Debug, PartialEq, CgpData)]
    pub struct Config {
        pub port: u16,
        pub verbose: bool,
    }

    pub fn with_defaults() -> Config {
        let partial = Config::builder().build_field(PhantomData::<Symbol!("port")>, 8080);

        TransformMapFields::<FillDefaults, IsPresent>::transform_map_fields(partial)
            .finalize_build()
    }

    pub fn demo() {
        assert_eq!(
            with_defaults(),
            Config {
                port: 8080,
                verbose: false,
            }
        );
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
