//! Code from `docs/reference/traits/wiring/can_use_component.md` — `CanUseComponent`.
//!
//! Pins the Examples program: a component with no parameters and one with a parameter, each checked
//! through `check_components!` and through a hand-written `CanUseComponent` bound.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    #[uses(HasName)]
    impl Greeter {
        fn greet(&self) -> String {
            format!("Hello, {}!", self.name())
        }
    }

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator<Rectangle> {
        fn area(&self, rectangle: &Rectangle) -> f64 {
            rectangle.width * rectangle.height
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
    }

    delegate_components! {
        App {
            GreeterComponent: GreetHello,
            AreaCalculatorComponent: RectangleArea,
        }
    }

    check_components! {
        App {
            GreeterComponent,
            AreaCalculatorComponent: Rectangle,
        }
    }

    pub fn assert_wiring()
    where
        App:
            CanUseComponent<GreeterComponent> + CanUseComponent<AreaCalculatorComponent, Rectangle>,
    {
    }

    pub fn demo() {
        assert_wiring();

        let app = App {
            name: "Ada".to_owned(),
        };
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };

        assert_eq!(app.greet(), "Hello, Ada!");
        assert_eq!(app.area(&rectangle), 12.0);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
