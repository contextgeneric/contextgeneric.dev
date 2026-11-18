//! Code from `docs/comparisons/dynamic-dispatch.md` — *Dynamic dispatch, dynamic typing, and
//! prototypal inheritance*.

/// ## Provider code calls methods on a generic receiver
pub mod cgp_code_reads_like_a_duck_typed_program {
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

    #[cgp_fn]
    pub fn greet_implicitly(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    delegate_components! { Person { GreeterComponent: GreetHello } }

    mod check_person {
        use super::*;
        check_components! { Person { GreeterComponent } }
    }

    #[test]
    fn the_context_responds_to_the_message() {
        let person = Person {
            name: "Ada".to_owned(),
        };
        assert_eq!(person.greet(), "Hello, Ada!");
        assert_eq!(person.greet_implicitly(), "Hello, Ada!");
    }
}

/// ## Wiring serves the selection role of a vtable
/// ## Delegation preserves the original context
pub mod delegate_component_is_a_compile_time_vtable {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_component(PerimeterCalculator)]
    pub trait CanCalculatePerimeter {
        fn perimeter(&self) -> f64;
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            width * height
        }
    }

    #[cgp_impl(new RectanglePerimeter)]
    impl PerimeterCalculator {
        fn perimeter(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            2.0 * (width + height)
        }
    }

    delegate_components! {
        new GeometryComponents {
            AreaCalculatorComponent: RectangleArea,
            PerimeterCalculatorComponent: RectanglePerimeter,
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        Rectangle {
            [AreaCalculatorComponent, PerimeterCalculatorComponent]: GeometryComponents,
        }
    }

    mod check_rectangle {
        use super::*;
        check_components! {
            Rectangle {
                AreaCalculatorComponent,
                PerimeterCalculatorComponent,
            }
        }
    }

    #[test]
    fn self_stays_bound_to_the_original_receiver() {
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        // `RectangleArea` reads `width` and `height` from `Rectangle`, not from the bundle.
        assert_eq!(rectangle.area(), 12.0);
        assert_eq!(rectangle.perimeter(), 14.0);
    }
}

/// ## Namespaces share bindings and leave paths for contexts to fill
pub mod namespaces_are_shared_prototypes {
    use cgp::prelude::*;

    cgp_namespace! { new AppNamespace {} }

    #[cgp_component(Greeter)]
    #[prefix(@app.GreeterComponent in AppNamespace)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_component(Farewell)]
    #[prefix(@app.FarewellComponent in AppNamespace)]
    pub trait CanSayGoodbye {
        fn goodbye(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello!".to_owned()
        }
    }

    #[cgp_impl(new GreetQuietly)]
    impl Greeter {
        fn greet(&self) -> String {
            "hello".to_owned()
        }
    }

    #[cgp_impl(new SayGoodbye)]
    impl Farewell {
        fn goodbye(&self) -> String {
            "Goodbye!".to_owned()
        }
    }

    // The shared prototype: it binds the farewell and leaves the greeting path open.
    cgp_namespace! {
        new AppDefaults: AppNamespace {
            @app.FarewellComponent: SayGoodbye,
        }
    }

    pub struct AppA;

    delegate_components! {
        AppA {
            namespace AppDefaults;         // inherit every wiring the namespace binds

            @app.GreeterComponent: GreetHello,   // fill a slot the namespace leaves open
        }
    }

    // A child namespace may bind an open path; it may not redefine a bound one.
    cgp_namespace! {
        new QuietDefaults: AppDefaults {
            @app.GreeterComponent: GreetQuietly,
        }
    }

    pub struct AppB;

    delegate_components! {
        AppB {
            namespace QuietDefaults;
        }
    }

    mod check_app_a {
        use super::*;
        check_components! { AppA { GreeterComponent, FarewellComponent } }
    }

    mod check_app_b {
        use super::*;
        check_components! { AppB { GreeterComponent, FarewellComponent } }
    }

    #[test]
    fn the_chain_is_walked_by_the_type_checker() {
        assert_eq!(AppA.greet(), "Hello!");
        assert_eq!(AppA.goodbye(), "Goodbye!");
        assert_eq!(AppB.greet(), "hello");
        assert_eq!(AppB.goodbye(), "Goodbye!");
    }
}
