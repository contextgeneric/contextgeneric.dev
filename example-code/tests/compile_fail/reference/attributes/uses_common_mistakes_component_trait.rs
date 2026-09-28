use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

// error: cannot find attribute `uses` in this scope
#[cgp_component(Shouter)]
#[uses(CanCalculateArea)]
pub trait CanShout {
    fn shout(&self) -> String;
}

fn main() {}
