//! Code from `docs/reference/traits/variant/extract_field.md` — `ExtractField`.
//!
//! Pins the Examples program: a two-variant chain closed by `finalize_extract_result`, with the
//! extractions written in either order. Extracting a variant twice and finalizing early are
//! trybuild fixtures.

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
                // `remainder` can no longer be a `Circle`.
                let rect = remainder
                    .extract_field(PhantomData::<Symbol!("Rectangle")>)
                    .finalize_extract_result();
                rect.width * rect.height
            }
        }
    }

    // The same chain with the extractions the other way round.
    pub fn perimeter(shape: Shape) -> f64 {
        match shape.to_extractor().extract_field(PhantomData::<Symbol!("Rectangle")>) {
            Ok(rect) => 2.0 * (rect.width + rect.height),
            Err(remainder) => {
                let circle = remainder
                    .extract_field(PhantomData::<Symbol!("Circle")>)
                    .finalize_extract_result();
                2.0 * core::f64::consts::PI * circle.radius
            }
        }
    }

    pub fn demo() {
        assert_eq!(area(Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 })), 12.0);
        assert_eq!(perimeter(Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 })), 14.0);
        assert!(area(Shape::Circle(Circle { radius: 1.0 })) > 3.14);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
