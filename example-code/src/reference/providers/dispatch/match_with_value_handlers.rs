//! Code from `docs/reference/providers/dispatch/match_with_value_handlers.md` — `MatchWithValueHandlers`.

/// ## Usage and Examples
///
/// `MatchWithValueHandlers` builds the per-variant handler list from the enum's own fields and passes
/// each payload as a bare value. Wired through `UseInputDelegate`, it is selected when the input is a
/// `Shape`, and each payload routes back through the context's own `ComputerComponent` to its handler.
pub mod matching_by_value {
    use core::convert::Infallible;
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::dispatch::MatchWithValueHandlers;
    use cgp::extra::handler::{Computer, ComputerComponent, UseInputDelegate};
    use cgp::prelude::*;

    #[derive(CgpData)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    pub struct Circle {
        pub radius: f64,
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, Circle> for ComputeArea {
        type Output = f64;

        fn compute(_context: &Context, _code: PhantomData<Code>, circle: Circle) -> f64 {
            core::f64::consts::PI * circle.radius * circle.radius
        }
    }

    #[cgp_provider]
    impl<Context, Code> Computer<Context, Code, Rectangle> for ComputeArea {
        type Output = f64;

        fn compute(_context: &Context, _code: PhantomData<Code>, rectangle: Rectangle) -> f64 {
            rectangle.width * rectangle.height
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<Infallible>,

            ComputerComponent: UseInputDelegate<new AreaComputers {
                Shape: MatchWithValueHandlers,
                Circle: ComputeArea,
                Rectangle: ComputeArea,
            }>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: [((), Shape), ((), Circle), ((), Rectangle)],
            }
        }
    }

    #[test]
    fn each_variant_reaches_its_handler() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(
            app.compute(code, Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 })),
            12.0,
        );

        let circle = app.compute(code, Shape::Circle(Circle { radius: 1.0 }));
        assert!((circle - core::f64::consts::PI).abs() < 1e-9);
    }
}
