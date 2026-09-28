use cgp::extra::dispatch::BuildAndMergeOutputs;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct App {
    pub db_url: String,
    pub user_agent: String,
}

#[derive(CgpData)]
pub struct DatabaseConfig {
    pub db_url: String,
}

#[derive(CgpData)]
pub struct HttpConfig {
    pub user_agent: String,
}

#[cgp_producer]
fn build_database_config() -> DatabaseConfig {
    DatabaseConfig {
        db_url: "sqlite://app.db".to_owned(),
    }
}

#[cgp_producer]
fn build_http_config() -> HttpConfig {
    HttpConfig {
        user_agent: "reader/1.0".to_owned(),
    }
}

pub struct AppBuilder;

// The table routes `ComputerRefComponent`, but `BuildWithHandlers` has no `ComputerRef` impl.
delegate_components! {
    AppBuilder {
        ComputerRefComponent:
            BuildAndMergeOutputs<App, Product![BuildDatabaseConfig, BuildHttpConfig]>,
    }
}

check_components! {
    AppBuilder {
        ComputerRefComponent: ((), ()),
    }
}

fn main() {}
