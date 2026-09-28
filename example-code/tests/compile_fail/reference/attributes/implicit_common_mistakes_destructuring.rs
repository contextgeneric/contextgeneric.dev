use cgp::prelude::*;

// error: Expected an identifier
#[cgp_fn]
fn area(&self, #[implicit] (width, height): (f64, f64)) -> f64 {
    width * height
}

fn main() {}
