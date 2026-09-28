//! Code from `docs/reference/providers/dispatch/match_first_with_handlers.md` — `MatchFirstWithHandlers`.
//!
//! Pins the multi-argument calling convention: the input is `(Input, Args)`, and each per-variant
//! handler receives the matched payload together with the shared `Args`, both through the explicit
//! list and through the `MatchFirstWithValueHandlers` convenience alias.

/// ## Examples
pub mod examples {
    use cgp::extra::dispatch::{
        ExtractFirstFieldAndHandle, HandleFirstFieldValue, MatchFirstWithHandlers,
    };
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
    impl<Context, Code> Computer<Context, Code, (Circle, f64)> for ScaledArea {
        type Output = f64;

        fn compute(
            _context: &Context,
            _code: PhantomData<Code>,
            (circle, scale): (Circle, f64),
        ) -> f64 {
            core::f64::consts::PI * circle.radius * circle.radius * scale * scale
        }
    }

    #[cgp_provider]
    impl<Context, Code> Computer<Context, Code, (Rectangle, f64)> for ScaledArea {
        type Output = f64;

        fn compute(
            _context: &Context,
            _code: PhantomData<Code>,
            (rectangle, scale): (Rectangle, f64),
        ) -> f64 {
            rectangle.width * rectangle.height * scale * scale
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent:
                MatchFirstWithHandlers<Product![
                    ExtractFirstFieldAndHandle<Symbol!("Circle"), HandleFirstFieldValue<ScaledArea>>,
                    ExtractFirstFieldAndHandle<Symbol!("Rectangle"), HandleFirstFieldValue<ScaledArea>>,
                ]>,
        }
    }

    check_components! {
        App {
            ComputerComponent: ((), (Shape, f64)),
        }
    }

    pub fn demo() {
        let code = PhantomData::<()>;
        let rectangle = Shape::Rectangle(Rectangle {
            width: 3.0,
            height: 4.0,
        });

        assert_eq!(App.compute(code, (rectangle, 2.0)), 48.0); // 12 * 2 * 2
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## When to use it
///
/// `MatchFirstWithValueHandlers` builds the same list from the enum's variants.
pub mod when_to_use_it {
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
    impl<Context, Code> Computer<Context, Code, (Circle, f64)> for ScaledArea {
        type Output = f64;

        fn compute(
            _context: &Context,
            _code: PhantomData<Code>,
            (circle, scale): (Circle, f64),
        ) -> f64 {
            core::f64::consts::PI * circle.radius * circle.radius * scale * scale
        }
    }

    #[cgp_provider]
    impl<Context, Code> Computer<Context, Code, (Rectangle, f64)> for ScaledArea {
        type Output = f64;

        fn compute(
            _context: &Context,
            _code: PhantomData<Code>,
            (rectangle, scale): (Rectangle, f64),
        ) -> f64 {
            rectangle.width * rectangle.height * scale * scale
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: MatchFirstWithValueHandlers<ScaledArea>,
        }
    }

    check_components! {
        App {
            ComputerComponent: ((), (Shape, f64)),
        }
    }

    #[test]
    fn the_alias_builds_the_same_list() {
        let rectangle = Shape::Rectangle(Rectangle {
            width: 3.0,
            height: 4.0,
        });

        assert_eq!(App.compute(PhantomData::<()>, (rectangle, 2.0)), 48.0);
    }
}
