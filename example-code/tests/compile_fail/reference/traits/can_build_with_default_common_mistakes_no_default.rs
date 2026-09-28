use cgp::extra::field::impls::CanBuildWithDefault;
use cgp::prelude::*;

// `Port` has no `Default`, and the source does not supply `port`.
pub struct Port(pub u16);

#[derive(CgpData)]
pub struct Host {
    pub host: String,
}

#[derive(CgpData)]
pub struct Server {
    pub host: String,
    pub port: Port,
}

fn main() {
    let _ = Server::build_with_default(Host {
        host: "localhost".to_owned(),
    });
}
