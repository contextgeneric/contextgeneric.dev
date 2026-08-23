//! Code from `docs/reference/providers/use_default.md` — *`UseDefault`*.
//!
//! Pins that empty-body `#[cgp_impl(UseDefault)]` blocks make the trait's default method bodies the
//! implementation, and that both components can be delegated to `UseDefault` in one array entry.

/// ## Examples
pub mod examples {
    use cgp::core::component::UseDefault;
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str {
            "John"
        }
    }

    #[cgp_component(Greeter)]
    pub trait CanGreet: HasName {
        fn greet(&self) -> String {
            format!("Hello, {}!", self.name())
        }
    }

    #[cgp_impl(UseDefault)]
    impl NameGetter {}

    #[cgp_impl(UseDefault)]
    #[uses(HasName)]
    impl Greeter {}

    pub struct App;

    delegate_components! {
        App {
            [
                NameGetterComponent,
                GreeterComponent,
            ]:
                UseDefault,
        }
    }

    check_components! {
        App {
            NameGetterComponent,
            GreeterComponent,
        }
    }

    #[test]
    fn test_use_default() {
        let app = App;
        assert_eq!(app.greet(), "Hello, John!");
    }
}
