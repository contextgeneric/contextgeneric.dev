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

cgp_namespace! {
    new AppNamespace {
        AreaCalculatorComponent: RectangleArea,
    }
}

pub struct App;

// error[E0119]: the namespace already binds `AreaCalculatorComponent`
delegate_components! {
    App {
        namespace AppNamespace;

        AreaCalculatorComponent: RectangleArea,
    }
}

fn main() {}
