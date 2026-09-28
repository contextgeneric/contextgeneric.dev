use cgp::extra::field::impls::*;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

fn main() {
    // A core builder's fields are `IsNothing`, not `IsOptional`.
    let _ = Context::builder().finalize_optional();
}
