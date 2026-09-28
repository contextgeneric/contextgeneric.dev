use cgp::prelude::*;

#[cgp_type]
pub trait HasFooType {
    type Foo;
}

#[cgp_type]
pub trait HasBarType {
    type Bar;
}

// error[E0425]: cannot find type `Output` in this scope
#[cgp_component(Maker)]
pub trait CanMake {
    type Output;
    fn make(&self) -> Output;
}

fn main() {}
