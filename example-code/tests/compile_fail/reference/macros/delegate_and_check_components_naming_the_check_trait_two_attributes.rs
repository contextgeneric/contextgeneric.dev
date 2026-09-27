use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

// error: Expected exactly one attribute for the check trait name
delegate_and_check_components! {
    #[check_trait(CheckRectangle)]
    #[check_trait(CheckRectangleAgain)]
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn main() {}
