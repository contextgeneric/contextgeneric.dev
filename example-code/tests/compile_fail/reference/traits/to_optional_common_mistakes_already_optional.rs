use cgp::extra::field::impls::*;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

fn main() {
    // The builder is already optional, and `TransformOptional` has no `IsOptional` source impl.
    let _ = Context::optional_builder().to_optional();
}
