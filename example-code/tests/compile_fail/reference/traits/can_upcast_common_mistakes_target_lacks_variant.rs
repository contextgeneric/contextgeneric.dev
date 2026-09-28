use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;

#[derive(CgpData)]
pub enum FooBar {
    Foo(u64),
    Bar(String),
}

// `FooBaz` has no `Bar`, so it is not wider than `FooBar`.
#[derive(CgpData)]
pub enum FooBaz {
    Foo(u64),
    Baz(bool),
}

fn main() {
    let _ = FooBar::Foo(1).upcast(PhantomData::<FooBaz>);
}
