//! Code from `docs/concepts/namespaces.md` — *Namespaces*.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_component(Farewell)]
pub trait CanSayGoodbye {
    fn goodbye(&self) -> String;
}

#[cgp_component(Announcer)]
pub trait CanAnnounce {
    fn announce(&self) -> String;
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

#[cgp_impl(new AnnounceLoudly)]
impl Announcer {
    fn announce(&self) -> String {
        "NOW SERVING".to_owned()
    }
}

/// ## Repeating a table across contexts
///
/// The problem the rest of the page is about: two contexts wanting almost the same set of choices,
/// with nothing in the code to say so.
pub mod repeating_a_table {
    use cgp::prelude::*;

    use super::*;

    pub struct App;
    pub struct TestApp;

    delegate_components! {
        App {
            GreeterComponent: GreetHello,
            FarewellComponent: SayGoodbye,
            AnnouncerComponent: AnnounceLoudly,
        }
    }

    delegate_components! {
        TestApp {
            GreeterComponent: GreetQuietly,
            FarewellComponent: SayGoodbye,
            AnnouncerComponent: AnnounceLoudly,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                GreeterComponent,
                FarewellComponent,
                AnnouncerComponent,
            }
        }
    }
}

/// ## Routing a component through a path
///
/// A namespace resolves *paths* rather than bare component names, and a component registers itself
/// at one. Nothing is decided yet: the path is a route, and something further along supplies the
/// provider.
pub mod routing_a_component_through_a_path {
    use cgp::prelude::*;

    cgp_namespace! { new AppNamespace {} }

    #[cgp_component(Greeter)]
    #[prefix(@app.GreeterComponent in AppNamespace)]
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

    mod check_app {
        use super::*;
        check_components! { App { GreeterComponent } }
    }

    #[test]
    fn the_lookup_walks_the_path() {
        assert_eq!(App.greet(), "Hello!");
    }
}

/// ## Binding what is shared, leaving open what varies
///
/// The preset shape. `AppDefaults` binds the leaf every context agrees on and leaves the varying
/// one unbound, so a context supplies one entry and inherits the rest.
pub mod binding_what_is_shared {
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

    cgp_namespace! {
        new AppDefaults: AppNamespace {
            @app.FarewellComponent: SayGoodbye,
        }
    }

    pub struct App;
    pub struct TestApp;

    delegate_components! {
        App {
            namespace AppDefaults;

            @app.GreeterComponent: GreetHello,
        }
    }

    delegate_components! {
        TestApp {
            namespace AppDefaults;

            @app.GreeterComponent: GreetQuietly,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { GreeterComponent, FarewellComponent } }
    }

    mod check_test_app {
        use super::*;
        check_components! { TestApp { GreeterComponent, FarewellComponent } }
    }

    #[test]
    fn the_contexts_differ_by_one_line() {
        assert_eq!(App.greet(), "Hello!");
        assert_eq!(TestApp.greet(), "hello");

        assert_eq!(App.goodbye(), "Goodbye!");
        assert_eq!(TestApp.goodbye(), "Goodbye!");
    }
}

/// ## A bound entry is not overridable
///
/// The rule that decides how a namespace is designed, and the error it produces when broken. Both
/// blocks are rejected.
///
/// A context re-wiring a key its namespace binds:
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/namespaces_a_bound_entry_is_not_overridable_1.rs`.
///
/// And a child namespace redefining a key it inherits:
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/namespaces_a_bound_entry_is_not_overridable_2.rs`.
pub mod a_bound_entry_is_not_overridable {}

/// ## One namespace per configuration
///
/// The arrangement the rule above points to: the base describes the structure and binds nothing
/// that varies, and each inheriting namespace binds one configuration.
pub mod one_namespace_per_configuration {
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

    // Shared structure. The greeter path is deliberately left open.
    cgp_namespace! {
        new AppDefaults: AppNamespace {
            @app.FarewellComponent: SayGoodbye,
        }
    }

    // One configuration each, binding the open path rather than overriding a bound one.
    cgp_namespace! {
        new ProductionDefaults: AppDefaults {
            @app.GreeterComponent: GreetHello,
        }
    }

    cgp_namespace! {
        new TestDefaults: AppDefaults {
            @app.GreeterComponent: GreetQuietly,
        }
    }

    pub struct App;
    pub struct TestApp;

    delegate_components! {
        App {
            namespace ProductionDefaults;
        }
    }

    delegate_components! {
        TestApp {
            namespace TestDefaults;
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { GreeterComponent, FarewellComponent } }
    }

    mod check_test_app {
        use super::*;
        check_components! { TestApp { GreeterComponent, FarewellComponent } }
    }

    #[test]
    fn a_context_is_now_one_line() {
        assert_eq!(App.greet(), "Hello!");
        assert_eq!(TestApp.greet(), "hello");
        assert_eq!(App.goodbye(), "Goodbye!");
    }
}
