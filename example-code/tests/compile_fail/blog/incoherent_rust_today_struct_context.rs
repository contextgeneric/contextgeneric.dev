// `blog/2026-11-25-incoherent-rust-today.md`, "Use of incoherent trait bounds within a struct":
// a `Foo` built for `ContextA` cannot be passed to `ContextB`'s `do_stuff`.
use cgp::prelude::*;
use core::marker::PhantomData;

#[cgp_component(NameImpl)]
pub trait Name<T> {
    fn name() -> &'static str;
}

#[cgp_impl(new DummyName)]
impl<T> NameImpl<T> {
    fn name() -> &'static str {
        "dummy"
    }
}

#[cgp_impl(new RealName)]
impl<T> NameImpl<T> {
    fn name() -> &'static str {
        core::any::type_name::<T>()
    }
}

pub struct Foo<Context: ?Sized, T> {
    pub value: T,
    pub phantom: PhantomData<Context>,
}

#[cgp_fn]
#[uses(Name<T>)]
pub fn do_stuff<T>(&self, foo: &Foo<Self, T>) {
    println!("{}", Self::name());
}

struct ContextA;

delegate_components! {
    ContextA {
        NameImplComponent: DummyName,
    }
}

struct ContextB;

delegate_components! {
    ContextB {
        NameImplComponent: RealName,
    }
}

fn main() {
    let foo: Foo<ContextA, i32> = Foo {
        value: 1,
        phantom: PhantomData,
    };

    ContextA.do_stuff(&foo);

    // type error: foo has type Foo<ContextA, i32> but ContextB expects Foo<ContextB, i32>
    ContextB.do_stuff(&foo);
}
