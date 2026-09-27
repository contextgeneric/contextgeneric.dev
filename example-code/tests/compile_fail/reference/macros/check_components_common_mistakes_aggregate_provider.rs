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

delegate_components! {
    new GeometryComponents {
        AreaCalculatorComponent: RectangleArea,
    }
}

// error[E0277]: `GeometryComponents: CanUseComponent<AreaCalculatorComponent>` is not satisfied
check_components! {
    GeometryComponents {
        AreaCalculatorComponent,
    }
}

fn main() {}
