//! Code from `docs/reference/attributes/uses.md` — *`#[uses]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/attributes/`.

/// ## Overview and Usage
///
/// The import list forms: one attribute with several entries, an ordinary Rust trait, a trait
/// with type arguments, split attributes, and the wider bound forms the section lists.
pub mod usage {
    use core::fmt::Display;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    #[cgp_fn]
    pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_fn]
    #[uses(RectangleArea, CanCalculateArea)]
    pub fn both_areas(&self) -> f64 {
        self.rectangle_area() + self.area()
    }

    #[cgp_fn]
    #[uses(Display, AsRef<[u8]>)]
    pub fn describe_bytes(&self) -> String {
        format!("{} ({} bytes)", self, self.as_ref().len())
    }

    #[cgp_fn]
    #[uses(RectangleArea)]
    #[uses(Display)]
    pub fn labelled_area(&self) -> String {
        format!("{}: {}", self, self.rectangle_area())
    }

    #[cgp_fn]
    #[uses(HasErrorType<Error = String>)]
    pub fn fail(&self) -> Result<(), String> {
        Err("failed".to_owned())
    }

    #[cgp_fn]
    #[uses('static, for<'a> PartialEq<&'a str>)]
    pub fn is_ada(&self) -> bool {
        *self == "ada"
    }

    #[cgp_fn]
    #[uses()]
    pub fn nothing(&self) -> u8 {
        0
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[cgp_impl(new RectangleAreaCalculator)]
    #[uses(RectangleArea)]
    impl AreaCalculator {
        fn area(&self) -> f64 {
            self.rectangle_area()
        }
    }

    impl Display for Rectangle {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "rectangle")
        }
    }

    delegate_components! {
        Rectangle {
            AreaCalculatorComponent: RectangleAreaCalculator,
            ErrorTypeProviderComponent: UseType<String>,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorComponent,
        }
    }

    #[test]
    fn every_form_imports() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(rect.both_areas(), 12.0);
        assert_eq!(rect.labelled_area(), "rectangle: 6");
        assert_eq!(rect.fail(), Err("failed".to_owned()));
        assert_eq!("hello".to_owned().describe_bytes(), "hello (5 bytes)");
        assert!("ada".is_ada());
        assert_eq!(rect.nothing(), 0);
    }
}

/// ## Examples
///
/// Both examples, with the value context the page wires for the second.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_fn]
    #[uses(CanCalculateArea)]
    pub fn scaled_area(&self, #[implicit] scale_factor: f64) -> f64 {
        self.area() * scale_factor * scale_factor
    }

    #[cgp_fn]
    pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }

    #[cgp_impl(new RectangleAreaCalculator)]
    #[uses(RectangleArea)]
    impl AreaCalculator {
        fn area(&self) -> f64 {
            self.rectangle_area()
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        Rectangle {
            AreaCalculatorComponent: RectangleAreaCalculator,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorComponent,
        }
    }

    /// A context with a scale factor gains `scaled_area` through its wired provider.
    #[derive(HasField)]
    pub struct ScaledRectangle {
        pub width: f64,
        pub height: f64,
        pub scale_factor: f64,
    }

    delegate_components! {
        ScaledRectangle {
            AreaCalculatorComponent: RectangleAreaCalculator,
        }
    }

    check_components! {
        ScaledRectangle {
            AreaCalculatorComponent,
        }
    }

    #[test]
    fn the_provider_and_the_function_compose() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(rect.area(), 6.0);

        let scaled = ScaledRectangle {
            width: 2.0,
            height: 3.0,
            scale_factor: 2.0,
        };
        assert_eq!(scaled.scaled_area(), 24.0);
    }
}

/// ## Under the hood
///
/// The listing's input, and the stacked form the section says produces the same predicate. The
/// page names `BaseArea` without declaring it.
pub mod under_the_hood {
    use core::fmt::Display;

    use cgp::prelude::*;

    #[cgp_fn]
    pub fn base_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }

    #[cgp_fn]
    #[uses(BaseArea, Display)]
    pub fn describe_area(&self) -> String {
        format!("{} for {}", self.base_area(), self)
    }

    #[cgp_fn]
    #[uses(BaseArea)]
    #[uses(Display)]
    pub fn describe_area_stacked(&self) -> String {
        format!("{} for {}", self.base_area(), self)
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    impl Display for Rectangle {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "rectangle")
        }
    }

    #[test]
    fn both_forms_describe() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(rect.describe_area(), "6 for rectangle");
        assert_eq!(rect.describe_area_stacked(), rect.describe_area());
    }
}
