use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        1.0
    }
}

// error[E0747]: `new` reads `N` as a type parameter of the declared struct
delegate_components! {
    <const N: usize> new ArrayTable<N> {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn main() {}
