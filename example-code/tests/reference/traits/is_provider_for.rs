//! Code from `docs/reference/traits/wiring/is_provider_for.md` — `IsProviderFor`.
//!
//! Pins the Params rule from Usage and the Examples program: a provider with a dependency, a context
//! that meets it, and the marker asserted both through the context and on the provider directly. A
//! context that misses the dependency, a one-element tuple for `Params`, and a provider impl written
//! without its attribute are trybuild fixtures.

/// ## Usage
///
/// One component parameter is passed directly as `Params`.
pub mod usage {
    use cgp::prelude::*;

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

    pub struct App;

    pub fn assert_provider()
    where
        RectangleArea: IsProviderFor<AreaCalculatorComponent, App, Rectangle>,
    {
    }

    #[test]
    fn one_parameter_is_passed_directly() {
        assert_provider();
    }
}

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

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    delegate_components! {
        Person {
            GreeterComponent: GreetHello,
        }
    }

    check_components! {
        Person {
            GreeterComponent,
        }
    }

    check_components! {
        #[check_trait(CheckGreetHello)]
        #[check_providers(GreetHello)]
        Person {
            GreeterComponent,
        }
    }

    pub fn demo() {
        let person = Person {
            name: "Ada".to_owned(),
        };

        assert_eq!(person.greet(), "Hello, Ada!");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
