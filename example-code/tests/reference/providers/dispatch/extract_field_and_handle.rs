//! Code from `docs/reference/providers/dispatch/extract_field_and_handle.md` — `ExtractFieldAndHandle`.
//!
//! Pins the Examples program, which calls two adapters by hand to show the `Result` shape a matcher
//! loop consumes: a hit returns `Ok` of the handler's output, and a miss returns the remainder, which
//! the next adapter takes as its input.

/// ## Examples
pub mod examples {
    use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue};
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

    pub type CircleArm = ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>;
    pub type RectangleArm =
        ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>;

    pub struct App;

    pub fn demo() {
        let code = PhantomData::<()>;

        // A hit: the circle arm extracts its variant and returns `Ok` of the area.
        let circle = Shape::Circle(Circle { radius: 1.0 });
        let hit = CircleArm::compute(&App, code, circle.to_extractor());
        assert_eq!(hit.ok(), Some(core::f64::consts::PI));

        // A miss: the circle arm hands back the remainder, which the rectangle arm takes.
        let rectangle = Shape::Rectangle(Rectangle {
            width: 3.0,
            height: 4.0,
        });
        match CircleArm::compute(&App, code, rectangle.to_extractor()) {
            Ok(_) => panic!("a rectangle is not a circle"),
            Err(remainder) => {
                let area = RectangleArm::compute(&App, code, remainder);
                assert_eq!(area.ok(), Some(12.0));
            }
        }
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
