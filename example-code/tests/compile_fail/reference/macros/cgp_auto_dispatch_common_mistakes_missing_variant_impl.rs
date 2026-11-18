//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: a variant payload without an impl fails where the method is called.

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

impl HasArea for Circle {
    fn area(&self) -> f64 {
        1.0
    }
}

pub fn area_of(shape: &Shape) -> f64 {
    shape.area()
}

fn main() {}
