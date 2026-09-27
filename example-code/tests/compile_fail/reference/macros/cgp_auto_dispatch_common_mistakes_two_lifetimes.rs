//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: a method needing two distinct lifetimes fails.

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
pub trait Lookup {
    fn lookup<'a>(&'a self, key: &str) -> &'a str;
}

fn main() {}
