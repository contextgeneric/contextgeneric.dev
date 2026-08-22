//! Code from `docs/reference/derives/derive_extract_field.md` — *`#[derive(ExtractField)]`*.
//!
//! The page's *Under the hood* section shows the two companion enums and their impls; those are not
//! repeated here, since re-declaring them beside the derive would be a coherence conflict and
//! `cargo cgp expand` is the check on that section. What this file pins is the accepted and rejected
//! shapes, the three extractors, and the exhaustiveness claim in both directions.

/// ## Overview
///
/// The opening extraction, and the payload types the page names without declaring.
pub mod what_its_for {
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq)]
    pub struct Circle {
        pub radius: u32,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Rectangle {
        pub width: u32,
        pub height: u32,
    }

    #[derive(ExtractField)]
    #[derive(Debug, Eq, PartialEq)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    #[test]
    fn test_one_attempt_yields_or_narrows() {
        // A match: the payload comes out.
        let hit = Shape::Circle(Circle { radius: 2 })
            .to_extractor()
            .extract_field(PhantomData::<Symbol!("Circle")>);
        assert!(hit.is_ok());

        // A miss: the remainder comes back, and `Circle` is now ruled out in its type.
        let miss = Shape::Rectangle(Rectangle {
            width: 3,
            height: 4,
        })
        .to_extractor()
        .extract_field(PhantomData::<Symbol!("Circle")>);
        assert!(miss.is_err());
    }
}

/// ## Using it
///
/// The accepted enum, the payload-struct rewrite the page recommends — including for a case that
/// carries nothing — and the variantless enum the page says degenerates.
pub mod using_it {
    use cgp::prelude::*;

    #[derive(Debug, PartialEq)]
    pub struct Circle {
        pub radius: f64,
    }

    /// The page's point that even a fieldless case needs a payload struct, since a bare `Empty` is a
    /// unit variant and would be refused.
    #[derive(Debug, PartialEq)]
    pub struct Empty;

    #[derive(ExtractField)]
    #[derive(Debug, PartialEq)]
    pub enum Shape {
        Circle(Circle),
        Empty(Empty),
    }

    #[derive(ExtractField)]
    pub enum Never {}

    #[derive(ExtractField)]
    pub enum Generic<T> {
        Item(T),
    }

    #[test]
    fn test_a_payload_struct_stands_in_for_a_unit_variant() {
        // A remainder carries none of the enum's attributes, so it is neither `Debug` nor
        // `PartialEq` — the result is unwrapped rather than compared whole.
        let extracted = Shape::Empty(Empty)
            .to_extractor()
            .extract_field(PhantomData::<Symbol!("Empty")>)
            .ok();

        assert_eq!(extracted, Some(Empty));
    }

    #[test]
    fn test_generics_are_carried_onto_the_companions() {
        let extracted = Generic::Item(42_u32)
            .to_extractor()
            .extract_field(PhantomData::<Symbol!("Item")>);

        assert_eq!(extracted.ok(), Some(42));
    }
}

/// The page's rejected snippet: a struct-style variant and a unit variant both fail, because
/// extraction has to name one payload type per variant.
///
/// ```compile_fail
/// use cgp::prelude::*;
///
/// #[derive(ExtractField)]
/// pub enum Shape {
///     Circle { radius: f64 },
///     Empty,
/// }
/// ```
pub mod rejected_variant_shapes {}

/// ## Examples
///
/// The three snippets the page shows: the owned chain closed with `finalize_extract_result`, the
/// borrowed read, and the mutable modification.
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
    #[derive(Debug, PartialEq)]
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
    fn test_area_reaches_both_variants() {
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

    #[test]
    fn test_a_borrowed_read_leaves_the_value_intact() {
        let shape = Shape::Circle(Circle { radius: 2.0 });

        let radius = shape
            .extractor_ref()
            .extract_field(PhantomData::<Symbol!("Circle")>)
            .map(|circle| circle.radius)
            .ok();

        assert_eq!(radius, Some(2.0));
        assert_eq!(shape, Shape::Circle(Circle { radius: 2.0 }));
    }

    #[test]
    fn test_a_mutable_extraction_modifies_in_place() {
        let mut shape = Shape::Circle(Circle { radius: 2.0 });

        if let Ok(circle) = shape
            .extractor_mut()
            .extract_field(PhantomData::<Symbol!("Circle")>)
        {
            circle.radius = 5.0;
        }

        assert_eq!(shape, Shape::Circle(Circle { radius: 5.0 }));
    }
}

/// The page's claim that finalizing before every variant has been tried does not compile, because the
/// remainder is still inhabited.
///
/// ```compile_fail
/// use cgp::core::field::traits::FinalizeExtractResult;
/// use cgp::prelude::*;
///
/// pub struct Circle { pub radius: f64 }
/// pub struct Rectangle { pub width: f64, pub height: f64 }
///
/// #[derive(ExtractField)]
/// pub enum Shape {
///     Circle(Circle),
///     Rectangle(Rectangle),
/// }
///
/// let shape = Shape::Circle(Circle { radius: 1.0 });
///
/// let _ = shape
///     .to_extractor()
///     .extract_field(PhantomData::<Symbol!("Circle")>)
///     .finalize_extract_result();
/// ```
pub mod rejected_premature_finalize {}
