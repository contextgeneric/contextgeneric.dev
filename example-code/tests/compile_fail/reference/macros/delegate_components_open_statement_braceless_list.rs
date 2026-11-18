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

// error: expected `;`, at the first comma
delegate_components! {
    App {
        open AreaCalculatorComponent, PerimeterCalculatorComponent;
    }
}

fn main() {}
