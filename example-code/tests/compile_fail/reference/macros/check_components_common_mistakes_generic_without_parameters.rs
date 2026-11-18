use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle;

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

// error[E0277]: `RectangleArea: IsProviderFor<AreaCalculatorComponent, MyApp>` is not satisfied
check_components! {
    MyApp {
        AreaCalculatorComponent,
    }
}

fn main() {}
