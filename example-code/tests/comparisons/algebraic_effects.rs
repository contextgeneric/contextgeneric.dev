//! Code from `docs/comparisons/algebraic-effects.md` — *Algebraic effects and handlers*.

/// ## Components are effect signatures; providers are tail-resumptive handlers
pub mod components_are_effect_signatures {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self);
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) {
            println!("Hello!");
        }
    }

    pub struct App;

    delegate_components! { App { GreeterComponent: GreetHello } }

    mod check_app {
        use super::*;
        check_components! { App { GreeterComponent } }
    }
}

/// ## Reading from the context is dynamic binding, exactly
pub mod reading_from_the_context_is_dynamic_binding {
    use cgp::prelude::*;

    #[cgp_fn]
    pub fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
    }

    #[test]
    fn the_context_supplies_the_value() {
        let app = App {
            name: "World".to_owned(),
        };
        assert_eq!(app.greet(), "Hello, World!");
    }
}

/// ## Raising an error looks like `raise` but passes a value, not control
/// ## Impl-side dependencies are the effect row; `check_components!` is "all effects handled"
pub mod raising_an_error {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self, path: &str) -> Result<String, Error>;
    }

    #[cgp_impl(new LoadOrFail)]
    #[uses(CanRaiseError<String>)]
    #[use_type(HasErrorType.Error)]
    impl Loader {
        fn load(&self, path: &str) -> Result<String, Error> {
            if path.is_empty() {
                return Err(Self::raise_error("empty path".to_owned()));
            }
            Ok(format!("contents of {path}"))
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            LoaderComponent: LoadOrFail,

            @ErrorRaiserComponent.String: RaiseFrom,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                LoaderComponent,
            }
        }
    }

    #[test]
    fn raise_error_returns_a_value_and_the_question_mark_does_the_abort() {
        assert_eq!(App.load("config.toml").unwrap(), "contents of config.toml");
        assert_eq!(App.load("").unwrap_err(), "empty path");
    }
}

/// ## Configuring abstract types through the same wiring
pub mod configuring_abstract_types {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<anyhow::Error>,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { ErrorTypeProviderComponent } }
    }

    #[test]
    fn the_context_fixes_the_type() {
        fn assert_error_type<Context: HasErrorType<Error = anyhow::Error>>() {}
        assert_error_type::<App>();
    }
}
