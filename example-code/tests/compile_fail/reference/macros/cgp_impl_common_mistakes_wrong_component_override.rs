//! `docs/reference/macros/cgp_impl.md`, *Common Mistakes*: a `: ComponentType` override that does
//! not belong to the provider trait makes the generated impl fail its `IsProviderFor` supertrait.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(PerimeterCalculator)]
pub trait CanCalculatePerimeter {
    fn perimeter(&self) -> f64;
}

#[cgp_impl(new Wrong: PerimeterCalculatorComponent)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        1.0
    }
}

fn main() {}
