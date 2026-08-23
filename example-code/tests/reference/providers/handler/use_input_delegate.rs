//! Code from `docs/reference/providers/handler/use_input_delegate.md` — `UseInputDelegate`.

/// ## Usage and Examples
///
/// `UseInputDelegate` chooses the handler by the type of the input. Here a `Circle` input routes to
/// `CircleArea` and a `Rectangle` input to `RectangleArea`, through one wiring entry keyed on the
/// input type.
pub mod dispatching_on_the_input_type {
    use cgp::extra::handler::{ComputerComponent, UseInputDelegate};
    use cgp::prelude::*;

    pub struct Circle {
        pub radius: f64,
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[cgp_computer]
    pub fn circle_area(circle: Circle) -> f64 {
        core::f64::consts::PI * circle.radius * circle.radius
    }

    #[cgp_computer]
    pub fn rectangle_area(rectangle: Rectangle) -> f64 {
        rectangle.width * rectangle.height
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: UseInputDelegate<new AppComputers {
                Circle: CircleArea,
                Rectangle: RectangleArea,
            }>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: [((), Circle), ((), Rectangle)],
            }
        }
    }

    #[test]
    fn each_input_reaches_its_own_handler() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(
            app.compute(code, Rectangle { width: 3.0, height: 4.0 }),
            12.0,
        );

        let area = app.compute(code, Circle { radius: 1.0 });
        assert!((area - core::f64::consts::PI).abs() < 1e-9);
    }
}
