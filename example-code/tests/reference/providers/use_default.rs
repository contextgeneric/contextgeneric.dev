//! Code from `docs/reference/providers/use_default.md` — *`UseDefault`*.
//!
//! Pins that empty-body `#[cgp_impl(UseDefault)]` blocks make the trait's default method bodies the
//! implementation, and that both components can be delegated to `UseDefault` in one array entry. The
//! impl without `#[uses(HasName)]` from Common Mistakes is a trybuild fixture.

/// ## Usage
///
/// The two steps the section shows: an empty `#[cgp_impl(UseDefault)]` block, then the wiring entry.
pub mod usage {
    use cgp::core::component::UseDefault;
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String {
            "Hello!".to_owned()
        }
    }

    #[cgp_impl(UseDefault)]
    impl Greeter {}

    pub struct App;

    delegate_components! {
        App {
            GreeterComponent: UseDefault,
        }
    }

    check_components! {
        App {
            GreeterComponent,
        }
    }

    #[test]
    fn the_default_body_answers() {
        assert_eq!(App.greet(), "Hello!");
    }
}

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
    #[extend(HasName)]
    pub trait CanGreet {
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

    pub fn demo() {
        assert_eq!(App.greet(), "Hello, John!");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
