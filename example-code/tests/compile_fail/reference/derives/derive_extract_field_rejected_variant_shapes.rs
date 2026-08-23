use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Shape {
    Circle { radius: f64 },
    Empty,
}

fn main() {}
