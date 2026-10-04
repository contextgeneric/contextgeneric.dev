//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: a typed receiver such as
//! `self: Box<Self>` is rejected, since no matcher takes it.

use cgp::prelude::*;

#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(self: Box<Self>) -> f64;
}

fn main() {}
