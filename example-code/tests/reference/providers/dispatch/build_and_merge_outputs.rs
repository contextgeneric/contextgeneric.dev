//! Code from `docs/reference/providers/dispatch/build_and_merge_outputs.md` — `BuildAndMergeOutputs`.
//!
//! Pins the Examples program: the same two sub-record providers as the `BuildAndMerge` page, listed
//! bare, with `BuildAndMergeOutputs` adding the merge step to each. Wiring a `…Ref` component to it
//! is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::extra::dispatch::BuildAndMergeOutputs;
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

    #[derive(CgpData)]
    pub struct HttpConfig {
        pub user_agent: String,
    }

    #[cgp_impl(new BuildHttpConfig)]
    impl<Code, Input> Computer<Code, Input> {
        type Output = HttpConfig;

        fn compute(
            &self,
            _code: PhantomData<Code>,
            _input: Input,
            #[implicit] agent_name: &str,
        ) -> HttpConfig {
            HttpConfig {
                user_agent: format!("{agent_name}/1.0"),
            }
        }
    }

    delegate_components! {
        AppBuilder {
            ComputerComponent:
                BuildAndMergeOutputs<App, Product![BuildDatabaseConfig, BuildHttpConfig]>,
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
