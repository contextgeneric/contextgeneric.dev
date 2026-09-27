//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: the per-method helper function takes the method's name in the module.

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

pub fn area() {}

fn main() {}
