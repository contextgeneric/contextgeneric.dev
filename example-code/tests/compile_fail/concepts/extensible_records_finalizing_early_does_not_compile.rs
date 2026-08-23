use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct App {
    pub database: String,
    pub http_timeout: u32,
}

#[derive(CgpData)]
pub struct DatabaseConfig {
    pub database: String,
}

fn main() {
    // `http_timeout` has never been set, so there is no `finalize_build` to call.
    let _app: App = App::builder()
        .build_from(DatabaseConfig { database: "postgres://…".to_owned() })
        .finalize_build();
}
