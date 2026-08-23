//! Code from `docs/reference/providers/dispatch/handle_field_value.md` — `HandleFieldValue`.

/// ## Usage and Examples
///
/// `HandleFieldValue<Provider>` strips the `Field<Tag, Value>` wrapper an extract adapter delivers and
/// passes the bare `Value` to `Provider`. That is what lets an ordinary computer over the payload type
/// serve as a per-variant handler. Here `ComputeArea` is a plain computer over each payload type,
/// reached through `HandleFieldValue`.
pub mod unwrapping_the_field {
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

    // A plain computer over the bare payload type; it never sees the `Field` wrapper.
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
    fn the_handler_receives_the_bare_payload() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        let circle = app.compute(code, Shape::Circle(Circle { radius: 2.0 }));
        assert!((circle - 4.0 * core::f64::consts::PI).abs() < 1e-9);
    }
}
