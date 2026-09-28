//! Code from `docs/reference/providers/dispatch/match_with_value_handlers.md` — `MatchWithValueHandlers`.
//!
//! Pins the Examples program, wired by input type with `open`, and the borrowed form from Usage,
//! which answers both `compute` over `&Shape` and `compute_ref` over `Shape`. A struct input and a
//! fallible slot on the borrowed form are trybuild fixtures.

/// ## Usage
///
/// `MatchWithValueHandlersRef` answers `ComputerComponent` over a borrowed enum with a provider over
/// borrowed payloads, and `ComputerRefComponent` with a `ComputerRef` provider over the payloads.
pub mod usage {
    use cgp::extra::handler::{CanCompute, CanComputeRef, ComputerRef};
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
    impl<'a, Context, Code> Computer<Context, Code, &'a Circle> for ComputeAreaOfRef {
        type Output = f64;

        fn compute(_context: &Context, _code: PhantomData<Code>, circle: &'a Circle) -> f64 {
            core::f64::consts::PI * circle.radius * circle.radius
        }
    }

    #[cgp_provider]
    impl<'a, Context, Code> Computer<Context, Code, &'a Rectangle> for ComputeAreaOfRef {
        type Output = f64;

        fn compute(_context: &Context, _code: PhantomData<Code>, rectangle: &'a Rectangle) -> f64 {
            rectangle.width * rectangle.height
        }
    }

    #[cgp_new_provider]
    impl<Context, Code> ComputerRef<Context, Code, Circle> for ComputeAreaByRef {
        type Output = f64;

        fn compute_ref(_context: &Context, _code: PhantomData<Code>, circle: &Circle) -> f64 {
            core::f64::consts::PI * circle.radius * circle.radius
        }
    }

    #[cgp_provider]
    impl<Context, Code> ComputerRef<Context, Code, Rectangle> for ComputeAreaByRef {
        type Output = f64;

        fn compute_ref(_context: &Context, _code: PhantomData<Code>, rectangle: &Rectangle) -> f64 {
            rectangle.width * rectangle.height
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: MatchWithValueHandlersRef<ComputeAreaOfRef>,
            ComputerRefComponent: MatchWithValueHandlersRef<ComputeAreaByRef>,
        }
    }

    check_components! {
        App {
            ComputerComponent: <'a> ((), &'a Shape),
            ComputerRefComponent: ((), Shape),
        }
    }

    #[test]
    fn one_matcher_serves_both_borrowed_interfaces() {
        let code = PhantomData::<()>;
        let shape = Shape::Rectangle(Rectangle {
            width: 3.0,
            height: 4.0,
        });

        assert_eq!(App.compute(code, &shape), 12.0);
        assert_eq!(App.compute_ref(code, &shape), 12.0);
    }
}

/// ## Usage: a named provider and the mutable form
///
/// A named provider needs no per-type entries, and `MatchWithValueHandlersMut` hands each payload
/// to a provider over `&mut` payloads, which can change the value in place.
pub mod usage_named_and_mut {
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

    #[cgp_new_provider]
    impl<'a, Context, Code> Computer<Context, Code, &'a mut Circle> for DoubleSize {
        type Output = ();

        fn compute(_context: &Context, _code: PhantomData<Code>, circle: &'a mut Circle) {
            circle.radius *= 2.0;
        }
    }

    #[cgp_provider]
    impl<'a, Context, Code> Computer<Context, Code, &'a mut Rectangle> for DoubleSize {
        type Output = ();

        fn compute(_context: &Context, _code: PhantomData<Code>, rectangle: &'a mut Rectangle) {
            rectangle.width *= 2.0;
            rectangle.height *= 2.0;
        }
    }

    pub struct Named;

    delegate_components! {
        Named {
            ComputerComponent: MatchWithValueHandlers<ComputeArea>,
        }
    }

    check_components! {
        Named {
            ComputerComponent: ((), Shape),
        }
    }

    pub struct Mutating;

    delegate_components! {
        Mutating {
            ComputerComponent: MatchWithValueHandlersMut<DoubleSize>,
        }
    }

    check_components! {
        Mutating {
            ComputerComponent: <'a> ((), &'a mut Shape),
        }
    }

    #[test]
    fn the_named_and_mutable_forms_match_each_variant() {
        let code = PhantomData::<()>;
        let mut shape = Shape::Rectangle(Rectangle {
            width: 3.0,
            height: 4.0,
        });

        Mutating.compute(code, &mut shape);
        assert_eq!(Named.compute(code, shape), 48.0);
    }
}

/// ## Examples
pub mod examples {
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
            open ComputerComponent;

            @ComputerComponent.<Code> Code.[Circle, Rectangle]: ComputeArea,
            @ComputerComponent.<Code> Code.Shape: MatchWithValueHandlers,
        }
    }

    check_components! {
        App {
            ComputerComponent: [((), Circle), ((), Rectangle), ((), Shape)],
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
