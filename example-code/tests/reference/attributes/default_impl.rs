//! Code from `docs/reference/attributes/default_impl.md` — *`#[default_impl(...)]`*.

/// ## Examples
///
/// The page's chain: a per-type default registered on the provider and pulled into a context by a
/// `for … in` loop, with one direct override. `ShowWithDisplay` is named by the page without being
/// introduced, so it is declared here.
pub mod examples {
    use core::fmt::Display;

    use cgp::core::component::DefaultImpls1;
    use cgp::prelude::*;

    #[cgp_component(ShowImpl)]
    #[prefix(@test in DefaultNamespace)]
    pub trait Show<T> {
        fn show(&self, value: &T) -> String;
    }

    #[cgp_impl(new ShowString)]
    #[default_impl(String in DefaultImpls1<ShowImplComponent>)]
    impl ShowImpl<String> {
        fn show(&self, value: &String) -> String {
            value.clone()
        }
    }

    #[cgp_impl(new ShowWithDisplay)]
    impl<T: Display> ShowImpl<T> {
        fn show(&self, value: &T) -> String {
            format!("{value}")
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace DefaultNamespace;

            for <T, Provider> in DefaultImpls1<ShowImplComponent> {
                @test.ShowImplComponent.T: Provider,
            }

            @test.ShowImplComponent.u64: ShowWithDisplay,
        }
    }

    check_components! {
        App {
            ShowImplComponent: [String, u64],
        }
    }

    #[test]
    fn default_and_override_resolve() {
        assert_eq!(App.show(&"text".to_owned()), "text");
        assert_eq!(App.show(&7u64), "7");
    }
}

/// ## Usage: a path key
///
/// The key may be a `@`-path, which binds the provider at a prefixed component's own path so a
/// context that joins the namespace resolves it with no loop at all.
pub mod path_key {
    use cgp::prelude::*;

    cgp_namespace! {
        new AppNamespace: DefaultNamespace {}
    }

    #[cgp_component(Greeter)]
    #[prefix(@app in DefaultNamespace)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    #[default_impl(@app.GreeterComponent in AppNamespace)]
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
            namespace AppNamespace;
        }
    }

    check_components! {
        App {
            GreeterComponent,
        }
    }

    #[test]
    fn path_key_resolves_through_the_joined_namespace() {
        let app = App {
            name: "Alice".to_owned(),
        };
        assert_eq!(app.greet(), "Hello, Alice!");
    }
}

/// ## Usage: repeated attributes and a user-defined lookup trait
///
/// One provider registers into two tables, one attribute each, and the second table is a trait
/// of the reader's own with the lookup shape.
pub mod repeated {
    use cgp::core::component::DefaultImpls1;
    use cgp::prelude::*;

    #[cgp_component(ShowImpl)]
    #[prefix(@test in DefaultNamespace)]
    pub trait Show<T> {
        fn show(&self, value: &T) -> String;
    }

    pub trait LoudDefaults<Component, Components> {
        type Delegate;
    }

    #[cgp_impl(new ShowString)]
    #[default_impl(String in DefaultImpls1<ShowImplComponent>)]
    #[default_impl(String in LoudDefaults<ShowImplComponent>)]
    impl ShowImpl<String> {
        fn show(&self, value: &String) -> String {
            value.clone()
        }
    }

    pub struct Quiet;

    delegate_components! {
        Quiet {
            namespace DefaultNamespace;

            for <T, Provider> in DefaultImpls1<ShowImplComponent> {
                @test.ShowImplComponent.T: Provider,
            }
        }
    }

    pub struct Loud;

    delegate_components! {
        Loud {
            namespace DefaultNamespace;

            for <T, Provider> in LoudDefaults<ShowImplComponent> {
                @test.ShowImplComponent.T: Provider,
            }
        }
    }

    check_components! {
        Quiet {
            ShowImplComponent: String,
        }
    }

    check_components! {
        Loud {
            ShowImplComponent: String,
        }
    }

    #[test]
    fn both_tables_hold_the_registration() {
        assert_eq!(Quiet.show(&"a".to_owned()), "a");
        assert_eq!(Loud.show(&"b".to_owned()), "b");
    }
}

/// ## Under the hood
///
/// The path-key registration impl the page shows, written by hand in the resugared `Path!` spelling
/// against a component that carries no `#[default_impl]`, and checked to resolve the same way.
pub mod under_the_hood {
    use cgp::prelude::*;

    cgp_namespace! {
        new AppNamespace: DefaultNamespace {}
    }

    #[cgp_component(Greeter)]
    #[prefix(@app in DefaultNamespace)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello!".to_owned()
        }
    }

    impl<Components> AppNamespace<Components> for Path!(@app.GreeterComponent) {
        type Delegate = GreetHello;
    }

    pub struct App;

    delegate_components! {
        App {
            namespace AppNamespace;
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
