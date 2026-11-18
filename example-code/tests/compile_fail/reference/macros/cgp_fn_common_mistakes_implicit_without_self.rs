//! `docs/reference/macros/cgp_fn.md`, *Common Mistakes*: implicit arguments need a `self` receiver.

use cgp::prelude::*;

#[cgp_fn]
pub fn rectangle_area(#[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}

fn main() {}
