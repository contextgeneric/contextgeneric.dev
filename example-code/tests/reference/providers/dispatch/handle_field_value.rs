//! Code from `docs/reference/providers/dispatch/handle_field_value.md` — `HandleFieldValue`.
//!
//! Pins the Examples program: `ComputeArea` is a plain computer over each payload type, reached from
//! the tagged `Field` an extract adapter delivers through `HandleFieldValue`.

/// ## Examples
pub mod examples {
    use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};
    use cgp::extra::handler::CanCompute;
    use cgp::prelude::*;

    pub struct Circle {
        pub radius: f64,
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[derive(CgpData)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
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
            ComputerComponent:
                MatchWithHandlers<Product![
                    ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
                    ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
                ]>,
        }
    }

    check_components! {
        App {
            ComputerComponent: ((), Shape),
        }
    }

    pub fn demo() {
        let circle = App.compute(PhantomData::<()>, Shape::Circle(Circle { radius: 2.0 }));
        assert!((circle - 4.0 * core::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
