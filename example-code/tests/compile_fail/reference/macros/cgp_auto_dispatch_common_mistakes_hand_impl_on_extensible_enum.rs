//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: a hand-written impl collides for a type the blanket impl covers.

use cgp::prelude::*;

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

pub struct Circle {
    pub radius: f64,
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}

impl HasArea for Shape {
    fn area(&self) -> f64 {
        0.0
    }
}

fn main() {}
