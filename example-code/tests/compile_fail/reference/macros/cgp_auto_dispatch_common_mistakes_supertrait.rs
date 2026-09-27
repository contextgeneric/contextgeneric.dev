//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: a supertrait is not carried onto the blanket impl.

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

pub trait Named {
    fn name(&self) -> String;
}

#[cgp_auto_dispatch]
pub trait HasArea: Named {
    fn area(&self) -> f64;
}

fn main() {}
