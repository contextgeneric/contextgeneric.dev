//! Code from `docs/reference/providers/dispatch/build_with_handlers.md` — `BuildWithHandlers`.
//!
//! Pins the Examples program: a builder context assembles an `App` from a merged sub-record and one
//! computed field, each provider reading the configuration it needs from the context. A list that
//! leaves a field unset is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::extra::dispatch::{BuildAndMerge, BuildAndSetField, BuildWithHandlers};
    use cgp::extra::handler::CanCompute;
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub struct App {
        pub db_url: String,
        pub max_connections: u32,
        pub user_agent: String,
    }

    #[derive(CgpData)]
    pub struct DatabaseConfig {
        pub db_url: String,
        pub max_connections: u32,
    }

    #[derive(HasField)]
    pub struct AppBuilder {
        pub db_path: String,
        pub agent_name: String,
    }

    #[cgp_impl(new BuildDatabaseConfig)]
    impl<Code, Input> Computer<Code, Input> {
        type Output = DatabaseConfig;

        fn compute(
            &self,
            _code: PhantomData<Code>,
            _input: Input,
            #[implicit] db_path: &str,
        ) -> DatabaseConfig {
            DatabaseConfig {
                db_url: format!("sqlite://{db_path}"),
                max_connections: 4,
            }
        }
    }

    #[cgp_impl(new BuildUserAgent)]
    impl<Code, Input> Computer<Code, Input> {
        type Output = String;

        fn compute(
            &self,
            _code: PhantomData<Code>,
            _input: Input,
            #[implicit] agent_name: &str,
        ) -> String {
            format!("{agent_name}/1.0")
        }
    }

    delegate_components! {
        AppBuilder {
            ComputerComponent:
                BuildWithHandlers<App, Product![
                    BuildAndMerge<BuildDatabaseConfig>,
                    BuildAndSetField<Symbol!("user_agent"), BuildUserAgent>,
                ]>,
        }
    }

    check_components! {
        AppBuilder {
            ComputerComponent: ((), ()),
        }
    }

    pub fn demo() {
        let builder = AppBuilder {
            db_path: "app.db".to_owned(),
            agent_name: "reader".to_owned(),
        };

        assert_eq!(
            builder.compute(PhantomData::<()>, ()),
            App {
                db_url: "sqlite://app.db".to_owned(),
                max_connections: 4,
                user_agent: "reader/1.0".to_owned(),
            },
        );
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
