//! `docs/reference/macros/cgp_impl.md`, *Implementing the consumer trait directly*: the `Self` form
//! needs the `for Context` clause.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(Self)]
impl CanCalculateArea {
    fn area(&self) -> f64 {
        1.0
    }
}

fn main() {}
