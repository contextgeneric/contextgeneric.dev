//! Code from `docs/reference/providers/dispatch/match_with_handlers.md` — `MatchWithHandlers`.
//!
//! Pins the borrowed form from Usage, the Examples program, and the lifts Common Mistakes gives for
//! the fallible slots. A list that misses a variant, and a matcher wired to `HandlerComponent`, are
//! trybuild fixtures.

/// ## Usage
///
/// `MatchWithHandlersRef` takes the same list over a borrowed input, so each payload handler
/// computes over a borrowed payload.
pub mod usage {
    use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlersRef};
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
    impl<'a, Context, Code> Computer<Context, Code, &'a Circle> for ComputeAreaRef {
        type Output = f64;

        fn compute(_context: &Context, _code: PhantomData<Code>, circle: &'a Circle) -> f64 {
            core::f64::consts::PI * circle.radius * circle.radius
        }
    }

    #[cgp_provider]
    impl<'a, Context, Code> Computer<Context, Code, &'a Rectangle> for ComputeAreaRef {
        type Output = f64;

        fn compute(_context: &Context, _code: PhantomData<Code>, rectangle: &'a Rectangle) -> f64 {
            rectangle.width * rectangle.height
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent:
                MatchWithHandlersRef<Product![
                    ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeAreaRef>>,
                    ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeAreaRef>>,
                ]>,
        }
    }

    check_components! {
        App {
            ComputerComponent: <'a> ((), &'a Shape),
        }
    }

    #[test]
    fn the_borrowed_matcher_leaves_the_value_in_place() {
        let shape = Shape::Rectangle(Rectangle {
            width: 3.0,
            height: 4.0,
        });

        assert_eq!(App.compute(PhantomData::<()>, &shape), 12.0);
        assert!(matches!(shape, Shape::Rectangle(_)));
    }
}

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
        let code = PhantomData::<()>;

        let rectangle = Shape::Rectangle(Rectangle {
            width: 3.0,
            height: 4.0,
        });
        assert_eq!(App.compute(code, rectangle), 12.0);

        let circle = App.compute(code, Shape::Circle(Circle { radius: 1.0 }));
        assert!((circle - core::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## Common Mistakes
///
/// A matcher implements only `Computer` and `AsyncComputer`, so the fallible slots take it through
/// the handler promotions: `Promote` for `TryComputer`, and `PromoteAsync<Promote<…>>` for `Handler`.
pub mod common_mistakes {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};
    use cgp::extra::handler::{CanHandle, CanTryCompute, Promote, PromoteAsync};
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

    pub type AreaMatcher = MatchWithHandlers<
        Product![
            ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
            ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
        ],
    >;

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            TryComputerComponent: Promote<AreaMatcher>,
            HandlerComponent: PromoteAsync<Promote<AreaMatcher>>,
        }
    }

    check_components! {
        App {
            [TryComputerComponent, HandlerComponent]: ((), Shape),
        }
    }

    pub async fn demo() {
        let code = PhantomData::<()>;
        let shape = || {
            Shape::Rectangle(Rectangle {
                width: 3.0,
                height: 4.0,
            })
        };

        assert_eq!(App.try_compute(code, shape()), Ok(12.0));
        assert_eq!(App.handle(code, shape()).await, Ok(12.0));
    }

    #[test]
    fn test_demo() {
        futures::executor::block_on(demo());
    }
}
