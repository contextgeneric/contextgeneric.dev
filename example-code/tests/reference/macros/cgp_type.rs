//! Code from `docs/reference/macros/cgp_type.md` — *`#[cgp_type]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`.

/// ## Overview
///
/// The wiring that supplies the scalar, and the direct impl the page offers as the alternative,
/// each on its own context since a context can take only one of them.
pub mod overview {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar;
    }

    pub struct App;

    delegate_components! {
        App {
            ScalarTypeProviderComponent: UseType<f64>,
        }
    }

    check_components! {
        App {
            ScalarTypeProviderComponent,
        }
    }

    pub struct DirectApp;

    impl HasScalarType for DirectApp {
        type Scalar = f64;
    }

    #[test]
    fn both_contexts_pick_f64() {
        let _: <App as HasScalarType>::Scalar = 1.0f64;
        let _: <DirectApp as HasScalarType>::Scalar = 1.0f64;
    }
}

/// ## Usage
///
/// The default name, the bare override, the keyed form, and a generic trait parameter, which the
/// page states in prose. Each is its own module since each declares `HasScalarType`.
pub mod usage {
    pub mod default_name {
        use cgp::prelude::*;

        #[cgp_type]
        pub trait HasScalarType {
            type Scalar;
        }

        pub struct App;

        delegate_components! {
            App {
                ScalarTypeProviderComponent: UseType<f64>,
            }
        }

        check_components! {
            App {
                ScalarTypeProviderComponent,
            }
        }
    }

    pub mod bare_override {
        use cgp::prelude::*;

        #[cgp_type(ProvideScalar)]
        pub trait HasScalarType {
            type Scalar;
        }

        pub struct App;

        delegate_components! {
            App {
                ProvideScalarComponent: UseType<f64>,
            }
        }

        check_components! {
            App {
                ProvideScalarComponent,
            }
        }
    }

    pub mod keyed_form {
        use cgp::prelude::*;

        #[cgp_type {
            name: ScalarComponent,
            context: Ctx,
        }]
        pub trait HasScalarType {
            type Scalar;
        }

        pub struct App;

        delegate_components! {
            App {
                ScalarComponent: UseType<f64>,
            }
        }

        check_components! {
            App {
                ScalarComponent,
            }
        }
    }

    pub mod generic_trait {
        use cgp::prelude::*;

        #[cgp_type]
        pub trait HasLabelType<Kind> {
            type Label;
        }

        pub struct App;

        delegate_components! {
            App {
                open LabelTypeProviderComponent;

                @LabelTypeProviderComponent.u32: UseType<String>,
            }
        }

        check_components! {
            App {
                LabelTypeProviderComponent: u32,
            }
        }

        #[test]
        fn the_label_type_is_chosen_per_parameter() {
            let _: <App as HasLabelType<u32>>::Label = String::new();
        }
    }

    /// The `Copy` bound, satisfied by the wired `f64`.
    pub mod bounds_on_the_associated_type {
        use cgp::prelude::*;

        #[cgp_type]
        pub trait HasScalarType {
            type Scalar: Copy;
        }

        pub struct App;

        delegate_components! {
            App {
                ScalarTypeProviderComponent: UseType<f64>,
            }
        }

        check_components! {
            App {
                ScalarTypeProviderComponent,
            }
        }
    }
}

/// ## Examples
///
/// The abstract scalar with its check, the generic `zero`, and the area component importing the
/// scalar. The page names `Rectangle` and `Circle` without declaring them; they are declared and
/// wired per shape here, with the context supplying `f64`.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Copy;
    }

    pub struct App;

    delegate_components! {
        App {
            open AreaCalculatorComponent;

            ScalarTypeProviderComponent: UseType<f64>,
            @AreaCalculatorComponent.Rectangle: RectangleArea,
            @AreaCalculatorComponent.Circle: CircleArea,
        }
    }

    check_components! {
        App {
            ScalarTypeProviderComponent,
            AreaCalculatorComponent: [Rectangle, Circle],
        }
    }

    pub fn zero<Context>() -> Context::Scalar
    where
        Context: HasScalarType,
        Context::Scalar: Default,
    {
        Default::default()
    }

    #[cgp_component(AreaCalculator)]
    #[use_type(HasScalarType.Scalar)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> Scalar;
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    pub struct Circle {
        pub radius: f64,
    }

    #[cgp_impl(new RectangleArea)]
    #[use_type(HasScalarType.{Scalar = f64})]
    impl AreaCalculator<Rectangle> {
        fn area(&self, shape: &Rectangle) -> Scalar {
            shape.width * shape.height
        }
    }

    #[cgp_impl(new CircleArea)]
    #[use_type(HasScalarType.{Scalar = f64})]
    impl AreaCalculator<Circle> {
        fn area(&self, shape: &Circle) -> Scalar {
            core::f64::consts::PI * shape.radius * shape.radius
        }
    }

    #[test]
    fn the_context_decides_the_scalar() {
        assert_eq!(zero::<App>(), 0.0);
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(App.area(&rect), 6.0);
        assert!(App.area(&Circle { radius: 1.0 }) > 3.14);
    }
}

/// ## Under the hood
///
/// The self-referential bound whose copy on the `UseType` and `WithProvider` impls is rewritten.
/// Checked with `cargo cgp expand --item reference::macros::cgp_type::under_the_hood`.
pub mod under_the_hood {
    use core::ops::Mul;

    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar: Mul<Output = Self::Scalar> + Clone;
    }

    pub struct App;

    delegate_components! {
        App {
            ScalarTypeProviderComponent: UseType<f64>,
        }
    }

    check_components! {
        App {
            ScalarTypeProviderComponent,
        }
    }
}
