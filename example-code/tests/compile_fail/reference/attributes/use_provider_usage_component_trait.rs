use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(PerimeterCalculator)]
pub trait CanCalculatePerimeter {
    fn perimeter(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        1.0
    }
}

// error: cannot find attribute `use_provider` in this scope
#[cgp_component(Shouter)]
#[use_provider(RectangleArea: AreaCalculator)]
pub trait CanShout {
    fn shout(&self) -> String;
}

fn main() {}
