//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: an enum without the extensible-data derive cannot be dispatched.

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

pub enum Plain {
    Circle(Circle),
}

#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}

impl HasArea for Circle {
    fn area(&self) -> f64 {
        1.0
    }
}

pub fn area_of(shape: &Plain) -> f64 {
    shape.area()
}

fn main() {}
