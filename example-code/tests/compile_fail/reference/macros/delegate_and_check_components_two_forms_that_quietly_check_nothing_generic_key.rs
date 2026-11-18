use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component {
    name: FooKeyComponent<I>,
    provider: FooProvider,
}]
pub trait CanFoo<I> {
    fn foo(&self, tag: PhantomData<I>) -> u8;
}

#[cgp_impl(new AnyFoo: FooKeyComponent<I>)]
impl<I> FooProvider<I> {
    fn foo(&self, _tag: PhantomData<I>) -> u8 {
        0
    }
}

pub struct App;

// error[E0277]: the derived check tests `FooKeyComponent<I>` at unit parameters
delegate_and_check_components! {
    App {
        <I> FooKeyComponent<I>: AnyFoo,
    }
}

fn main() {}
