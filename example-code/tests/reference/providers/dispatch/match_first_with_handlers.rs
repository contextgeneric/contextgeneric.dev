//! Code from `docs/reference/providers/dispatch/match_first_with_handlers.md` — `MatchFirstWithHandlers`.
//!
//! Pins the multi-argument calling convention: the input is `(Input, Args)`, and each per-variant
//! handler receives the matched payload together with the shared `Args`.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::dispatch::{
        ExtractFirstFieldAndHandle, HandleFirstFieldValue, MatchFirstWithHandlers,
    };
    use cgp::extra::handler::ComputerComponent;
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    #[derive(Debug, PartialEq)]
    pub struct Circle {
        pub radius: f64,
    }

    #[derive(Debug, PartialEq)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    pub trait Container {
        fn contains(self, x: f64, y: f64) -> bool;
    }

    impl Container for Circle {
        fn contains(self, _x: f64, _y: f64) -> bool {
            true
        }
    }

    impl Container for Rectangle {
        fn contains(self, _x: f64, _y: f64) -> bool {
            true
        }
    }

    // A computer that takes the payload plus the shared `(x, y)` argument.
    #[cgp_computer]
    fn contains<T: Container>(shape: T, (x, y): (f64, f64)) -> bool {
        shape.contains(x, y)
    }

    pub type Handlers = Product![
        ExtractFirstFieldAndHandle<Symbol!("Circle"), HandleFirstFieldValue<Contains>>,
        ExtractFirstFieldAndHandle<Symbol!("Rectangle"), HandleFirstFieldValue<Contains>>,
    ];

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: MatchFirstWithHandlers<Handlers>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: ((), (Shape, (f64, f64))),
            }
        }
    }

    #[test]
    fn test_match_first_with_handlers() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;
        let circle = Shape::Circle(Circle { radius: 5.0 });

        assert!(app.compute(code, (circle, (1.0, 2.0))));
    }
}
