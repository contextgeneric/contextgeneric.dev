use cgp::extra::field::impls::CanFinalizeWithDefault;
use cgp::prelude::*;

// `Port` has no `Default`, so an unset `port` cannot be filled.
pub struct Port(pub u16);

#[derive(CgpData)]
pub struct Server {
    pub host: String,
    pub port: Port,
}

fn main() {
    let _ = Server::builder()
        .build_field(PhantomData::<Symbol!("host")>, "localhost".to_owned())
        .finalize_with_default();
}
