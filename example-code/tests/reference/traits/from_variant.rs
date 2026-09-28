//! Code from `docs/reference/traits/variant/from_variant.md` — `FromVariant`.
//!
//! Pins the Examples program: one generic `wrap` building either variant, and a narrow enum
//! upcast into `Shape`, which needs only `FromVariant` on the target. Leaving the tag to inference
//! on a two-variant enum is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::CanUpcast;
    use cgp::prelude::*;

    #[derive(Debug, PartialEq)]
    pub struct Circle {
        pub radius: f64,
    }

    #[derive(Debug, PartialEq)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[derive(Debug, PartialEq, FromVariant)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    pub fn wrap<Tag>(tag: PhantomData<Tag>, value: <Shape as FromVariant<Tag>>::Value) -> Shape
    where
        Shape: FromVariant<Tag>,
    {
        Shape::from_variant(tag, value)
    }

    // A routine that only ever produces circles works in a one-variant enum.
    #[derive(HasFields, ExtractField)]
    pub enum RoundShape {
        Circle(Circle),
    }

    pub fn unit_circle() -> Shape {
        RoundShape::Circle(Circle { radius: 1.0 }).upcast(PhantomData::<Shape>)
    }

    pub fn demo() {
        let circle = wrap(PhantomData::<Symbol!("Circle")>, Circle { radius: 2.0 });
        assert_eq!(circle, Shape::Circle(Circle { radius: 2.0 }));

        let rect = wrap(
            PhantomData::<Symbol!("Rectangle")>,
            Rectangle { width: 3.0, height: 4.0 },
        );
        assert_eq!(rect, Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 }));

        assert_eq!(unit_circle(), Shape::Circle(Circle { radius: 1.0 }));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
