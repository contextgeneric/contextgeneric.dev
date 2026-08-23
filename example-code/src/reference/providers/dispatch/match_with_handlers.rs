//! Code from `docs/reference/providers/dispatch/match_with_handlers.md` — `MatchWithHandlers`.

/// ## Usage and Examples
///
/// `MatchWithHandlers` runs a spelled-out list of per-variant adapters over an enum, stopping at the
/// first match. Each adapter extracts one variant and hands its payload to a handler; the list is
/// exhaustive, so no wildcard arm is needed.
pub mod matching_an_explicit_list {
    use core::marker::PhantomData;

    use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};
    use cgp::extra::handler::{Computer, ComputerComponent};
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

    // One provider handles every payload type, with an impl per shape.
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
            ComputerComponent: MatchWithHandlers<
                Product![
                    ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
                    ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
                ]
            >,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: ((), Shape),
            }
        }
    }

    #[test]
    fn the_first_matching_arm_handles_the_value() {
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
