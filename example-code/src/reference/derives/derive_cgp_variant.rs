//! Code from `docs/reference/derives/derive_cgp_variant.md` — *`#[derive(CgpVariant)]`*.
//!
//! The page's worked example, run, plus the shape it rejects. Written with `CgpVariant` rather than
//! the umbrella, which is the page's own claim that the two emit the same output on an enum.

/// ## Usage
///
/// The page's rejected snippet: a struct-style variant has no single payload type, so the derive
/// fails. Every variant must carry exactly one unnamed payload, with no per-variant opt-out.
///
/// ```compile_fail
/// use cgp::prelude::*;
///
/// #[derive(CgpVariant)]
/// pub enum Shape {
///     Circle { radius: f64 },
/// }
/// ```
pub mod rejected_struct_style_variant {}

/// ## Examples
///
/// `area`, an extraction chain closed with no wildcard arm. The page names `Circle` and `Rectangle`
/// without declaring them, so they are declared here.
pub mod examples {
    use cgp::core::field::traits::FinalizeExtractResult;
    use cgp::prelude::*;

    pub struct Circle {
        pub radius: f64,
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[derive(CgpVariant)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    pub fn area(shape: Shape) -> f64 {
        match shape
            .to_extractor()
            .extract_field(PhantomData::<Symbol!("Circle")>)
        {
            Ok(circle) => core::f64::consts::PI * circle.radius * circle.radius,
            Err(remainder) => {
                let rect = remainder
                    .extract_field(PhantomData::<Symbol!("Rectangle")>)
                    .finalize_extract_result();
                rect.width * rect.height
            }
        }
    }

    #[test]
    fn test_area_of_each_variant() {
        assert_eq!(
            area(Shape::Circle(Circle { radius: 1.0 })),
            core::f64::consts::PI
        );

        assert_eq!(
            area(Shape::Rectangle(Rectangle {
                width: 3.0,
                height: 4.0,
            })),
            12.0
        );
    }
}
