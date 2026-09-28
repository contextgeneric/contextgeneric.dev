//! Code from `docs/reference/providers/with_field_ref.md` — *`WithFieldRef`*.
//!
//! `WithFieldRef` (`WithProvider<UseFieldRef<..>>`) is the form that wires the foundational
//! `UseFieldRef` getter. This pins that a `-> &Config` getter reads a stored `AsRef<Config>` field,
//! and that the borrowed views When to use it names need only the plain `UseField`. The unsized
//! `str` value from Common Mistakes is a trybuild fixture.

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

    pub fn demo() {
        let app = App { config: StoredConfig(Config { port: 8080 }) };
        assert_eq!(app.config().port, 8080);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## When to use it
///
/// The borrowed views the plain `UseField` already serves: `&[u8]` over `Vec<u8>`, and `Option<&T>`
/// and `Option<&str>` over `Option<T>` and `Option<String>`.
pub mod when_to_use_it {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasBody {
        fn body(&self) -> &[u8];
    }

    #[cgp_getter]
    pub trait HasNick {
        fn nick(&self) -> Option<&str>;
    }

    #[cgp_getter]
    pub trait HasAge {
        fn age(&self) -> Option<&u8>;
    }

    #[derive(HasField)]
    pub struct Person {
        pub payload: Vec<u8>,
        pub nickname: Option<String>,
        pub years: Option<u8>,
    }

    delegate_components! {
        Person {
            BodyGetterComponent: UseField<Symbol!("payload")>,
            NickGetterComponent: UseField<Symbol!("nickname")>,
            AgeGetterComponent: UseField<Symbol!("years")>,
        }
    }

    check_components! {
        Person {
            BodyGetterComponent,
            NickGetterComponent,
            AgeGetterComponent,
        }
    }

    #[test]
    fn the_shorthands_borrow_without_with_field_ref() {
        let person = Person {
            payload: vec![1, 2],
            nickname: Some("Al".to_owned()),
            years: None,
        };

        assert_eq!(person.body(), &[1, 2]);
        assert_eq!(person.nick(), Some("Al"));
        assert_eq!(person.age(), None);
    }
}
