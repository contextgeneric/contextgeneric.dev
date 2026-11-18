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

#[cgp_impl(new ShapeArea)]
impl<Shape> AreaCalculator<Shape> {
    fn area(&self, _shape: &Shape) -> f64 {
        1.0
    }
}

pub struct MyApp;

// error[E0277]: `RectangleArea: IsProviderFor<AreaCalculatorComponent, MyApp>` is not satisfied
delegate_and_check_components! {
    MyApp {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn main() {}
