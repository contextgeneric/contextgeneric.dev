//! Code from `docs/reference/traits/variant/finalize_extract_result.md` — `FinalizeExtractResult`.
//!
//! Pins the Examples program: a chain closed by `finalize_extract_result`. Calling it one step
//! early, and calling it without the import, are trybuild fixtures.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::FinalizeExtractResult;
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

    pub fn area(shape: Shape) -> f64 {
        match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
            Ok(circle) => core::f64::consts::PI * circle.radius * circle.radius,
            Err(remainder) => {
                let rect = remainder
                    .extract_field(PhantomData::<Symbol!("Rectangle")>)
                    .finalize_extract_result(); // no variants left, so this cannot fail
                rect.width * rect.height
            }
        }
    }

    pub fn demo() {
        assert_eq!(area(Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 })), 12.0);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
