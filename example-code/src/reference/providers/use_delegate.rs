//! Code from `docs/reference/providers/use_delegate.md` — *`UseDelegate`*.
//!
//! Pins the legacy nested-table dispatch and its `open` equivalent, both routing a `Shape` parameter
//! to a per-shape provider. The two forms are wired on separate contexts, since each is a full
//! wiring of the same component.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    #[derive_delegate(UseDelegate<Shape>)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }
    pub struct Circle {
        pub radius: f64,
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator<Rectangle> {
        fn area(&self, shape: &Rectangle) -> f64 {
            shape.width * shape.height
        }
    }

    #[cgp_impl(new CircleArea)]
    impl AreaCalculator<Circle> {
        fn area(&self, shape: &Circle) -> f64 {
            core::f64::consts::PI * shape.radius * shape.radius
        }
    }

    // Legacy nested-table form.
    pub struct MyApp;

    delegate_components! {
        MyApp {
            AreaCalculatorComponent:
                UseDelegate<new AreaCalculatorComponents {
                    Rectangle: RectangleArea,
                    Circle: CircleArea,
                }>,
        }
    }

    check_components! {
        MyApp {
            AreaCalculatorComponent: [Rectangle, Circle],
        }
    }

    // The `open` equivalent, on a separate context.
    pub struct MyApp2;

    delegate_components! {
        MyApp2 {
            open AreaCalculatorComponent;

            @AreaCalculatorComponent.Rectangle: RectangleArea,
            @AreaCalculatorComponent.Circle: CircleArea,
        }
    }

    check_components! {
        MyApp2 {
            AreaCalculatorComponent: [Rectangle, Circle],
        }
    }

    #[test]
    fn test_use_delegate_dispatch() {
        let app = MyApp;
        assert_eq!(
            app.area(&Rectangle {
                width: 2.0,
                height: 3.0
            }),
            6.0
        );
    }
}
