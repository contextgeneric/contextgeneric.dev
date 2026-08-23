//! Code from `docs/concepts/abstract-types.md` — *Abstract types*.

/// ## An associated type is already this, in ordinary Rust
pub mod an_associated_type_is_already_this {
    pub trait HasScalarType {
        type Scalar: Copy;
    }

    pub struct HighPrecision;
    pub struct Embedded;

    impl HasScalarType for HighPrecision {
        type Scalar = f64;
    }

    impl HasScalarType for Embedded {
        type Scalar = f32;
    }

    // Generic code names the type without fixing it.
    pub fn double<Context>(scalar: Context::Scalar) -> Context::Scalar
    where
        Context: HasScalarType,
        Context::Scalar: core::ops::Add<Output = Context::Scalar>,
    {
        scalar + scalar
    }
}

/// ## Choosing the type by wiring
pub mod choosing_the_type_by_wiring {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Copy;
    }

    pub struct HighPrecision;
    pub struct Embedded;

    delegate_components! {
        HighPrecision {
            ScalarTypeProviderComponent: UseType<f64>,
        }
    }

    delegate_components! {
        Embedded {
            ScalarTypeProviderComponent: UseType<f32>,
        }
    }

    mod check_high_precision {
        use super::*;
        check_components! { HighPrecision { ScalarTypeProviderComponent } }
    }

    mod check_embedded {
        use super::*;
        check_components! { Embedded { ScalarTypeProviderComponent } }
    }

    #[test]
    fn each_context_answers_with_its_own_type() {
        fn size_of_scalar<Context: HasScalarType>() -> usize {
            core::mem::size_of::<Context::Scalar>()
        }

        assert_eq!(size_of_scalar::<HighPrecision>(), 8);
        assert_eq!(size_of_scalar::<Embedded>(), 4);
    }
}

/// ## One type, agreed on by everything that needs it
///
/// The shapes carry no scalar type of their own. The context supplies one, and every capability
/// that mentions it means the same type.
pub mod one_type_agreed_on_by_everything {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Copy;
    }

    #[cgp_component(ShapeAreaCalculator)]
    #[use_type(HasScalarType.Scalar)]
    pub trait CanCalculateShapeArea<Shape> {
        fn shape_area(&self, shape: &Shape) -> Scalar;
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[cgp_impl(new RectangleArea)]
    #[use_type(HasScalarType.{Scalar = f64})]
    impl ShapeAreaCalculator<Rectangle> {
        fn shape_area(&self, shape: &Rectangle) -> Scalar {
            shape.width * shape.height
        }
    }

    pub struct App;

    delegate_components! {
        App {
            // An `open` statement has to lead the table, before any plain entry.
            open ShapeAreaCalculatorComponent;

            ScalarTypeProviderComponent: UseType<f64>,
            @ShapeAreaCalculatorComponent.Rectangle: RectangleArea,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ScalarTypeProviderComponent,
                ShapeAreaCalculatorComponent: Rectangle,
            }
        }
    }

    #[test]
    fn the_shape_never_names_a_scalar() {
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert_eq!(App.shape_area(&rectangle), 12.0);
    }
}

/// ## The canonical one: a context's error type
pub mod the_canonical_one_an_error_type {
    // The error component's wiring keys and its backend providers are deliberately not in the
    // prelude, so they are imported by name.
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self, path: &str) -> Result<String, Error>;
    }

    #[cgp_impl(new LoadOrFail)]
    #[uses(CanRaiseError<String>)]
    #[use_type(HasErrorType.Error)]
    impl Loader {
        fn load(&self, path: &str) -> Result<String, Error> {
            if path.is_empty() {
                return Err(Self::raise_error("empty path".to_owned()));
            }
            Ok(format!("contents of {path}"))
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            LoaderComponent: LoadOrFail,
            @ErrorRaiserComponent.String: RaiseFrom,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { LoaderComponent } }
    }

    #[test]
    fn the_provider_never_names_the_concrete_error() {
        assert_eq!(App.load("a.txt").unwrap(), "contents of a.txt");
        assert_eq!(App.load("").unwrap_err(), "empty path");
    }
}
