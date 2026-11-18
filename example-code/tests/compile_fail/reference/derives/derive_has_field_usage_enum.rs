use cgp::prelude::*;

// error: expected `struct`
#[derive(HasField)]
pub enum Shape {
    Circle(f64),
}

fn main() {}
