use cgp::prelude::*;

#[cgp_type]
pub trait HasFooType {
    type Foo;
}

#[cgp_type]
pub trait HasBarType {
    type Bar;
}

// error[E0425]: cannot find type `Foo` in this scope
#[cgp_fn]
#[use_type(HasFooType.{Foo = Foo})]
pub fn pick(&self, value: Foo) -> Foo {
    value
}

fn main() {}
