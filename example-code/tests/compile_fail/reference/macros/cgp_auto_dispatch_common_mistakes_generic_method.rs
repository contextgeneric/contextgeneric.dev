//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: a dispatch method cannot take a type parameter.

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
pub trait CanDescribe {
    fn describe<T: Default>(&self) -> T;
}

fn main() {}
