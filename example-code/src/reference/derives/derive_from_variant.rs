//! Code from `docs/reference/derives/derive_from_variant.md` — *`#[derive(FromVariant)]`*.
//!
//! The page's *Under the hood* section shows the per-variant impls; those are not repeated here, since
//! re-declaring them beside the derive would be a coherence conflict and `cargo cgp expand` is the
//! check on that section. What this file pins is that the tag-generic constructor works, and the two
//! shapes the page says are rejected.

/// ## Overview
///
/// The opening claim: `from_variant` with a tag is the same construction as naming the variant.
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

    #[derive(FromVariant)]
    #[derive(Debug, Eq, PartialEq)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    #[test]
    fn test_the_tagged_call_equals_the_named_one() {
        let by_tag = Shape::from_variant(PhantomData::<Symbol!("Circle")>, Circle { radius: 2 });

        assert_eq!(by_tag, Shape::Circle(Circle { radius: 2 }));
    }
}

/// ## Using it
///
/// The accepted enum, a generic one, and the variantless enum the page says produces no impls.
pub mod using_it {
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq)]
    pub struct Circle {
        pub radius: u32,
    }

    #[derive(FromVariant)]
    #[derive(Debug, Eq, PartialEq)]
    pub enum Shape {
        Circle(Circle),
    }

    #[derive(FromVariant)]
    #[derive(Debug, Eq, PartialEq)]
    pub enum Generic<T> {
        Item(T),
    }

    #[derive(FromVariant)]
    pub enum Never {}

    #[test]
    fn test_generics_are_carried_onto_every_impl() {
        let value = Generic::from_variant(PhantomData::<Symbol!("Item")>, 42_u32);

        assert_eq!(value, Generic::Item(42));
    }
}

/// The page's rejected snippet: a struct-style variant has no single payload type for the constructor
/// to take.
///
/// ```compile_fail
/// use cgp::prelude::*;
///
/// #[derive(FromVariant)]
/// pub enum Shape {
///     Circle { radius: f64 },
/// }
/// ```
pub mod rejected_struct_style_variant {}

/// ## Examples
///
/// The `wrap` function the page shows: one function that builds either variant, staying generic over
/// the tag and naming its payload type through the trait.
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

    #[derive(FromVariant)]
    #[derive(Debug, PartialEq)]
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

    #[test]
    fn test_one_function_builds_either_variant() {
        let circle = wrap(PhantomData::<Symbol!("Circle")>, Circle { radius: 2.0 });
        assert_eq!(circle, Shape::Circle(Circle { radius: 2.0 }));

        let rect = wrap(
            PhantomData::<Symbol!("Rectangle")>,
            Rectangle {
                width: 3.0,
                height: 4.0,
            },
        );
        assert_eq!(
            rect,
            Shape::Rectangle(Rectangle {
                width: 3.0,
                height: 4.0,
            })
        );
    }
}

/// ## Examples
///
/// The upcast the page closes on: construct into a small local enum, then widen. The page shows one
/// line from the expression-interpreter scenario, so the surrounding types are declared here.
pub mod examples_upcast {
    use cgp::prelude::*;

    #[derive(Debug, PartialEq)]
    pub struct Ident(pub String);

    #[derive(Debug, PartialEq)]
    pub struct Literal(pub u64);

    /// The narrow enum an implementation constructs into.
    #[derive(CgpData)]
    #[derive(Debug, PartialEq)]
    pub enum LispSubExpr {
        Ident(Ident),
    }

    /// The full enum it is widened into.
    #[derive(CgpData)]
    #[derive(Debug, PartialEq)]
    pub enum LispExpr {
        Ident(Ident),
        Literal(Literal),
    }

    #[test]
    fn test_upcast_from_the_narrow_enum() {
        use cgp::core::field::impls::CanUpcast;

        let ident = LispSubExpr::Ident(Ident("+".to_owned())).upcast(PhantomData::<LispExpr>);

        assert_eq!(ident, LispExpr::Ident(Ident("+".to_owned())));
    }
}
