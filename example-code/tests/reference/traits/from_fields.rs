//! Code from `docs/reference/traits/shape/from_fields.md` — `FromFields`.
//!
//! Pins the Examples program: a generic function that rebuilds a value from its shape, round-tripping
//! a struct and an enum.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(Clone, Debug, Eq, PartialEq, HasFields)]
    pub struct Config {
        pub host: String,
        pub port: u16,
    }

    #[derive(Clone, Debug, PartialEq, HasFields)]
    pub enum Shape {
        Circle(f64),
        Square(f64),
    }

    pub fn round_trip<T>(value: T) -> T
    where
        T: ToFields + FromFields,
    {
        T::from_fields(value.to_fields())
    }

    pub fn demo() {
        let config = Config {
            host: "localhost".to_owned(),
            port: 8080,
        };
        assert_eq!(round_trip(config.clone()), config);

        assert_eq!(round_trip(Shape::Square(2.0)), Shape::Square(2.0));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
