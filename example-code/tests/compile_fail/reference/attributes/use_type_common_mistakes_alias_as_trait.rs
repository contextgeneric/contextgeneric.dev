use cgp::prelude::*;

#[cgp_type]
pub trait HasFooType {
    type Foo;
}

#[cgp_type]
pub trait HasBarType {
    type Bar;
}

// error[E0405]: cannot find trait `Foo` in this scope
#[cgp_fn]
#[use_type(HasFooType.Foo, Foo.Bar)]
pub fn pick(&self, value: Bar) -> Bar {
    value
}

fn main() {}
