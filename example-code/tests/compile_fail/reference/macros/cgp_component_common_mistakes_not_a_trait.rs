//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: the attribute must be applied to a
//! trait.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

fn main() {}
