//! Code from `docs/reference/traits/namespace/default_impls2.md` — `DefaultImpls2`.
//!
//! Pins the Examples program: a two-parameter component, a provider registered under a pair with
//! `#[default_impl]`, and a context pulling every default for one fixed target in with a loop.

/// ## Examples
pub mod examples {
    use cgp::core::component::DefaultImpls2;
    use cgp::prelude::*;

    #[cgp_component(Converter)]
    #[prefix(@test in DefaultNamespace)]
    pub trait CanConvert<Source, Target> {
        fn convert(&self, value: &Source) -> Target;
    }

    #[cgp_impl(new ParseU64)]
    #[default_impl(String in DefaultImpls2<ConverterComponent, u64>)]
    impl Converter<String, u64> {
        fn convert(&self, value: &String) -> u64 {
            value.parse().unwrap_or_default()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace DefaultNamespace;

            // Every source type with a registered default into `u64`.
            for <T, Provider> in DefaultImpls2<ConverterComponent, u64> {
                @test.ConverterComponent.T.u64: Provider,
            }
        }
    }

    check_components! {
        App {
            ConverterComponent: (String, u64),
        }
    }

    pub fn demo() {
        let parsed: u64 = App.convert(&"42".to_owned());
        assert_eq!(parsed, 42);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
