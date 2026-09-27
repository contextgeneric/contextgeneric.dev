//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: every method needs a `self` receiver.

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
pub trait HasOrigin {
    fn origin() -> (f64, f64);
}

fn main() {}
