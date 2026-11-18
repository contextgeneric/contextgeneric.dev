use cgp::prelude::*;

// error: expected `struct`
#[derive(CgpRecord)]
pub enum Shape {
    Circle(f64),
}

fn main() {}
