//! Code from `docs/reference/traits/namespace/default_namespace.md` — `DefaultNamespace`.
//!
//! Pins the Examples program: a component registered into `DefaultNamespace` with `#[prefix]`, and
//! a context joining the namespace and filling the paths it leaves open.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;
    use core::fmt::Display;

    #[cgp_component(ShowImpl)]
    #[prefix(@test in DefaultNamespace)]
    pub trait Show<T> {
        fn show(&self, value: &T) -> String;
    }

    #[cgp_impl(new ShowWithDisplay)]
    impl<T: Display> ShowImpl<T> {
        fn show(&self, value: &T) -> String {
            value.to_string()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace DefaultNamespace;

            // The namespace routes the component to this path and binds nothing there.
            @test.ShowImplComponent.u64: ShowWithDisplay,
        }
    }

    check_components! {
        App {
            ShowImplComponent: u64,
        }
    }

    pub fn demo() {
        assert_eq!(App.show(&5u64), "5");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
