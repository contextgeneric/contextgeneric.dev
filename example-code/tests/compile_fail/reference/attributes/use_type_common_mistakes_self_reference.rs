use cgp::prelude::*;

#[cgp_type]
pub trait HasAType {
    type A;
}

#[cgp_type]
pub trait HasBType {
    type B;
}

// error: cannot ground `#[use_type]` imports: they resolve through one another in a cycle
#[cgp_fn]
#[use_type(HasAType.A in A)]
pub fn pick(&self, value: A) -> A {
    value
}

fn main() {}
