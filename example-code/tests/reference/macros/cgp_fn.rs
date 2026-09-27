//! Code from `docs/reference/macros/cgp_fn.md` — *`#[cgp_fn]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`.

/// ## Overview
///
/// The opening function, and a context with the two fields it reads.
pub mod overview {
    use cgp::prelude::*;

    #[cgp_fn]
    pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[test]
    fn any_struct_with_the_fields_gets_the_method() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(rect.rectangle_area(), 6.0);
    }
}

/// ## Usage
///
/// The trait-name override, the receiver-less form the page says the macro accepts, and a
/// `&mut self` function writing through a mutable implicit argument. The last two are stated in
/// prose on the page.
pub mod usage {
    use cgp::prelude::*;

    #[cgp_fn(CanCalculateRectangleArea)]
    pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }

    #[cgp_fn]
    pub fn unit_area() -> f64 {
        1.0
    }

    #[cgp_fn]
    pub fn grow(&mut self, #[implicit] width: &mut f64) {
        *width *= 2.0;
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[test]
    fn the_forms_behave_as_described() {
        let mut rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(CanCalculateRectangleArea::rectangle_area(&rect), 6.0);
        assert_eq!(<Rectangle as UnitArea>::unit_area(), 1.0);
        rect.grow();
        assert_eq!(rect.width, 4.0);
    }
}

/// ### Generics, and where each bound lands
///
/// The `scale` function: `Scalar` lands on the trait, its `where` bound on the impl alone.
pub mod generics_and_where_each_bound_lands {
    use core::ops::Mul;

    use cgp::prelude::*;

    #[cgp_fn]
    pub fn scale<Scalar>(&self, #[implicit] factor: Scalar) -> Scalar
    where
        Scalar: Mul<Output = Scalar> + Copy,
    {
        factor * factor
    }

    #[derive(HasField)]
    pub struct Scaled {
        pub factor: f64,
    }

    /// A caller bounds on the trait alone, without repeating `Mul`.
    pub fn squared<Context: Scale<f64>>(context: &Context) -> f64 {
        context.scale()
    }

    #[test]
    fn the_caller_needs_only_the_trait() {
        assert_eq!(squared(&Scaled { factor: 3.0 }), 9.0);
    }
}

/// #### A type the caller should not name
///
/// `#[impl_generics(Name: Display)]`: the trait is not generic, and the field fixes `Name`.
pub mod a_type_the_caller_should_not_name {
    use core::fmt::Display;

    use cgp::prelude::*;

    #[cgp_fn]
    #[impl_generics(Name: Display)]
    pub fn describe(&self, #[implicit] name: &Name) -> String {
        format!("{name}")
    }

    #[derive(HasField)]
    pub struct Labeled {
        pub name: u32,
    }

    #[test]
    fn the_field_type_is_inferred() {
        assert_eq!(Labeled { name: 7 }.describe(), "7");
    }
}

/// ### Companion attributes
///
/// `#[async_trait]` beneath `#[cgp_fn]` on an `async fn`, which the page says lands on both
/// items. The page describes it without a snippet.
pub mod companion_attributes {
    use cgp::prelude::*;

    #[cgp_fn]
    #[async_trait]
    pub async fn fetch_label(&self, #[implicit] label: &str) -> String {
        label.to_owned()
    }

    #[derive(HasField)]
    pub struct Item {
        pub label: String,
    }

    #[test]
    fn the_async_method_runs() {
        let item = Item {
            label: "x".to_owned(),
        };
        assert_eq!(futures::executor::block_on(item.fetch_label()), "x");
    }
}

/// ## Examples
///
/// The page's two functions and the context that gets both without wiring.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_fn]
    pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }

    #[cgp_fn]
    #[uses(RectangleArea)]
    pub fn scaled_rectangle_area(&self, #[implicit] scale_factor: f64) -> f64 {
        self.rectangle_area() * scale_factor * scale_factor
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
        pub scale_factor: f64,
    }

    pub fn report(rect: &Rectangle) {
        println!("base area   = {}", rect.rectangle_area());
        println!("scaled area = {}", rect.scaled_rectangle_area());
    }

    #[test]
    fn both_traits_apply() {
        let rect = Rectangle {
            width: 3.0,
            height: 4.0,
            scale_factor: 2.0,
        };
        report(&rect);
        assert_eq!(rect.rectangle_area(), 12.0);
        assert_eq!(rect.scaled_rectangle_area(), 48.0);
    }
}

/// ## Under the hood
///
/// The order of the impl's `where` clause, with every contributor present, checked with
/// `cargo cgp expand --item reference::macros::cgp_fn::under_the_hood`. `HasScalarType` and
/// `HasLabel` are declared here to give `#[use_type]` and `#[uses]` something to import.
pub mod under_the_hood {
    use core::fmt::Debug;
    use core::ops::Mul;

    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Mul<Output = Self::Scalar> + Copy;
    }

    #[cgp_auto_getter]
    pub trait HasLabel {
        fn label(&self) -> &str;
    }

    #[cgp_fn]
    #[extend(Debug)]
    #[uses(HasLabel)]
    #[extend_where(Self: Sized)]
    #[use_type(HasScalarType.Scalar)]
    pub fn labeled_square<T>(&self, #[implicit] side: Scalar, tag: T) -> (String, Scalar)
    where
        T: Debug,
    {
        (format!("{}{:?}", self.label(), tag), side * side)
    }

    #[derive(Debug, HasField)]
    pub struct Square {
        pub label: String,
        pub side: f64,
    }

    delegate_components! {
        Square {
            ScalarTypeProviderComponent: UseType<f64>,
        }
    }

    #[test]
    fn every_contributor_is_satisfied() {
        let square = Square {
            label: "s".to_owned(),
            side: 3.0,
        };
        assert_eq!(square.labeled_square(1u8), ("s1".to_owned(), 9.0));
    }

    /// A private function yields a private trait.
    #[cgp_fn]
    fn private_area(&self, #[implicit] side: f64) -> f64 {
        side * side
    }

    #[test]
    fn the_private_trait_is_usable_here() {
        let square = Square {
            label: String::new(),
            side: 2.0,
        };
        assert_eq!(square.private_area(), 4.0);
    }
}
