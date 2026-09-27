//! `docs/reference/macros/cgp_impl.md`, *Implementing the consumer trait directly*: a direct impl
//! and a wiring entry for the same component overlap.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new UnitArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        1.0
    }
}

pub struct Square;

#[cgp_impl(Self)]
impl CanCalculateArea for Square {
    fn area(&self) -> f64 {
        1.0
    }
}

delegate_components! {
    Square {
        AreaCalculatorComponent: UnitArea,
    }
}

fn main() {}
