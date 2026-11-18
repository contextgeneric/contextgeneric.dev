use cgp::prelude::*;

#[cgp_type]
pub trait HasFooType {
    type Foo;
}

#[cgp_type]
pub trait HasBarType {
    type Bar;
}

// error[E0275]: overflow evaluating the requirement `<__Context__ as HasFooType>::Foo == _`
#[cgp_fn]
#[use_type(HasFooType.{Foo = Bar}, HasBarType.{Bar = Foo})]
pub fn pick(&self, value: Foo) -> Bar {
    value
}

fn main() {}
