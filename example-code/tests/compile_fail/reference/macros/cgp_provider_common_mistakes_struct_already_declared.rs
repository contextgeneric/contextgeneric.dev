//! `docs/reference/macros/cgp_provider.md`, *Common Mistakes*: `#[cgp_new_provider]` over a struct
//! that is already declared defines it twice.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

pub struct RectangleArea;

#[cgp_new_provider]
impl<Context> AreaCalculator<Context> for RectangleArea {
    fn area(_context: &Context) -> f64 {
        1.0
    }
}

fn main() {}
