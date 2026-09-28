use cgp::prelude::*;

// error: expected `enum`
#[derive(CgpVariant)]
pub struct Circle {
    pub radius: f64,
}

fn main() {}
