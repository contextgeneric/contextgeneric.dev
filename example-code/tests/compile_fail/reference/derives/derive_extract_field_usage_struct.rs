use cgp::prelude::*;

// error: expected `enum`
#[derive(ExtractField)]
pub struct Circle {
    pub radius: f64,
}

fn main() {}
