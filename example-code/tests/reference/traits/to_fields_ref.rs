//! Code from `docs/reference/traits/shape/to_fields_ref.md` — `ToFieldsRef`.
//!
//! Pins the Examples program: reading a value's borrowed shape and using the value afterwards.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub struct Config {
        pub host: String,
        pub port: u16,
    }

    pub fn host_of(config: &Config) -> &String {
        config.to_fields_ref().0.value
    }

    pub fn demo() {
        let config = Config {
            host: "localhost".to_owned(),
            port: 8080,
        };

        assert_eq!(host_of(&config), "localhost");
        assert_eq!(config.port, 8080); // `config` is still usable
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
