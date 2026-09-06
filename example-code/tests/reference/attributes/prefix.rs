//! Code from `docs/reference/attributes/prefix.md` — *`#[prefix(...)]`*.

/// ## Overview
///
/// The opening registration. The page names `AppNamespace` without defining it, so it is defined
/// here, and a context joins it and binds the path so the route resolves.
pub mod overview {
    use cgp::prelude::*;

    cgp_namespace! {
        new AppNamespace {}
    }

    #[cgp_component(Greeter)]
    #[prefix(@app in AppNamespace)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello!".to_owned()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace AppNamespace;

            @app.GreeterComponent: GreetHello,
        }
    }

    check_components! {
        App {
            GreeterComponent,
        }
    }

    #[test]
    fn resolves_through_the_prefix() {
        assert_eq!(App.greet(), "Hello!");
    }
}

/// ## Usage
///
/// The two-namespace registration, and a generic component addressed with its parameter after
/// the marker.
pub mod usage {
    use core::fmt::Display;

    use cgp::prelude::*;

    cgp_namespace! {
        new MyNamespace {}
    }

    cgp_namespace! {
        new OtherNamespace {}
    }

    pub struct MyApp;

    pub struct MyBarComponent;

    #[cgp_component(BarProvider)]
    #[prefix(@MyApp.MyBarComponent in MyNamespace)]
    #[prefix(@my_app.MyBarComponent in OtherNamespace)]
    pub trait Bar {
        fn bar(&self);
    }

    #[cgp_impl(new DummyBar)]
    impl BarProvider {
        fn bar(&self) {}
    }

    pub struct AppA;

    delegate_components! {
        AppA {
            namespace MyNamespace;

            @MyApp.MyBarComponent.BarProviderComponent: DummyBar,
        }
    }

    pub struct AppB;

    delegate_components! {
        AppB {
            namespace OtherNamespace;

            @my_app.MyBarComponent.BarProviderComponent: DummyBar,
        }
    }

    check_components! {
        AppA {
            BarProviderComponent,
        }
    }

    check_components! {
        AppB {
            BarProviderComponent,
        }
    }

    #[cgp_component(ShowImpl)]
    #[prefix(@app in MyNamespace)]
    pub trait CanShow<T> {
        fn show(&self, value: &T) -> String;
    }

    #[cgp_impl(new ShowWithDisplay)]
    impl<T: Display> ShowImpl<T> {
        fn show(&self, value: &T) -> String {
            format!("{value}")
        }
    }

    pub struct AppC;

    delegate_components! {
        AppC {
            namespace MyNamespace;

            @app.ShowImplComponent.String: ShowWithDisplay,
        }
    }

    check_components! {
        AppC {
            ShowImplComponent: String,
        }
    }

    #[test]
    fn shows_by_prefixed_path() {
        AppA.bar();
        AppB.bar();
        assert_eq!(AppC.show(&"hi".to_owned()), "hi");
    }
}

/// ## Examples
pub mod examples {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::ReturnError;
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    #[prefix(@app in DefaultNamespace)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self, #[implicit] name: &str) -> String {
            format!("Hello, {name}!")
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
    }

    delegate_components! {
        App {
            namespace DefaultNamespace;

            @app.GreeterComponent: GreetHello,
        }
    }

    check_components! {
        App {
            GreeterComponent,
        }
    }

    #[cgp_impl(new GreetFormally)]
    impl Greeter {
        fn greet(&self, #[implicit] name: &str) -> String {
            format!("Good day, {name}.")
        }
    }

    #[derive(HasField)]
    pub struct FormalApp {
        pub name: String,
    }

    delegate_components! {
        FormalApp {
            namespace DefaultNamespace;

            @app.GreeterComponent: GreetFormally,
        }
    }

    check_components! {
        FormalApp {
            GreeterComponent,
        }
    }

    pub struct Service;

    delegate_components! {
        Service {
            namespace DefaultNamespace;

            @cgp.core.error.ErrorTypeProviderComponent: UseType<String>,
            @cgp.core.error.ErrorRaiserComponent.String: ReturnError,
        }
    }

    check_components! {
        Service {
            ErrorTypeProviderComponent,
            ErrorRaiserComponent: String,
        }
    }

    #[test]
    fn two_contexts_bind_the_same_path_differently() {
        let app = App {
            name: "Alice".to_owned(),
        };
        let formal = FormalApp {
            name: "Alice".to_owned(),
        };
        assert_eq!(app.greet(), "Hello, Alice!");
        assert_eq!(formal.greet(), "Good day, Alice.");
    }
}

/// ## Under the hood
///
/// The impl the attribute emits, written by hand for an unprefixed component, resolves the same
/// way as the attribute's own output.
pub mod under_the_hood {
    use cgp::prelude::*;

    cgp_namespace! {
        new AppNamespace {}
    }

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    impl<__Components__> AppNamespace<__Components__> for GreeterComponent {
        type Delegate = RedirectLookup<__Components__, Path!(@app.GreeterComponent)>;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello!".to_owned()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace AppNamespace;

            @app.GreeterComponent: GreetHello,
        }
    }

    check_components! {
        App {
            GreeterComponent,
        }
    }

    #[test]
    fn hand_written_registration_resolves() {
        assert_eq!(App.greet(), "Hello!");
    }
}
