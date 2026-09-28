use cgp::prelude::*;

// error: Mutable variables are not allowed in implicit arguments.
#[cgp_fn]
fn area(&self, #[implicit] mut width: f64) -> f64 {
    width += 1.0;
    width
}

fn main() {}
