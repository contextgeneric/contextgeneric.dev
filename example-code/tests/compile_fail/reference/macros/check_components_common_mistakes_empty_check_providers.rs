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

pub struct App;

// error: `#[check_providers(...)]` requires at least one provider type.
check_components! {
    #[check_providers()]
    App {
        AreaCalculatorComponent,
    }
}

fn main() {}
