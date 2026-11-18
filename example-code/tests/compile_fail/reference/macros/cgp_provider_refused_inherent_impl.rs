//! `docs/reference/macros/cgp_provider.md`, *Input the macros refuse*: an inherent impl.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

pub struct RectangleArea;

#[cgp_provider]
impl RectangleArea {
    pub fn area() -> f64 {
        1.0
    }
}

fn main() {}
