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

// error: associated bindings (`Name = ...`) are not allowed in type arguments
#[cgp_impl(new Wrap<Inner>)]
#[use_provider(Inner: AreaCalculator<Output = f64>)]
impl<Inner> AreaCalculator {
    fn area(&self) -> f64 {
        Inner::area(self)
    }
}

fn main() {}
