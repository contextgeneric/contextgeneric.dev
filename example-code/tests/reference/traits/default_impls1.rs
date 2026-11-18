//! Code from `docs/reference/traits/namespace/default_impls1.md` — `DefaultImpls1`.
//!
//! Pins the Examples program: a provider registered as the `String` default with `#[default_impl]`,
//! a context pulling registered defaults in with a `for … in` loop, and a direct entry for a type the
//! registry does not cover. A loop whose key omits the loop variable is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::component::DefaultImpls1;
    use cgp::prelude::*;
    use core::fmt::Display;

    #[cgp_component(ShowImpl)]
    #[prefix(@test in DefaultNamespace)]
    pub trait Show<T> {
        fn show(&self, value: &T) -> String;
    }

    #[cgp_impl(new ShowString)]
    #[default_impl(String in DefaultImpls1<ShowImplComponent>)]
    impl ShowImpl<String> {
        fn show(&self, value: &String) -> String {
            format!("{value:?}")
        }
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

            for <T, Provider> in DefaultImpls1<ShowImplComponent> {
                @test.ShowImplComponent.T: Provider,
            }

            // `u64` has no registered default, so the context supplies one.
            @test.ShowImplComponent.u64: ShowWithDisplay,
        }
    }

    check_components! {
        App {
            ShowImplComponent: [String, u64],
        }
    }

    pub fn demo() {
        assert_eq!(App.show(&"hi".to_owned()), "\"hi\"");
        assert_eq!(App.show(&5u64), "5");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
