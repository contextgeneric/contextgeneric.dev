use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;

#[derive(CgpData)]
pub enum Foo64 {
    Foo(u64),
}

// The variant name matches, but the payload type does not.
#[derive(CgpData)]
pub enum Foo32Bar {
    Foo(u32),
    Bar(String),
}

fn main() {
    let _ = Foo64::Foo(1).upcast(PhantomData::<Foo32Bar>);
}
