use cgp::prelude::*;

#[cgp_type]
pub trait HasFooType {
    type Foo;
}

#[cgp_type]
pub trait HasBarType {
    type Bar;
}

// error: Multiple abstract types cannot share the same identifier or alias
#[cgp_fn]
#[use_type(HasFooType.Foo, HasBarType.Bar as Foo)]
pub fn pick(&self, value: Foo) -> Foo {
    value
}

fn main() {}
