use cgp::prelude::*;

// error: `#[implicit]` does not take any arguments; write it as a bare `#[implicit]`
#[cgp_fn]
fn area(&self, #[implicit(width)] width: f64) -> f64 {
    width
}

fn main() {}
