use cgp::extra::field::impls::{CanFinalizeWithDefault, HasOptionalBuilder, SetOptional};
use cgp::prelude::*;

// `Port` has no `Default`. Setting `port` does not help: on an optional builder the field stays
// `IsOptional`, and `TransformMapDefault` converts `IsOptional` only for a `Default` type.
pub struct Port(pub u16);

#[derive(CgpData)]
pub struct Server {
    pub host: String,
    pub port: Port,
}

fn main() {
    let _ = Server::optional_builder()
        .set(PhantomData::<Symbol!("host")>, "localhost".to_owned())
        .set(PhantomData::<Symbol!("port")>, Port(8080))
        .finalize_with_default();
}
