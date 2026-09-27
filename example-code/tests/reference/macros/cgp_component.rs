//! Code from `docs/reference/macros/cgp_component.md` — *`#[cgp_component]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`, one fixture
//! per *Common Mistakes* entry.

/// ## Usage
///
/// The bare form and the key/value form. The page shows the two as alternative definitions of one
/// trait, so each gets its own module, and a context wires each to prove the generated names are
/// the ones the page gives.
pub mod usage {
    pub mod bare_form {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_impl(new UnitArea)]
        impl AreaCalculator {
            fn area(&self) -> f64 {
                1.0
            }
        }

        pub struct Unit;

        delegate_components! {
            Unit {
                AreaCalculatorComponent: UnitArea,
            }
        }

        check_components! {
            Unit {
                AreaCalculatorComponent,
            }
        }
    }

    pub mod key_value_form {
        use cgp::prelude::*;

        #[cgp_component {
            name: AreaCalculatorComponent,
            provider: AreaCalculator,
            context: Context,
        }]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_impl(new UnitArea)]
        impl AreaCalculator {
            fn area(&self) -> f64 {
                1.0
            }
        }

        pub struct Unit;

        delegate_components! {
            Unit {
                AreaCalculatorComponent: UnitArea,
            }
        }

        check_components! {
            Unit {
                AreaCalculatorComponent,
            }
        }
    }
}

/// ## Companion attributes
///
/// The `#[use_type(HasErrorType.Error)]` component. The page shows only the trait, so a provider
/// and a context with a `String` error type are added to prove the bare `Error` resolves.
pub mod companion_attributes {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self, path: &str) -> Result<String, Error>;
    }

    #[cgp_impl(new LoadEcho)]
    #[use_type(HasErrorType.Error)]
    impl Loader {
        fn load(&self, path: &str) -> Result<String, Error> {
            Ok(format!("contents of {path}"))
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            LoaderComponent: LoadEcho,
        }
    }

    check_components! {
        App {
            LoaderComponent,
        }
    }

    #[test]
    fn the_component_loads() {
        assert_eq!(App.load("a.txt"), Ok("contents of a.txt".to_owned()));
    }
}

/// ## Examples
///
/// The page's example as written, including its `check_components!` block.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            width * height
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        Rectangle {
            AreaCalculatorComponent: RectangleArea,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorComponent,
        }
    }

    pub fn print_area(rect: &Rectangle) {
        println!("area = {}", rect.area());
    }

    #[test]
    fn the_wired_area_is_computed() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        print_area(&rect);
        assert_eq!(rect.area(), 6.0);
    }
}

/// ## Under the hood
///
/// The inputs whose expansions the page lists. The listings themselves are checked with
/// `cargo cgp expand --test reference_tests --item reference::macros::cgp_component::under_the_hood`,
/// not here; these modules prove the inputs compile and the described behavior holds.
pub mod under_the_hood {
    /// The parameterless component the listings expand.
    pub mod plain {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }
    }

    /// A local associated type: `Self::Output` stays as written and names the provider's own
    /// `Output`.
    pub mod local_associated_type {
        use cgp::prelude::*;

        #[cgp_component(OutputProducer)]
        pub trait CanProduceOutput {
            type Output;

            fn produce(&self) -> Self::Output;
        }

        #[cgp_impl(new ProduceNumber)]
        impl OutputProducer {
            type Output = u32;

            fn produce(&self) -> u32 {
                7
            }
        }

        pub struct App;

        delegate_components! {
            App {
                OutputProducerComponent: ProduceNumber,
            }
        }

        check_components! {
            App {
                OutputProducerComponent,
            }
        }

        #[test]
        fn the_provider_chooses_the_output() {
            assert_eq!(App.produce(), 7);
        }
    }

    /// `#[extend(HasName)]` on `CanGreet`, with the default body the page describes, and the empty
    /// `UseDefault` provider that inherits it. The page names `HasName` without declaring it; it is
    /// an auto getter here.
    pub mod supertrait_and_default_body {
        use cgp::core::component::UseDefault;
        use cgp::prelude::*;

        #[cgp_auto_getter]
        pub trait HasName {
            fn name(&self) -> &str;
        }

        #[cgp_component(Greeter)]
        #[extend(HasName)]
        pub trait CanGreet {
            fn greet(&self) -> String {
                format!("Hello, {}!", self.name())
            }
        }

        #[cgp_impl(UseDefault)]
        impl<Context: HasName> Greeter for Context {}

        #[derive(HasField)]
        pub struct Person {
            pub name: String,
        }

        delegate_components! {
            Person {
                GreeterComponent: UseDefault,
            }
        }

        check_components! {
            Person {
                GreeterComponent,
            }
        }

        #[test]
        fn the_empty_provider_inherits_the_default() {
            let person = Person {
                name: "World".to_owned(),
            };
            assert_eq!(person.greet(), "Hello, World!");
        }
    }

    /// The `context: Ctx` override, whose provider-trait receiver is `__ctx__`.
    pub mod context_override {
        use cgp::prelude::*;

        #[cgp_component { provider: Namer, context: Ctx }]
        pub trait CanName {
            fn name(&self) -> String;
        }
    }

    /// The components of the parameter table. `CanCalculateArea<Shape>` is wired per shape with
    /// `open`, which is the per-type path the page describes the `RedirectLookup` impl building.
    pub mod generic_components {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea<Shape> {
            fn area(&self, shape: &Shape) -> f64;
        }

        #[cgp_component(Encoder)]
        pub trait CanEncode<Value, Format> {
            fn encode(&self, value: &Value, format: &Format) -> Vec<u8>;
        }

        #[cgp_component(ReferenceGetter)]
        pub trait HasReference<'a, T> {
            fn reference(&self) -> &'a T;
        }

        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }

        #[cgp_impl(new RectangleArea)]
        impl AreaCalculator<Rectangle> {
            fn area(&self, shape: &Rectangle) -> f64 {
                shape.width * shape.height
            }
        }

        pub struct App;

        delegate_components! {
            App {
                open AreaCalculatorComponent;

                @AreaCalculatorComponent.Rectangle: RectangleArea,
            }
        }

        check_components! {
            App {
                AreaCalculatorComponent: Rectangle,
            }
        }

        #[test]
        fn the_per_type_entry_resolves() {
            let rect = Rectangle {
                width: 2.0,
                height: 3.0,
            };
            assert_eq!(App.area(&rect), 6.0);
        }
    }

    /// `#[track_caller]` on a component method reaches every provider, so a provider sees the
    /// consumer call's location.
    pub mod method_attributes {
        use core::panic::Location;

        use cgp::prelude::*;

        #[cgp_component(LineReporter)]
        pub trait CanReportLine {
            #[track_caller]
            fn report_line(&self) -> u32;
        }

        #[cgp_impl(new CallerLine)]
        impl LineReporter {
            fn report_line(&self) -> u32 {
                Location::caller().line()
            }
        }

        pub struct App;

        delegate_components! {
            App {
                LineReporterComponent: CallerLine,
            }
        }

        check_components! {
            App {
                LineReporterComponent,
            }
        }

        #[test]
        fn the_provider_reports_the_calling_line() {
            let expected = line!() + 1;
            let line = App.report_line();
            assert_eq!(line, expected);
        }
    }
}

/// ## Formal grammar
///
/// A component name carrying the trait's parameter, and the provider that names the component
/// explicitly because the default `{Provider}Component` carries no arguments. The page names
/// `Square` without declaring it.
pub mod formal_grammar {
    use cgp::prelude::*;

    #[cgp_component {
        provider: AreaCalculator,
        name: AreaCalculatorComponent<Shape>,
    }]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct Square {
        pub side: f64,
    }

    #[cgp_impl(new SquareArea: AreaCalculatorComponent<Square>)]
    impl AreaCalculator<Square> {
        fn area(&self, shape: &Square) -> f64 {
            shape.side * shape.side
        }
    }

    pub struct App;

    delegate_components! {
        App {
            AreaCalculatorComponent<Square>: SquareArea,
        }
    }

    check_components! {
        App {
            AreaCalculatorComponent<Square>: Square,
        }
    }

    #[test]
    fn the_parameterized_marker_wires() {
        assert_eq!(App.area(&Square { side: 3.0 }), 9.0);
    }
}
