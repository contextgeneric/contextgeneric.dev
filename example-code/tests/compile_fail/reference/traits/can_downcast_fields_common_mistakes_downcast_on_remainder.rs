use cgp::core::field::impls::CanDowncast;
use cgp::prelude::*;

#[derive(CgpData)]
pub enum FooBarBaz {
    Foo(u64),
    Bar(String),
    Baz(bool),
}

#[derive(CgpData)]
pub enum JustFoo {
    Foo(u64),
}

#[derive(CgpData)]
pub enum JustBar {
    Bar(String),
}

fn main() {
    let value = FooBarBaz::Bar("hi".to_owned());
    if let Err(remainder) = value.downcast(PhantomData::<JustFoo>) {
        // A remainder is an extractor, not an enum, so it takes `downcast_fields`.
        let _ = remainder.downcast(PhantomData::<JustBar>);
    }
}
