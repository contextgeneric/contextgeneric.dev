//! `docs/reference/macros/cgp_provider.md`, *Common Mistakes*: a component argument that does not
//! belong to the provider trait registers the provider under the wrong key.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(PerimeterCalculator)]
pub trait CanCalculatePerimeter {
    fn perimeter(&self) -> f64;
}

pub struct UnitArea;

#[cgp_provider(PerimeterCalculatorComponent)]
impl<Context> AreaCalculator<Context> for UnitArea {
    fn area(_context: &Context) -> f64 {
        1.0
    }
}

pub struct Square;

delegate_components! {
    Square {
        AreaCalculatorComponent: UnitArea,
    }
}

check_components! {
    Square {
        AreaCalculatorComponent,
    }
}

fn main() {}
