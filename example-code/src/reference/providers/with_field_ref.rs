//! Code from `docs/reference/providers/with_field_ref.md` — *`WithFieldRef`*.
//!
//! `WithFieldRef` (`WithProvider<UseFieldRef<..>>`) is the form that wires the foundational
//! `UseFieldRef` getter. This pins that a `-> &Config` getter reads a stored `AsRef<Config>` field.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::WithFieldRef;
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq)]
    pub struct Config {
        pub port: u16,
    }

    // A stored wrapper that borrows as `Config`.
    pub struct StoredConfig(pub Config);

    impl AsRef<Config> for StoredConfig {
        fn as_ref(&self) -> &Config {
            &self.0
        }
    }

    #[cgp_getter]
    pub trait HasConfig {
        fn config(&self) -> &Config;
    }

    #[derive(HasField)]
    pub struct App {
        pub config: StoredConfig,
    }

    delegate_components! {
        App {
            ConfigGetterComponent: WithFieldRef<Symbol!("config"), Config>,
        }
    }

    check_components! {
        App {
            ConfigGetterComponent,
        }
    }

    #[test]
    fn test_with_field_ref_borrows_through_as_ref() {
        let app = App {
            config: StoredConfig(Config { port: 8080 }),
        };
        assert_eq!(app.config(), &Config { port: 8080 });
    }
}
