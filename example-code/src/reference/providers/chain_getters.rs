//! Code from `docs/reference/providers/chain_getters.md` — *`ChainGetters`*.
//!
//! Pins that a chain, wired as a getter provider through `WithProvider`, reaches a field on a nested
//! inner context: `App` holds a `Config`, and the getter reads the `Config`'s `name`.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::ChainGetters;
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[derive(HasField)]
    pub struct Config {
        pub name: String,
    }

    #[derive(HasField)]
    pub struct App {
        pub config: Config,
    }

    delegate_components! {
        App {
            NameGetterComponent: WithProvider<
                ChainGetters<Product![
                    UseField<Symbol!("config")>,
                    UseField<Symbol!("name")>,
                ]>,
            >,
        }
    }

    check_components! {
        App {
            NameGetterComponent,
        }
    }

    #[test]
    fn test_chain_getters_reaches_nested_field() {
        let app = App {
            config: Config {
                name: "test".to_owned(),
            },
        };
        assert_eq!(app.name(), "test");
    }
}
