//! Code from `docs/reference/traits/variant/has_extractor_ref.md` — `HasExtractorRef`.
//!
//! Pins the Examples program: a single borrowed attempt, and a full borrowed chain closed by
//! `finalize_extract_result`, both leaving the value usable.

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

    #[derive(Debug, PartialEq, ExtractField)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    pub fn radius(shape: &Shape) -> Option<f64> {
        shape
            .extractor_ref()
            .extract_field(PhantomData::<Symbol!("Circle")>)
            .map(|circle| circle.radius)
            .ok()
    }

    pub fn area(shape: &Shape) -> f64 {
        match shape.extractor_ref().extract_field(PhantomData::<Symbol!("Circle")>) {
            Ok(circle) => core::f64::consts::PI * circle.radius * circle.radius,
            Err(remainder) => {
                let rect = remainder
                    .extract_field(PhantomData::<Symbol!("Rectangle")>)
                    .finalize_extract_result();
                rect.width * rect.height
            }
        }
    }

    pub fn demo() {
        let shape = Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 });
        assert_eq!(radius(&shape), None);
        assert_eq!(area(&shape), 12.0);

        // `shape` is still usable here.
        assert_eq!(shape, Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 }));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
