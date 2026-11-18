//! `docs/reference/macros/cgp_auto_getter.md`, *Common Mistakes*: a `&mut` return needs a `&mut self` receiver.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasCounter {
    fn counter(&self) -> &mut u32;
}

fn main() {}
