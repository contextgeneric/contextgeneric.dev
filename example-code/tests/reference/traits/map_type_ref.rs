//! Code from `docs/reference/traits/type-level/map_type_ref.md` — `MapTypeRef`.
//!
//! Pins the Examples program: what each standard marker stores at a lifetime, checked as type
//! equalities, and the two borrowed extractors whose outer marker is `IsRef` or `IsMut`.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::IsOwned;
    use cgp::prelude::*;

    pub fn by_ref<'a>(value: <IsRef as MapTypeRef>::Map<'a, String>) -> &'a String {
        value
    }

    pub fn by_mut<'a>(value: <IsMut as MapTypeRef>::Map<'a, String>) -> &'a mut String {
        value
    }

    pub fn owned<'a>(value: <IsOwned as MapTypeRef>::Map<'a, String>) -> String {
        value
    }

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

    pub fn demo() {
        let mut shape = Shape::Circle(Circle { radius: 1.0 });

        // The outer marker is `IsRef`, so the payload comes out as `&Circle`.
        let radius = shape
            .extractor_ref()
            .extract_field(PhantomData::<Symbol!("Circle")>)
            .map(|circle| circle.radius)
            .ok();
        assert_eq!(radius, Some(1.0));

        // The outer marker is `IsMut`, so the payload comes out as `&mut Circle`.
        if let Ok(circle) = shape
            .extractor_mut()
            .extract_field(PhantomData::<Symbol!("Circle")>)
        {
            circle.radius = 5.0;
        }
        assert_eq!(shape, Shape::Circle(Circle { radius: 5.0 }));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
