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

// error[E0119]: the merged lists name `Rectangle` twice, so its check impl is emitted twice
delegate_and_check_components! {
    MyApp {
        #[check_params(Rectangle)]
        [
            #[check_params(Rectangle, Circle)]
            AreaCalculatorComponent,
        ]: ShapeArea,
    }
}

fn main() {}
