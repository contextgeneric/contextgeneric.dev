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

pub struct App;

pub struct ArrayKey<const N: usize>;

// error[E0747]: the bare `N` declares a type parameter, so the const argument does not fit it
delegate_components! {
    new MyComponents {
        <const N: usize> ArrayKey<N>: UseDelegate<new ArrayTable<N> {
            u32: RectangleArea,
        }>,
    }
}

fn main() {}
