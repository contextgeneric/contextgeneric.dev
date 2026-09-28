use cgp::extra::dispatch::{BuildAndMerge, BuildWithHandlers};
use cgp::prelude::*;

#[derive(CgpData)]
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

// No step sets `user_agent`, so the builder never reaches the all-present state.
delegate_components! {
    AppBuilder {
        ComputerComponent:
            BuildWithHandlers<App, Product![
                BuildAndMerge<BuildDatabaseConfig>,
            ]>,
    }
}

check_components! {
    AppBuilder {
        ComputerComponent: ((), ()),
    }
}

fn main() {}
