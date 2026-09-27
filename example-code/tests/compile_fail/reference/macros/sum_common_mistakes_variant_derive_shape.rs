use cgp::prelude::*;

// error: Expected variant to contain exactly one unnamed field
#[derive(CgpData)]
pub enum Shape {
    Circle(f64),
    Rectangle { width: f64, height: f64 },
}

fn main() {}
