//! Code from `docs/reference/traits/variant/has_extractor.md` — `HasExtractor`.
//!
//! Pins the Examples program: `to_extractor` starting a chain, and `from_extractor` rebuilding the
//! enum from an extractor that was not narrowed.

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

    pub fn area(shape: Shape) -> f64 {
        match shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
            Ok(circle) => core::f64::consts::PI * circle.radius * circle.radius,
            Err(remainder) => {
                let rect = remainder
                    .extract_field(PhantomData::<Symbol!("Rectangle")>)
                    .finalize_extract_result();
                rect.width * rect.height
            }
        }
    }

    // Any extractable enum survives the round trip unchanged.
    pub fn round_trip<E: HasExtractor>(value: E) -> E {
        E::from_extractor(value.to_extractor())
    }

    pub fn demo() {
        assert_eq!(area(Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 })), 12.0);

        let shape = Shape::Circle(Circle { radius: 2.0 });
        assert_eq!(round_trip(shape), Shape::Circle(Circle { radius: 2.0 }));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
