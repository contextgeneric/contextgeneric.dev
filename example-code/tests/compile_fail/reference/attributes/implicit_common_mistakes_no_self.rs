use cgp::prelude::*;

// error: The first argument of a function with implicit arguments must be `self`
#[cgp_fn]
fn area(#[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}

fn main() {}
