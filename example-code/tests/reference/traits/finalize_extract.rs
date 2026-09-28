//! Code from `docs/reference/traits/variant/finalize_extract.md` — `FinalizeExtract`.
//!
//! Pins the Examples program: `finalize_extract` called directly on a remainder unwrapped by a
//! nested `match`. Finalizing before every variant is tried is a trybuild fixture.

/// ## Examples
pub mod examples {
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

    #[derive(ExtractField)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    pub fn describe(shape: Shape) -> String {
        match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
            Ok(circle) => format!("circle of radius {}", circle.radius),
            Err(remainder) => match remainder.extract_field(PhantomData::<Symbol!("Rectangle")>) {
                Ok(rect) => format!("{} by {} rectangle", rect.width, rect.height),
                // Uninhabited: this arm can never run.
                Err(remainder) => remainder.finalize_extract(),
            },
        }
    }

    pub fn demo() {
        assert_eq!(describe(Shape::Circle(Circle { radius: 2.0 })), "circle of radius 2");
        assert_eq!(
            describe(Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 })),
            "3 by 4 rectangle"
        );
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
