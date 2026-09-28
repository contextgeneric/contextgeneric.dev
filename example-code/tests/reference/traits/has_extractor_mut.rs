//! Code from `docs/reference/traits/variant/has_extractor_mut.md` — `HasExtractorMut`.
//!
//! Pins the Examples program: a full mutable chain that scales whichever variant is present, the
//! writes landing in the original value.

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

    pub fn scale(shape: &mut Shape, factor: f64) {
        match shape.extractor_mut().extract_field(PhantomData::<Symbol!("Circle")>) {
            Ok(circle) => circle.radius *= factor,
            Err(remainder) => {
                let rect = remainder
                    .extract_field(PhantomData::<Symbol!("Rectangle")>)
                    .finalize_extract_result();
                rect.width *= factor;
                rect.height *= factor;
            }
        }
    }

    pub fn demo() {
        let mut shape = Shape::Circle(Circle { radius: 1.0 });
        scale(&mut shape, 5.0);

        // `shape` now holds the updated `Circle`.
        assert_eq!(shape, Shape::Circle(Circle { radius: 5.0 }));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
