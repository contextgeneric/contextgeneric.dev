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

// error: expected `:`
#[cgp_fn]
#[use_provider(RectangleArea)]
pub fn rect_area(&self) -> f64 {
    RectangleArea::area(self)
}

fn main() {}
