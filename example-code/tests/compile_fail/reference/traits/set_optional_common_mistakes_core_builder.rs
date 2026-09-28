use cgp::extra::field::impls::SetOptional;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

fn main() {
    // A core builder's fields are `IsNothing`, not `IsOptional`.
    let _ = Context::builder().set(PhantomData::<Symbol!("foo")>, "foo".to_owned());
}
