use cgp::prelude::*;

#[cgp_component(FooProvider)]
pub trait Foo<T> {
    fn foo(&self, value: &T);
}

#[cgp_impl(new DummyFoo)]
impl<T> FooProvider<T> {
    fn foo(&self, _value: &T) {}
}

pub struct App;

// error: expected `:`
delegate_components! {
    App {
        open FooProviderComponent;

        @FooProviderComponent.{String, u32}.bool: DummyFoo,
    }
}

fn main() {}
