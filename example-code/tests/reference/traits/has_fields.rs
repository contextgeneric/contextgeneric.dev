//! Code from `docs/reference/traits/shape/has_fields.md` — `HasFields`.
//!
//! Pins the Examples program: the shapes a struct, an enum, and a one-field tuple struct derive,
//! each asserted as a type equality through a generic bound.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasField, HasFields)]
    pub struct Config {
        pub host: String,
        pub port: u16,
    }

    pub struct Circle {
        pub radius: f64,
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[derive(HasFields)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    #[derive(HasFields)]
    pub struct UserId(pub u64);

    pub fn assert_shape<T, Fields>()
    where
        T: HasFields<Fields = Fields>,
    {
    }

    pub fn demo() {
        assert_shape::<Config, Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>]>(
        );
        assert_shape::<
            Shape,
            Sum![Field<Symbol!("Circle"), Circle>, Field<Symbol!("Rectangle"), Rectangle>],
        >();
        assert_shape::<UserId, u64>();
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
