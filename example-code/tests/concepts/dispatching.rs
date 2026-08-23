//! Code from `docs/concepts/dispatching.md` — *Dispatching*.

/// ## Matching: one handler per variant
pub mod matching_one_handler_per_variant {
    use core::convert::Infallible;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::dispatch::MatchWithValueHandlers;
    use cgp::extra::handler::{ComputerComponent, UseInputDelegate};
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
            ErrorTypeProviderComponent: UseType<Infallible>,

            ComputerComponent: UseInputDelegate<
                // The whole enum routes to the matcher; each variant's payload routes to its own
                // handler. No indirection is needed here — a *recursive* language, where a
                // variant's payload contains the enum again, is the case that needs a thin
                // wrapper between the two to break the resolution cycle.
                new AreaComponents {
                    Shape: MatchWithValueHandlers,
                    Circle: CircleArea,
                    Rectangle: RectangleArea,
                }
            >,
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
    fn the_value_reaches_the_handler_for_its_variant() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(
            app.compute(code, Shape::Rectangle(Rectangle { width: 3.0, height: 4.0 })),
            12.0,
        );

        let circle_area = app.compute(code, Shape::Circle(Circle { radius: 1.0 }));
        assert!((circle_area - core::f64::consts::PI).abs() < 1e-9);
    }
}

/// ## The same idea from a per-type trait
///
/// When the per-variant logic is an ordinary trait with one impl per payload type,
/// `#[cgp_auto_dispatch]` generates the matcher wiring rather than making it something to write.
pub mod the_same_idea_from_a_per_type_trait {
    use cgp::prelude::*;

    #[cgp_auto_dispatch]
    pub trait CanDescribe {
        fn describe(&self) -> String;
    }

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

    impl CanDescribe for Circle {
        fn describe(&self) -> String {
            format!("circle of radius {}", self.radius)
        }
    }

    impl CanDescribe for Rectangle {
        fn describe(&self) -> String {
            format!("{}x{} rectangle", self.width, self.height)
        }
    }

    #[test]
    fn the_enum_gains_the_trait_from_its_variants() {
        let shape = Shape::Circle(Circle { radius: 2.0 });
        assert_eq!(shape.describe(), "circle of radius 2");
    }
}
