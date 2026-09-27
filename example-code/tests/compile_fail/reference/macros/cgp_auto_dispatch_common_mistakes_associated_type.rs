//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: every trait item must be a method.

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
pub trait HasUnit {
    type Unit;

    fn area(&self) -> f64;
}

fn main() {}
