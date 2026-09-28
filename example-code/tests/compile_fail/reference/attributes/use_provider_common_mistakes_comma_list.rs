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

// error: expected `+`
#[cgp_impl(new Both<A, P>)]
#[use_provider(A: AreaCalculator, P: PerimeterCalculator)]
impl<A, P> AreaCalculator {
    fn area(&self) -> f64 {
        A::area(self) + P::perimeter(self)
    }
}

fn main() {}
