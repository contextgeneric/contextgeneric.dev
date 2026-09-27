//! `docs/reference/macros/cgp_provider.md`, *Common Mistakes*: `new` is not part of
//! `#[cgp_provider]`'s argument.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_provider(new RectangleArea)]
impl<Context> AreaCalculator<Context> for RectangleArea {
    fn area(_context: &Context) -> f64 {
        1.0
    }
}

fn main() {}
