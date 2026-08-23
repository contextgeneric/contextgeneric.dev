use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 { width * height }
}

// error[E0277]: the trait bound
//              `GeometryComponents: CanUseComponent<AreaCalculatorComponent>`
//              is not satisfied
delegate_and_check_components! {
    new GeometryComponents {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn main() {}
