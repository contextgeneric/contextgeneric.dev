use cgp::prelude::*;

// error: expected `struct`
#[derive(BuildField)]
pub enum Shape {
    Circle(f64),
}

fn main() {}
