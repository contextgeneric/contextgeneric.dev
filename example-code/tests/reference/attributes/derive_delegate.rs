//! Code from `docs/reference/attributes/derive_delegate.md` — *`#[derive_delegate]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/attributes/`.

/// ## Usage
///
/// Two dispatchers on one component, the second through a user-defined wrapper, and a tuple key.
/// The page names `UseInputDelegate` as a struct the reader defines.
pub mod usage {
    use cgp::prelude::*;

    pub struct UseInputDelegate<Components>(pub PhantomData<Components>);

    #[cgp_component(Computer)]
    #[derive_delegate(UseDelegate<Code>)]
    #[derive_delegate(UseInputDelegate<Input>)]
    pub trait CanCompute<Code, Input> {
        type Output;

        fn compute(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
    }

    #[cgp_component(Pairer)]
    #[derive_delegate(UseDelegate<(Code, Input)>)]
    pub trait CanPair<Code, Input> {
        fn pair(&self, input: Input) -> u64;
    }

    pub struct Double;
    pub struct Negate;

    #[cgp_impl(new DoubleIt)]
    impl<Code> Computer<Code, u64> {
        type Output = u64;

        fn compute(&self, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    #[cgp_impl(new NegateIt)]
    impl<Code> Computer<Code, i64> {
        type Output = i64;

        fn compute(&self, _code: PhantomData<Code>, input: i64) -> i64 {
            -input
        }
    }

    #[cgp_impl(new PairDouble)]
    impl Pairer<Double, u64> {
        fn pair(&self, input: u64) -> u64 {
            input * 2
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent:
                UseInputDelegate<new ComputerInputs {
                    u64: DoubleIt,
                    i64: NegateIt,
                }>,
            PairerComponent:
                UseDelegate<new PairerTable {
                    (Double, u64): PairDouble,
                }>,
        }
    }

    check_components! {
        App {
            ComputerComponent: [(Double, u64), (Negate, i64)],
            PairerComponent: (Double, u64),
        }
    }

    #[test]
    fn each_dispatcher_keys_on_its_own_parameter() {
        assert_eq!(App.compute(PhantomData::<Double>, 4u64), 8);
        assert_eq!(App.compute(PhantomData::<Negate>, 4i64), -4);
        assert_eq!(CanPair::<Double, u64>::pair(&App, 3), 6);
    }
}

/// ## Examples
///
/// The dispatching component and the two wirings the page compares, on two contexts.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    #[derive_delegate(UseDelegate<Shape>)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }
    pub struct Circle {
        pub radius: f64,
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator<Rectangle> {
        fn area(&self, shape: &Rectangle) -> f64 {
            shape.width * shape.height
        }
    }

    #[cgp_impl(new CircleArea)]
    impl AreaCalculator<Circle> {
        fn area(&self, shape: &Circle) -> f64 {
            core::f64::consts::PI * shape.radius * shape.radius
        }
    }

    pub struct MyApp;

    delegate_components! {
        MyApp {
            AreaCalculatorComponent:
                UseDelegate<new AreaCalculatorComponents {
                    Rectangle: RectangleArea,
                    Circle: CircleArea,
                }>,
        }
    }

    check_components! {
        MyApp {
            AreaCalculatorComponent: [Rectangle, Circle],
        }
    }

    /// The same result through `open`. A second context stands in for the page's `MyApp`, since
    /// one context cannot use both forms.
    pub struct OpenApp;

    delegate_components! {
        OpenApp {
            open AreaCalculatorComponent;

            @AreaCalculatorComponent.Rectangle: RectangleArea,
            @AreaCalculatorComponent.Circle: CircleArea,
        }
    }

    check_components! {
        OpenApp {
            AreaCalculatorComponent: [Rectangle, Circle],
        }
    }

    #[test]
    fn both_forms_resolve_to_the_same_providers() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(MyApp.area(&rect), 6.0);
        assert_eq!(OpenApp.area(&rect), MyApp.area(&rect));
        assert_eq!(
            OpenApp.area(&Circle { radius: 1.0 }),
            MyApp.area(&Circle { radius: 1.0 })
        );
    }
}

/// ## Under the hood
///
/// A one-element tuple key, written with a trailing comma, keys the table on the tuple.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    #[derive_delegate(UseDelegate<Shape>)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    #[cgp_component(TupleArea)]
    #[derive_delegate(UseDelegate<(Shape,)>)]
    pub trait CanTupleArea<Shape> {
        fn tuple_area(&self, shape: &Shape) -> f64;
    }

    pub struct Square(pub f64);

    #[cgp_impl(new SquareArea)]
    impl TupleArea<Square> {
        fn tuple_area(&self, shape: &Square) -> f64 {
            shape.0 * shape.0
        }
    }

    pub struct App;

    delegate_components! {
        App {
            TupleAreaComponent:
                UseDelegate<new TupleAreas {
                    (Square,): SquareArea,
                }>,
        }
    }

    check_components! {
        App {
            TupleAreaComponent: Square,
        }
    }

    #[test]
    fn the_tuple_key_resolves() {
        assert_eq!(App.tuple_area(&Square(3.0)), 9.0);
    }
}
