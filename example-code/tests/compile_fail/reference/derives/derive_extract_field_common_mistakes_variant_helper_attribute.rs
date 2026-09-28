use cgp::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Circle {
    pub radius: f64,
}

// error: cannot find attribute `serde` in this scope
#[derive(Serialize, ExtractField)]
pub enum Shape {
    #[serde(rename = "circle")]
    Circle(Circle),
}

fn main() {}
