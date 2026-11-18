use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle;
pub struct Circle;

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<Rectangle> {
    fn area(&self, _shape: &Rectangle) -> f64 {
        1.0
    }
}

pub struct MyApp;

delegate_components! {
    MyApp {
        AreaCalculatorComponent: RectangleArea,
    }
}

// error[E0277]: the caret lands on `Circle`, the parameter that failed
check_components! {
    MyApp {
        AreaCalculatorComponent: [Rectangle, Circle],
    }
}

fn main() {}
