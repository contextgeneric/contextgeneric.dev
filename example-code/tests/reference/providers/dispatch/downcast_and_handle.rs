//! Code from `docs/reference/providers/dispatch/downcast_and_handle.md` — `DowncastAndHandle`.
//!
//! Pins the Examples program: `DowncastAndHandle` narrows a `Shape` to the smaller `Quadrilateral`
//! enum and hands both of its variants to one handler, beside a single-variant arm for `Circle`.

/// ## Examples
pub mod examples {
    use cgp::extra::dispatch::{
        DowncastAndHandle, ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers,
    };
    use cgp::extra::handler::CanCompute;
    use cgp::prelude::*;

    pub struct Circle {
        pub radius: f64,
    }

    pub struct Square {
        pub side: f64,
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[derive(CgpData)]
    pub enum Shape {
        Circle(Circle),
        Square(Square),
        Rectangle(Rectangle),
    }

    #[derive(CgpData)]
    pub enum Quadrilateral {
        Square(Square),
        Rectangle(Rectangle),
    }

    #[cgp_computer]
    pub fn count_circle_corners(_circle: Circle) -> u32 {
        0
    }

    #[cgp_computer]
    pub fn count_quadrilateral_corners(_shape: Quadrilateral) -> u32 {
        4
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent:
                MatchWithHandlers<Product![
                    ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<CountCircleCorners>>,
                    DowncastAndHandle<Quadrilateral, CountQuadrilateralCorners>,
                ]>,
        }
    }

    check_components! {
        App {
            ComputerComponent: ((), Shape),
        }
    }

    pub fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.compute(code, Shape::Circle(Circle { radius: 1.0 })), 0);
        assert_eq!(App.compute(code, Shape::Square(Square { side: 2.0 })), 4);
        let rectangle = Shape::Rectangle(Rectangle {
            width: 3.0,
            height: 4.0,
        });
        assert_eq!(App.compute(code, rectangle), 4);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
