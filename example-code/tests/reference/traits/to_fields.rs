//! Code from `docs/reference/traits/shape/to_fields.md` — `ToFields`.
//!
//! Pins the Examples program: a value taken apart into its shape by a generic function, then read
//! from the product and rebuilt.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(Clone, Debug, Eq, PartialEq, HasFields)]
    pub struct Config {
        pub host: String,
        pub port: u16,
    }

    pub fn shape_of<T>(value: T) -> T::Fields
    where
        T: ToFields,
    {
        value.to_fields()
    }

    pub fn demo() {
        let config = Config {
            host: "localhost".to_owned(),
            port: 8080,
        };

        let fields = shape_of(config.clone());
        assert_eq!(fields.1 .0.value, 8080);

        assert_eq!(Config::from_fields(fields), config);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
