use cgp::prelude::*;

// error: expected `enum`
#[derive(FromVariant)]
pub struct Circle(pub f64);

fn main() {}
