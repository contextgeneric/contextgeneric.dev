//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: two dispatch traits in one
//! module sharing a method name emit the same helper and provider names.

use cgp::prelude::*;

#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}

#[cgp_auto_dispatch]
pub trait HasSurface {
    fn area(&self) -> f64;
}

fn main() {}
