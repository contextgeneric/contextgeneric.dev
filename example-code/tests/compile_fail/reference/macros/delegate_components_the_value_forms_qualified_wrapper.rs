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

// error: expected `,`, at the inner table's name
delegate_components! {
    App {
        AreaCalculatorComponent: cgp::prelude::UseDelegate<new AreaCalculatorComponents {
            u32: RectangleArea,
        }>,
    }
}

fn main() {}
