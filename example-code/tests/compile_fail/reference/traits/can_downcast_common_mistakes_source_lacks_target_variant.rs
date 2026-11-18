use cgp::core::field::impls::CanDowncast;
use cgp::prelude::*;

#[derive(CgpData)]
pub enum FooBar {
    Foo(u64),
    Bar(String),
}

// `FooQux` has a `Qux` that `FooBar` can never hold.
#[derive(CgpData)]
pub enum FooQux {
    Foo(u64),
    Qux(char),
}

fn main() {
    let _ = FooBar::Foo(1).downcast(PhantomData::<FooQux>);
}
