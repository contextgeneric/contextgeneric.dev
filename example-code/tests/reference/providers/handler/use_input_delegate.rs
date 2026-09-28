//! Code from `docs/reference/providers/handler/use_input_delegate.md` — `UseInputDelegate`.
//!
//! Pins the Usage and Examples programs, the legacy table and its `open` equivalent, and the
//! component whose generated `UseInputDelegate` impl Under the hood lists, for `cargo cgp expand`.

/// ## Examples
pub mod examples {
    use cgp::extra::handler::{CanCompute, UseInputDelegate};
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

    check_components! {
        App {
            ComputerComponent: [((), Circle), ((), Rectangle)],
        }
    }

    pub struct OpenApp;

    delegate_components! {
        OpenApp {
            open ComputerComponent;

            @ComputerComponent.<Code> Code.Circle: CircleArea,
            @ComputerComponent.<Code> Code.Rectangle: RectangleArea,
        }
    }

    check_components! {
        OpenApp {
            ComputerComponent: [((), Circle), ((), Rectangle)],
        }
    }

    pub fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.compute(code, Rectangle { width: 3.0, height: 4.0 }), 12.0);
        assert_eq!(OpenApp.compute(code, Rectangle { width: 3.0, height: 4.0 }), 12.0);
        assert!((App.compute(code, Circle { radius: 1.0 }) - core::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## Under the hood
///
/// A component declared with the same `#[derive_delegate(UseInputDelegate<Input>)]` directive as the
/// handler components, whose generated impl the page lists.
pub mod under_the_hood {
    use cgp::extra::handler::UseInputDelegate;
    use cgp::prelude::*;

    #[cgp_component(Measurer)]
    #[derive_delegate(UseInputDelegate<Input>)]
    pub trait CanMeasure<Code, Input> {
        type Output;

        fn measure(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
    }
}
